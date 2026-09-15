use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::tempdir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_maverick"))
}

fn stub_grok(dir: &Path) -> PathBuf {
    let path = dir.join("grok");
    fs::write(
        &path,
        r#"#!/bin/sh
if [ "$1" = models ]; then
cat <<'EOF'
You are logged in with grok.com.

Default model: grok-4.6

Available models:
  * grok-4.6 (default)
  - grok-4.5
EOF
fi
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    path
}

#[test]
fn models_json_uses_discovery_without_creating_run_storage() {
    let root = tempdir().unwrap();
    let path_dir = root.path().join("bin");
    fs::create_dir(&path_dir).unwrap();
    stub_grok(&path_dir);
    let config = root.path().join("user.toml");
    let path = format!(
        "{}:{}",
        path_dir.display(),
        std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".into())
    );
    let output = bin()
        .env("PATH", &path)
        .env("MAVERICK_CONFIG", &config)
        .env_remove("XAI_API_KEY")
        .args(["models", "--root"])
        .arg(root.path().join("runtime"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["execution"], "disabled");
    assert_eq!(value["setup"]["providers"][0]["id"], "grok");
    assert_eq!(
        value["setup"]["providers"][0]["models"],
        serde_json::json!(["grok-4.6", "grok-4.5"])
    );
    assert!(!value.to_string().contains("secret"));
    assert!(!root.path().join("runtime/maverick.db").exists());
    assert!(!config.exists());
}

#[test]
fn project_overlay_wins_and_non_tty_bare_command_does_not_enter_tui() {
    let root = tempdir().unwrap();
    let user = root.path().join("user.toml");
    fs::write(
        &user,
        r#"
schema_version = 1
[[providers]]
id = "grok"
kind = "grok"
display_name = "Grok CLI"
models = ["grok-4.6"]
"#,
    )
    .unwrap();
    let project = root.path().join("project.toml");
    fs::write(
        &project,
        r#"
schema_version = 1
[[providers]]
id = "local-llm"
kind = "openai_compatible"
display_name = "Local"
base_url = "http://127.0.0.1:11434/v1"
models = ["llama3.2"]

[default]
provider = "local-llm"
model = "llama3.2"

[[roles.planner]]
provider = "local-llm"
model = "llama3.2"
"#,
    )
    .unwrap();
    let output = bin()
        .env("PATH", "/usr/bin:/bin")
        .env("MAVERICK_CONFIG", &user)
        .env_remove("XAI_API_KEY")
        .args(["models", "--config"])
        .arg(&user)
        .arg("--project")
        .arg(&project)
        .arg("--root")
        .arg(root.path().join("runtime"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["setup"]["default"]["provider"], "local-llm");
    assert_eq!(value["setup"]["roles"]["planner"][0]["model"], "llama3.2");

    let bare = bin()
        .env("MAVERICK_CONFIG", &user)
        .arg("--root")
        .arg(root.path().join("runtime"))
        .output()
        .unwrap();
    assert_eq!(bare.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bare.stderr).contains("TTY"));
    assert!(!root.path().join("runtime/maverick.db").exists());
}

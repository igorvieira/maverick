use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::tempdir;

fn command(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_maverick"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn main_flow_survives_separate_processes_and_rejects_invalid_transition() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("runtime");
    let start = success(command(&root, &["start", "--task", "Sample task"]));
    let id = start["id"].as_str().unwrap();
    assert_eq!(start["state"], "created");
    for to in ["context-resolved", "planned"] {
        assert_eq!(
            success(command(&root, &["transition", id, "--to", to]))["state"],
            to
        );
    }
    let status = success(command(&root, &["status", id]));
    let history = success(command(&root, &["history", id]));
    assert_eq!(status["state"], "planned");
    assert_eq!(history.as_array().unwrap().len(), 2);
    let payload = dir.path().join("payload.json");
    fs::write(&payload, r#"{"steps":["sample"]}"#).unwrap();
    let artifact = success(command(
        &root,
        &[
            "artifact",
            id,
            "--kind",
            "plan",
            "--name",
            "plan.json",
            "--file",
            payload.to_str().unwrap(),
        ],
    ));
    assert_eq!(artifact["revision"], 1);
    assert_eq!(artifact["schema_version"], 1);
    let bad = command(&root, &["transition", id, "--to", "completed"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("invalid transition"));
    assert_eq!(success(command(&root, &["status", id])), status);
    assert_eq!(success(command(&root, &["history", id])), history);
    let saved: Value =
        serde_json::from_slice(&fs::read(root.join("runs").join(id).join("plan.json")).unwrap())
            .unwrap();
    assert_eq!(saved, artifact);
    for to in [
        "awaiting-approval",
        "implementing",
        "reviewing",
        "fixing",
        "reviewing",
        "testing",
        "delivering",
        "completed",
    ] {
        success(command(&root, &["transition", id, "--to", to]));
    }
    assert!(!command(&root, &["transition", id, "--to", "failed"])
        .status
        .success());
    assert_eq!(
        success(command(&root, &["status", id]))["state"],
        "completed"
    );
}

#[test]
fn input_errors_have_nonzero_exit_codes_and_do_not_publish_artifacts() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("runtime");
    let start = success(command(&root, &["start", "--task", "Task"]));
    let id = start["id"].as_str().unwrap();
    let payload = dir.path().join("invalid.json");
    fs::write(&payload, "{invalid}").unwrap();
    let bad = command(
        &root,
        &[
            "artifact",
            id,
            "--kind",
            "plan",
            "--name",
            "plan.json",
            "--file",
            payload.to_str().unwrap(),
        ],
    );
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("invalid JSON"));
    fs::write(&payload, "{}").unwrap();
    for name in ["../plan.json", "/tmp/escape.json", "..\\plan.json"] {
        assert!(!command(
            &root,
            &[
                "artifact",
                id,
                "--kind",
                "plan",
                "--name",
                name,
                "--file",
                payload.to_str().unwrap()
            ]
        )
        .status
        .success());
    }
    for args in [
        vec!["start", "--task", " "],
        vec!["status", "../escape"],
        vec!["status", "00000000-0000-0000-0000-000000000000"],
        vec!["transition", id, "--to", "unknown"],
        vec!["shell", "echo"],
        vec!["history", "bad-id"],
    ] {
        assert!(!command(&root, &args).status.success());
    }
    assert!(!root.join("runs").join(id).join("plan.json").exists());
    assert_eq!(success(command(&root, &["history", id])), json!([]));
}

#[test]
fn default_root_and_global_option_after_subcommand() {
    let dir = tempdir().unwrap();
    let start = success(
        Command::new(env!("CARGO_BIN_EXE_maverick"))
            .current_dir(dir.path())
            .args(["start", "--task", "Task"])
            .output()
            .unwrap(),
    );
    assert!(dir.path().join(".maverick/maverick.db").is_file());
    let output = Command::new(env!("CARGO_BIN_EXE_maverick"))
        .args(["status", start["id"].as_str().unwrap(), "--root"])
        .arg(dir.path().join(".maverick"))
        .output()
        .unwrap();
    assert_eq!(success(output), start);
}

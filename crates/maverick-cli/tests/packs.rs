use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::tempdir;

fn source(id: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../claude/agents")
        .join(id)
        .join("pack.json")
}

#[test]
fn both_packs_use_the_same_cli_path_without_run_storage() {
    for (id, caps, expected) in [
        (
            "go",
            ["domain", "eventing"],
            vec!["idiom", "domain", "eventing"],
        ),
        (
            "typescript",
            ["frontend", "api_contract"],
            vec!["type_safety", "frontend", "contracts"],
        ),
    ] {
        let root = tempdir().unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_maverick"))
            .current_dir(root.path())
            .arg("pack")
            .arg("--manifest")
            .arg(source(id))
            .args(["--capability", caps[0], "--capability", caps[1]])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        let selected: Vec<_> = result["packs"][0]["reviewers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(selected, expected);
        assert_eq!(result["execution"], "disabled");
        assert!(!root.path().join(".maverick").exists());
    }
}

#[test]
fn cli_rejects_invalid_manifests_capabilities_classification_and_duplicate_packs() {
    let root = tempdir().unwrap();
    let manifest = root.path().join("bad.json");
    fs::write(&manifest, "not JSON").unwrap();
    let bad = Command::new(env!("CARGO_BIN_EXE_maverick"))
        .args(["pack", "--manifest"])
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("invalid manifest"));
    let bad = Command::new(env!("CARGO_BIN_EXE_maverick"))
        .args(["pack", "--manifest"])
        .arg(source("go"))
        .args(["--capability", "Not Valid"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    let bad = Command::new(env!("CARGO_BIN_EXE_maverick"))
        .args(["pack", "--manifest"])
        .arg(source("go"))
        .arg("--classification")
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("invalid classification"));
    let duplicate = Command::new(env!("CARGO_BIN_EXE_maverick"))
        .args(["pack", "--manifest"])
        .arg(source("go"))
        .arg("--manifest")
        .arg(source("go"))
        .output()
        .unwrap();
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("duplicate pack id"));
}

#[test]
fn cli_loads_installed_layout_classification_and_adapter_without_executing_commands() {
    let root = tempdir().unwrap();
    let project = root.path().join("project");
    fs::create_dir(&project).unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let installed = Command::new("bash")
        .arg(repo.join("setup.sh"))
        .arg("project")
        .arg(&project)
        .args(["--pack", "go", "--pack", "typescript"])
        .output()
        .unwrap();
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    let adapter = project.join(".claude/maverick/project.json");
    let marker = project.join("must-not-exist");
    fs::write(&adapter, json!({"schema_version":1,"packs":["typescript","go"],"commands":{"test":format!("touch {}", marker.display())}}).to_string()).unwrap();
    let classification = project.join("classification.json");
    fs::write(
        &classification,
        json!({"capabilities":["frontend","api_contract"]}).to_string(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_maverick"))
        .args(["pack", "--manifest"])
        .arg(project.join(".claude/maverick/packs/go.json"))
        .arg("--manifest")
        .arg(project.join(".claude/maverick/packs/typescript.json"))
        .arg("--agents-dir")
        .arg(project.join(".claude/agents"))
        .arg("--adapter")
        .arg(&adapter)
        .arg("--classification")
        .arg(&classification)
        .arg("--project")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["packs"][0]["id"], "typescript");
    assert_eq!(result["packs"][1]["id"], "go");
    assert_eq!(result["packs"][0]["reviewers"].as_array().unwrap().len(), 3);
    assert!(result["packs"][0]["commands"]["test"]
        .as_str()
        .unwrap()
        .starts_with("touch "));
    assert!(!marker.exists());
}

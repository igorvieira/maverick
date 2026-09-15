use maverick_core::{pack::*, Finding, ReviewResult, ReviewStatus, Severity};
use maverick_packs::*;
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use tempfile::tempdir;

fn source(id: &str) -> (PathBuf, PathBuf) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../claude/agents")
        .join(id);
    (dir.join("pack.json"), dir)
}
fn pack(id: &str) -> PackManifest {
    let (manifest, agents) = source(id);
    load_pack(&manifest, &agents).unwrap()
}
fn capabilities(values: &[&str]) -> HashSet<Capability> {
    values.iter().map(|v| v.parse().unwrap()).collect()
}
fn selected(pack: &PackManifest, caps: &[&str]) -> Vec<String> {
    pack.select_reviewers(&capabilities(caps))
        .unwrap()
        .iter()
        .map(|r| r.id.clone())
        .collect()
}
fn value() -> Value {
    serde_json::to_value(pack("go")).unwrap()
}

#[test]
fn both_packs_satisfy_the_same_loading_detection_and_routing_contract() {
    // One parameterized contract: no language-dependent execution path.
    for (id, required, capability, conditional, count) in [
        ("go", "idiom", "domain", "domain", 6),
        ("typescript", "type_safety", "frontend", "frontend", 3),
    ] {
        let pack = pack(id);
        assert_eq!(pack.id, id);
        assert_eq!(pack.reviewers.len(), count);
        assert_eq!(selected(&pack, &[]), [required]);
        assert_eq!(selected(&pack, &[capability]), [required, conditional]);
        assert_eq!(selected(&pack, &["future_capability"]), [required]);
        let project = tempdir().unwrap();
        assert!(!detect_pack(&pack, project.path()).unwrap());
        for file in pack
            .detection
            .files
            .iter()
            .chain(pack.detection.any_files.iter().take(1))
        {
            fs::write(project.path().join(file), "").unwrap();
        }
        assert!(detect_pack(&pack, project.path()).unwrap());
        assert!(pack.commands.test.is_some());
        pack.validate().unwrap();
    }
}

#[test]
fn schema_and_pack_ids_and_unknown_fields_are_rejected() {
    let mut manifest = value();
    manifest["schema_version"] = json!(99);
    assert!(matches!(
        PackManifest::from_json(&manifest.to_string()),
        Err(PackError::SchemaVersion(99))
    ));
    for id in ["", "../escape", "Upper", "two words", "-bad", "bad__id"] {
        let mut manifest = value();
        manifest["id"] = json!(id);
        assert!(PackManifest::from_json(&manifest.to_string())
            .unwrap_err()
            .to_string()
            .contains("pack id"));
    }
    let mut manifest = value();
    manifest["commandz"] = json!({});
    assert!(PackManifest::from_json(&manifest.to_string()).is_err());
    assert!(PackManifest::from_json("not JSON").is_err());
}

#[test]
fn duplicate_reviewer_ids_and_pack_ids_are_rejected() {
    let mut manifest = value();
    manifest["reviewers"][1]["id"] = manifest["reviewers"][0]["id"].clone();
    assert!(
        matches!(PackManifest::from_json(&manifest.to_string()), Err(PackError::Duplicate { field, .. }) if field == "reviewer id")
    );
    let src = source("go");
    assert!(
        matches!(load_packs(&[src.clone(), src]), Err(LoadError::Contract(PackError::Duplicate { field, .. })) if field == "pack id")
    );
    let mut manifest = value();
    manifest["reviewers"][1]["agent"] = manifest["reviewers"][0]["agent"].clone();
    assert!(
        matches!(PackManifest::from_json(&manifest.to_string()), Err(PackError::Duplicate { field, .. }) if field == "reviewer agent")
    );
}

#[test]
fn capabilities_validate_on_construction_and_deserialization() {
    for invalid in [
        "",
        " ",
        "domain-model",
        "Domain",
        "api contract",
        "a__b",
        "_a",
        "a_",
        "1a",
        "a/b",
        "área",
    ] {
        assert!(invalid.parse::<Capability>().is_err(), "{invalid:?}");
        assert!(serde_json::from_value::<Capability>(json!(invalid)).is_err());
        let mut manifest = value();
        manifest["reviewers"][1]["capabilities"] = json!([invalid]);
        assert!(PackManifest::from_json(&manifest.to_string()).is_err());
    }
    let cap: Capability = "new_capability_2".parse().unwrap();
    assert_eq!(
        serde_json::to_value(&cap).unwrap(),
        json!("new_capability_2")
    );
    let restored: Capability = serde_json::from_value(json!("new_capability_2")).unwrap();
    assert_eq!(cap, restored);
    assert_eq!(HashSet::from([cap, restored]).len(), 1);
}

#[test]
fn routing_preserves_order_and_never_duplicates_multi_capability_matches() {
    let go = pack("go");
    assert_eq!(
        selected(&go, &["eventing", "domain", "domain"]),
        ["idiom", "domain", "eventing"]
    );
    assert_eq!(
        selected(&go, &["adapters", "api_contract", "database"]),
        ["idiom", "adapter"]
    );
    let ts = pack("typescript");
    assert_eq!(
        selected(&ts, &["api_contract", "frontend", "accessibility"]),
        ["type_safety", "frontend", "contracts"]
    );
    let mut reordered = ts.clone();
    reordered.reviewers.reverse();
    assert_eq!(
        selected(&reordered, &["frontend", "api_contract"]),
        ["contracts", "frontend", "type_safety"]
    );
}

#[test]
fn unrelated_manifest_ids_agents_and_novel_capabilities_route_identically() {
    for id in ["go", "typescript"] {
        let mut manifest = pack(id);
        manifest.id = "independent-example".into();
        manifest.agents.planner = "custom-planner".into();
        manifest.agents.implementer = "custom-implementer".into();
        manifest.agents.red_team = None;
        manifest.red_team = None;
        manifest.reviewers.truncate(2);
        for (i, reviewer) in manifest.reviewers.iter_mut().enumerate() {
            reviewer.id = format!("reviewer_{i}");
            reviewer.agent = format!("prompt-{i}");
            reviewer.capabilities = vec!["novel_feature".parse().unwrap()];
        }
        assert_eq!(selected(&manifest, &[]), ["reviewer_0"]);
        assert_eq!(
            selected(&manifest, &["novel_feature"]),
            ["reviewer_0", "reviewer_1"]
        );
        assert_eq!(selected(&manifest, &["unknown_feature"]), ["reviewer_0"]);
    }
}

#[test]
fn command_sets_are_optional_distinct_and_adapter_overrides_are_explicit() {
    let go = pack("go");
    let ts = pack("typescript");
    assert!(go.commands.format.is_some());
    assert!(go.commands.typecheck.is_none());
    assert!(go.commands.lint.is_none());
    assert!(ts.commands.format.is_none());
    assert!(ts.commands.typecheck.is_some());
    let adapter = ProjectAdapter::from_json(
        &json!({
            "schema_version": 1, "packs": ["go", "typescript"],
            "commands": {"test": "project-test"},
            "pack_commands": {"typescript": {"test": "package-test", "build": "package-build"}},
            "project_commands": {"migrate": "project-migrate"}
        })
        .to_string(),
    )
    .unwrap();
    let go_commands = adapter.resolve_commands(&go).unwrap();
    assert_eq!(go_commands.test.as_deref(), Some("project-test"));
    assert_eq!(go_commands.build, go.commands.build);
    assert!(go_commands.typecheck.is_none());
    let ts_commands = adapter.resolve_commands(&ts).unwrap();
    assert_eq!(ts_commands.test.as_deref(), Some("package-test"));
    assert_eq!(ts_commands.build.as_deref(), Some("package-build"));
    assert!(ts_commands.format.is_none());
    assert_eq!(adapter.project_commands["migrate"], "project-migrate");
    assert!(ProjectAdapter::from_json(r#"{"schema_version":1,"packs":["go","go"]}"#).is_err());
    assert!(ProjectAdapter::from_json(
        r#"{"schema_version":1,"packs":[],"commands":{"test":" "}}"#
    )
    .is_err());
    assert!(ProjectAdapter::from_json(
        r#"{"schema_version":1,"packs":[],"pack_commands":{"missing":{}}}"#
    )
    .is_err());
}

#[test]
fn explicit_multi_pack_selection_overrides_detection_and_preserves_adapter_order() {
    let packs = load_packs(&[source("go"), source("typescript")]).unwrap();
    let project = tempdir().unwrap();
    assert!(select_packs(&packs, None, project.path())
        .unwrap()
        .is_empty());
    let adapter =
        ProjectAdapter::from_json(r#"{"schema_version":1,"packs":["typescript","go"]}"#).unwrap();
    let selected = select_packs(&packs, Some(&adapter), project.path()).unwrap();
    assert_eq!(
        selected.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(),
        ["typescript", "go"]
    );
    let empty = ProjectAdapter::from_json(r#"{"schema_version":1,"packs":[]}"#).unwrap();
    fs::write(project.path().join("go.mod"), "").unwrap();
    assert!(select_packs(&packs, Some(&empty), project.path())
        .unwrap()
        .is_empty());
    assert_eq!(
        select_packs(&packs, None, project.path()).unwrap()[0].id,
        "go"
    );
    let unknown = ProjectAdapter::from_json(r#"{"schema_version":1,"packs":["missing"]}"#).unwrap();
    assert!(matches!(
        select_packs(&packs, Some(&unknown), project.path()),
        Err(LoadError::Contract(PackError::UnknownPack(_)))
    ));
}

#[test]
fn detection_requires_all_files_and_at_least_one_any_file() {
    let rules = DetectionRules {
        files: vec!["required".into()],
        any_files: vec!["one".into(), "two".into()],
    };
    assert!(!rules.matches(&HashSet::from(["one".into()])));
    assert!(!rules.matches(&HashSet::from(["required".into()])));
    assert!(rules.matches(&HashSet::from(["required".into(), "two".into()])));
    assert!(!DetectionRules {
        files: vec![],
        any_files: vec![]
    }
    .matches(&HashSet::new()));
    let mut manifest = value();
    for path in ["../outside", "/absolute", "a\\b", "**/*.json", "a/../b"] {
        manifest["detection"]["files"] = json!([path]);
        assert!(PackManifest::from_json(&manifest.to_string()).is_err());
    }
}

#[test]
fn every_missing_agent_slot_is_detected_before_routing() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("pack.json");
    let (_, agents) = source("go");
    for pointer in [
        "/agents/planner",
        "/agents/implementer",
        "/agents/red_team",
        "/reviewers/0/agent",
    ] {
        let mut manifest = value();
        *manifest.pointer_mut(pointer).unwrap() = json!("missing-agent");
        fs::write(&path, manifest.to_string()).unwrap();
        assert!(
            matches!(load_pack(&path, &agents), Err(LoadError::MissingAgent { agent, .. }) if agent == "missing-agent")
        );
    }
    let mut manifest = value();
    manifest["agents"]["planner"] = json!("../../outside");
    assert!(PackManifest::from_json(&manifest.to_string()).is_err());
}

#[cfg(unix)]
#[test]
fn agent_symlink_cannot_escape_its_root() {
    let temp = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let manifest = pack("typescript");
    for agent in manifest.agent_references() {
        fs::write(temp.path().join(format!("{agent}.md")), "prompt").unwrap();
    }
    let planner = temp.path().join(format!("{}.md", manifest.agents.planner));
    fs::remove_file(&planner).unwrap();
    let target = outside.path().join("outside.md");
    fs::write(&target, "external prompt").unwrap();
    std::os::unix::fs::symlink(target, planner).unwrap();
    let file = temp.path().join("pack.json");
    fs::write(&file, serde_json::to_string(&manifest).unwrap()).unwrap();
    assert!(matches!(
        load_pack(&file, temp.path()),
        Err(LoadError::MissingAgent { .. })
    ));
}

#[test]
fn go_gates_and_legacy_policies_are_preserved_in_the_manifest() {
    let go = pack("go");
    for capability in [
        "domain",
        "eventing",
        "aggregate_design",
        "transaction_boundaries",
        "cross_service_consistency",
        "eventing_strategy",
    ] {
        assert!(go.requires_red_team(&capabilities(&[capability])).unwrap());
    }
    for capability in ["pure_implementation", "adapters"] {
        assert!(!go.requires_red_team(&capabilities(&[capability])).unwrap());
    }
    assert_eq!(selected(&go, &["concurrency"]), ["idiom", "architecture"]);
    assert_eq!(go.review_policy.max_fix_rounds, 3);
    assert!(go.review_policy.rerun_blocked_only);
    assert!(go.review_policy.resolve_warnings);
    for reviewer in &go.reviewers {
        assert!(reviewer
            .legacy_verdicts
            .values()
            .any(|s| *s == ReviewStatus::Fail));
        assert!(reviewer
            .legacy_verdicts
            .values()
            .any(|s| *s == ReviewStatus::Warn));
        assert!(reviewer
            .legacy_verdicts
            .values()
            .any(|s| *s == ReviewStatus::Pass));
    }
    assert_eq!(
        go.red_team.as_ref().unwrap().legacy_verdicts["Architecture has critical flaws"],
        ReviewStatus::Fail
    );
    assert!(!pack("typescript")
        .requires_red_team(&capabilities(&["domain", "eventing"]))
        .unwrap());
    let mut invalid = go.clone();
    invalid.agents.red_team = None;
    assert!(invalid.validate().is_err());
    invalid = go;
    invalid.reviewers[0].blocking.severities.clear();
    assert!(invalid.validate().is_err());
}

#[test]
fn aggregation_uses_structured_results_and_fails_closed_on_missing_reviews() {
    for id in ["go", "typescript"] {
        let pack = pack(id);
        let caps = capabilities(&[]);
        let required = &pack.reviewers[0].id;
        let missing = pack.aggregate_reviews(&caps, &[]).unwrap();
        assert!(missing.is_blocking());
        assert_eq!(missing.missing, std::slice::from_ref(required));
        let mut result = ReviewResult {
            reviewer: required.clone(),
            status: ReviewStatus::Pass,
            findings: vec![],
        };
        assert!(!pack
            .aggregate_reviews(&caps, std::slice::from_ref(&result))
            .unwrap()
            .is_blocking());
        result.status = ReviewStatus::Fail;
        assert_eq!(
            pack.aggregate_reviews(&caps, std::slice::from_ref(&result))
                .unwrap()
                .blocked,
            std::slice::from_ref(required)
        );
        result.status = ReviewStatus::Warn;
        for severity in [Severity::Blocker, Severity::High] {
            result.findings = vec![Finding {
                severity,
                rule: "sample".into(),
                file: None,
                line: None,
                message: "Positive prose cannot override severity".into(),
                suggested_fix: None,
            }];
            assert!(pack
                .aggregate_reviews(&caps, std::slice::from_ref(&result))
                .unwrap()
                .is_blocking());
        }
        result.findings.clear();
        let summary = pack
            .aggregate_reviews(&caps, std::slice::from_ref(&result))
            .unwrap();
        assert!(!summary.is_blocking());
        assert_eq!(summary.warnings, std::slice::from_ref(required));
        assert!(pack
            .aggregate_reviews(&caps, &[result.clone(), result.clone()])
            .is_err());
        result.reviewer = "unselected".into();
        assert!(pack.aggregate_reviews(&caps, &[result]).is_err());
    }
}

#[test]
fn omitted_optional_fields_are_absent_not_invented() {
    let parsed = PackManifest::from_json(
        r#"{
            "schema_version": 1,
            "id": "example",
            "display_name": "Example",
            "detection": {"files": ["example.lock"]},
            "commands": {"test": "example-test"},
            "agents": {"planner": "example-planner", "implementer": "example-implementer"},
            "reviewers": [{
                "id": "always_reviewer",
                "agent": "example-reviewer",
                "always": true,
                "capabilities": [],
                "blocking": {"statuses": ["fail"], "severities": ["blocker", "high"]}
            }],
            "review_policy": {"max_fix_rounds": 1, "rerun_blocked_only": true, "resolve_warnings": false}
        }"#,
    )
    .unwrap();
    assert!(parsed.red_team.is_none());
    assert!(parsed.agents.red_team.is_none());
    assert!(parsed.commands.format.is_none());
    assert!(parsed.commands.build.is_none());
    assert!(parsed.commands.lint.is_none());
    assert!(parsed.commands.typecheck.is_none());
    assert_eq!(parsed.commands.test.as_deref(), Some("example-test"));
    assert!(parsed.detection.any_files.is_empty());
    assert_eq!(
        selected(&parsed, &["unknown_capability"]),
        ["always_reviewer"]
    );
}

#[test]
fn runtime_source_has_no_language_or_agent_name_branches() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![manifest_dir.join("../maverick-cli/src/packs.rs")];
    for dir in [
        manifest_dir.join("src"),
        manifest_dir.join("../maverick-core/src/pack"),
    ] {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    let needles = [
        "if language",
        "language ==",
        "pack.id ==",
        "== \"go\"",
        "== \"typescript\"",
        "gofmt",
        "golangci",
        "npm run",
        "go-idiom-reviewer",
        "go-domain-model",
        "typescript-frontend-reviewer",
        "typescript-implementer",
    ];
    for path in files {
        let src = fs::read_to_string(&path).unwrap();
        for needle in needles {
            assert!(
                !src.contains(needle),
                "{} must not contain language-specific {needle:?}",
                path.display()
            );
        }
    }
}

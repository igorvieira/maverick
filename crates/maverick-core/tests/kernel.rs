use maverick_core::*;
use serde_json::json;

const STATES: [RunState; 11] = [
    RunState::Created,
    RunState::ContextResolved,
    RunState::Planned,
    RunState::AwaitingApproval,
    RunState::Implementing,
    RunState::Reviewing,
    RunState::Fixing,
    RunState::Testing,
    RunState::Delivering,
    RunState::Completed,
    RunState::Failed,
];

#[test]
fn complete_lifecycle_including_fix_loop() {
    let mut run = Run::new("Implement sample task".into(), 100).unwrap();
    let id = run.id();
    for state in [
        RunState::ContextResolved,
        RunState::Planned,
        RunState::AwaitingApproval,
        RunState::Implementing,
        RunState::Reviewing,
        RunState::Fixing,
        RunState::Reviewing,
        RunState::Testing,
        RunState::Delivering,
        RunState::Completed,
    ] {
        run.transition(state).unwrap();
        assert_eq!(run.state(), state);
    }
    assert_eq!(run.id(), id);
    assert_eq!(run.revision(), 10);
    assert_eq!(run.created_at(), 100);
    assert_eq!(run.task(), "Implement sample task");
}

#[test]
fn every_state_pair_obeys_the_contract_and_invalid_transitions_are_immutable() {
    // Explicit adjacency specification, independent of implementation matching.
    let allowed = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 4),
        (4, 5),
        (5, 6),
        (6, 5),
        (5, 7),
        (7, 8),
        (8, 9),
    ];
    for (i, from) in STATES.iter().enumerate() {
        for (j, to) in STATES.iter().enumerate() {
            let mut snapshot = serde_json::to_value(Run::new("Task".into(), 0).unwrap()).unwrap();
            snapshot["state"] = json!(from);
            let mut run: Run = serde_json::from_value(snapshot).unwrap();
            let before = run.clone();
            let valid = allowed.contains(&(i, j)) || (i < 9 && j == 10);
            assert_eq!(run.transition(*to).is_ok(), valid, "{from} -> {to}");
            if !valid {
                assert_eq!(run, before);
            }
        }
    }
}

#[test]
fn identity_task_and_state_parsing() {
    let first = Run::new("Task".into(), 0).unwrap();
    assert_ne!(first.id(), Run::new("Task".into(), 0).unwrap().id());
    assert_eq!(first.id().to_string().parse::<RunId>().unwrap(), first.id());
    for state in STATES {
        assert_eq!(state.to_string().parse::<RunState>().unwrap(), state);
    }
    assert!("../escape".parse::<RunId>().is_err());
    assert!("unknown".parse::<RunState>().is_err());
    assert!(Run::new(" \n ".into(), 0).is_err());
}

#[test]
fn reviews_use_status_and_severity_instead_of_prose() {
    for status in [ReviewStatus::Pass, ReviewStatus::Warn, ReviewStatus::Fail] {
        let mut review = ReviewResult {
            reviewer: "reviewer".into(),
            status,
            findings: vec![],
        };
        assert_eq!(review.is_blocking(), status == ReviewStatus::Fail);
        for severity in [
            Severity::Blocker,
            Severity::High,
            Severity::Medium,
            Severity::Low,
            Severity::Info,
        ] {
            review.findings = vec![Finding {
                severity,
                rule: "sample-rule".into(),
                file: None,
                line: None,
                message: "Any prose, including a positive verdict".into(),
                suggested_fix: None,
            }];
            assert_eq!(
                review.is_blocking(),
                status == ReviewStatus::Fail
                    || matches!(severity, Severity::Blocker | Severity::High)
            );
            let restored: ReviewResult =
                serde_json::from_value(serde_json::to_value(&review).unwrap()).unwrap();
            assert_eq!(restored, review);
        }
    }
}

#[test]
fn artifacts_validate_names_kinds_and_review_payloads() {
    for name in [
        "../plan.json",
        "a/plan.json",
        "a\\plan.json",
        "/plan.json",
        "C:\\plan.json",
        "",
        ".json",
        "..",
        "a%2fb.json",
        "x\0.json",
    ] {
        assert!(
            ArtifactKind::Review.validate_name(name).is_err(),
            "{name:?}"
        );
    }
    assert!(ArtifactKind::Plan.validate_name("task.json").is_err());
    assert!(ArtifactKind::Review
        .validate_name("security-review_1.json")
        .is_ok());
    assert!(ArtifactKind::Review
        .validate_payload(&json!({"verdict": "pass"}))
        .is_err());
    assert!(ArtifactKind::Review
        .validate_payload(&json!({"reviewer": "sample", "status": "pass", "findings": []}))
        .is_ok());
}

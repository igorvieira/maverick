use maverick_core::{Artifact, ArtifactKind, RunState};
use maverick_store::{Store, StoreError};
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn resume_run_history_and_versioned_artifacts() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Sample task".into()).unwrap();
    assert!(store.history(run.id()).unwrap().is_empty());
    let run = store.transition(&run, RunState::ContextResolved).unwrap();
    let first_history = store.history(run.id()).unwrap();
    let run = store.transition(&run, RunState::Planned).unwrap();
    assert_eq!(&store.history(run.id()).unwrap()[..1], first_history);
    let one = store
        .put_artifact(
            run.id(),
            ArtifactKind::Plan,
            "plan.json",
            json!({"steps": ["first"]}),
        )
        .unwrap();
    let two = store
        .put_artifact(
            run.id(),
            ArtifactKind::Plan,
            "plan.json",
            json!({"steps": ["second"]}),
        )
        .unwrap();
    assert_eq!((one.revision, two.revision), (1, 2));
    let before = store.history(run.id()).unwrap();
    drop(store);
    let store = Store::open(dir.path()).unwrap();
    assert_eq!(store.get(run.id()).unwrap(), run);
    assert_eq!(store.history(run.id()).unwrap(), before);
    assert_eq!(
        store
            .artifact(run.id(), ArtifactKind::Plan, "plan.json", 1)
            .unwrap(),
        one
    );
    assert_eq!(
        store
            .artifact(run.id(), ArtifactKind::Plan, "plan.json", 2)
            .unwrap(),
        two
    );
    let folder = dir.path().join("runs").join(run.id().to_string());
    let latest: Artifact =
        serde_json::from_slice(&fs::read(folder.join("plan.json")).unwrap()).unwrap();
    assert_eq!(latest, two);
    assert!(folder.join("task.json").is_file());
    assert!(folder.join("reviews").is_dir());
    assert!(store
        .artifact(run.id(), ArtifactKind::Plan, "plan.json", 3)
        .is_err());
}

#[test]
fn invalid_transition_changes_neither_state_nor_history() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    assert!(matches!(
        store.transition(&run, RunState::Completed),
        Err(StoreError::Run(_))
    ));
    assert_eq!(store.get(run.id()).unwrap(), run);
    assert!(store.history(run.id()).unwrap().is_empty());
    let failed = store.transition(&run, RunState::Failed).unwrap();
    assert!(store
        .transition(&failed, RunState::ContextResolved)
        .is_err());
    assert_eq!(store.history(run.id()).unwrap().len(), 1);
}

#[test]
fn history_is_append_only_even_via_sql_and_failed_insert_rolls_back_state() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    let run = store.transition(&run, RunState::ContextResolved).unwrap();
    let sql = Connection::open(dir.path().join("maverick.db")).unwrap();
    assert!(sql.execute("DELETE FROM transitions", []).is_err());
    assert!(sql
        .execute("UPDATE transitions SET to_state = 'completed'", [])
        .is_err());
    let before = store.history(run.id()).unwrap();
    sql.execute_batch("CREATE TRIGGER simulate_disk_error BEFORE INSERT ON transitions BEGIN SELECT RAISE(ABORT, 'simulated failure'); END;").unwrap();
    assert!(store.transition(&run, RunState::Planned).is_err());
    assert_eq!(store.get(run.id()).unwrap(), run);
    assert_eq!(store.history(run.id()).unwrap(), before);
}

#[test]
fn stale_writers_and_aba_loop_are_rejected() {
    let dir = tempdir().unwrap();
    let mut first = Store::open(dir.path()).unwrap();
    let mut run = first.start("Task".into()).unwrap();
    let mut second = Store::open(dir.path()).unwrap();
    let stale = second.get(run.id()).unwrap();
    run = first.transition(&run, RunState::ContextResolved).unwrap();
    assert!(matches!(
        second.transition(&stale, RunState::Failed),
        Err(StoreError::Conflict(_))
    ));
    for state in [
        RunState::Planned,
        RunState::AwaitingApproval,
        RunState::Implementing,
        RunState::Reviewing,
    ] {
        run = first.transition(&run, state).unwrap();
    }
    let stale = second.get(run.id()).unwrap();
    run = first.transition(&run, RunState::Fixing).unwrap();
    run = first.transition(&run, RunState::Reviewing).unwrap();
    assert!(matches!(
        second.transition(&stale, RunState::Testing),
        Err(StoreError::Conflict(_))
    ));
    assert_eq!(second.get(run.id()).unwrap(), run);
    assert_eq!(second.history(run.id()).unwrap().len(), 7);
}

#[test]
fn simultaneous_writers_have_one_winner() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let root = dir.path().to_owned();
            let run = run.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                let mut store = Store::open(root).unwrap();
                barrier.wait();
                store.transition(&run, RunState::ContextResolved)
            })
        })
        .collect();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|r| matches!(r, Err(StoreError::Conflict(_))))
            .count(),
        1
    );
    assert_eq!(store.history(run.id()).unwrap().len(), 1);
}

#[test]
fn traversal_and_wrong_review_contract_do_not_write_artifacts() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    for name in [
        "../escape.json",
        "../../escape.json",
        "/tmp/escape.json",
        "..\\escape.json",
        "reviews/a.json",
    ] {
        assert!(matches!(
            store.put_artifact(run.id(), ArtifactKind::Review, name, json!({})),
            Err(StoreError::Artifact(_))
        ));
    }
    assert!(store
        .put_artifact(
            run.id(),
            ArtifactKind::Review,
            "review.json",
            json!({"status": "Fail"})
        )
        .is_err());
    assert!(!dir.path().join("escape.json").exists());
    let review = json!({"reviewer": "sample", "status": "fail", "findings": []});
    let saved = store
        .put_artifact(run.id(), ArtifactKind::Review, "review.json", review)
        .unwrap();
    assert_eq!(saved.revision, 1);
    assert!(dir
        .path()
        .join("runs")
        .join(run.id().to_string())
        .join("reviews/review.json")
        .is_file());
}

#[test]
fn reopen_repairs_missing_or_stale_latest_view_without_losing_revisions() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    let first = store
        .put_artifact(run.id(), ArtifactKind::Plan, "plan.json", json!(1))
        .unwrap();
    let latest = store
        .put_artifact(run.id(), ArtifactKind::Plan, "plan.json", json!(2))
        .unwrap();
    let file = dir
        .path()
        .join("runs")
        .join(run.id().to_string())
        .join("plan.json");
    drop(store);
    for stale in [None, Some(serde_json::to_vec(&first).unwrap())] {
        if let Some(bytes) = stale {
            fs::write(&file, bytes).unwrap();
        } else {
            fs::remove_file(&file).unwrap();
        }
        let store = Store::open(dir.path()).unwrap();
        let repaired: Artifact = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        assert_eq!(repaired, latest);
        assert_eq!(
            store
                .artifact(run.id(), ArtifactKind::Plan, "plan.json", 1)
                .unwrap(),
            first
        );
    }
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_redirect_storage_writes() {
    use std::os::unix::fs::symlink;
    for component in ["reviews", ".versions", "plan.json"] {
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let run = store.start("Task".into()).unwrap();
        let path = dir
            .path()
            .join("runs")
            .join(run.id().to_string())
            .join(component);
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        }
        symlink(outside.path(), &path).unwrap();
        let (kind, name, payload) = if component == "reviews" {
            (
                ArtifactKind::Review,
                "review.json",
                json!({"reviewer": "sample", "status": "pass", "findings": []}),
            )
        } else {
            (ArtifactKind::Plan, "plan.json", json!({}))
        };
        assert!(matches!(
            store.put_artifact(run.id(), kind, name, payload),
            Err(StoreError::UnsafePath(_))
        ));
        assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    }
    for name in ["runs", "maverick.db", "maverick.db-journal"] {
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        symlink(outside.path().join("missing"), dir.path().join(name)).unwrap();
        assert!(matches!(
            Store::open(dir.path()),
            Err(StoreError::UnsafePath(_))
        ));
    }
}

#[test]
fn uncommitted_artifact_is_ignored_and_its_revision_can_be_retried() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    let first = store
        .put_artifact(run.id(), ArtifactKind::Plan, "plan.json", json!("first"))
        .unwrap();
    let sql = Connection::open(dir.path().join("maverick.db")).unwrap();
    sql.execute_batch("CREATE TRIGGER fail_artifact BEFORE INSERT ON artifacts BEGIN SELECT RAISE(ABORT, 'simulated failure'); END;").unwrap();
    assert!(store
        .put_artifact(
            run.id(),
            ArtifactKind::Plan,
            "plan.json",
            json!("uncommitted")
        )
        .is_err());
    drop(store);
    let mut store = Store::open(dir.path()).unwrap();
    assert_eq!(
        store
            .artifact(run.id(), ArtifactKind::Plan, "plan.json", 1)
            .unwrap(),
        first
    );
    assert!(matches!(
        store.artifact(run.id(), ArtifactKind::Plan, "plan.json", 2),
        Err(StoreError::ArtifactNotFound { .. })
    ));
    let latest_path = dir
        .path()
        .join("runs")
        .join(run.id().to_string())
        .join("plan.json");
    let latest: Artifact = serde_json::from_slice(&fs::read(&latest_path).unwrap()).unwrap();
    assert_eq!(latest, first);
    sql.execute_batch("DROP TRIGGER fail_artifact;").unwrap();
    let next = store
        .put_artifact(run.id(), ArtifactKind::Plan, "plan.json", json!("retry"))
        .unwrap();
    assert_eq!(next.revision, 2);
    assert_eq!(
        store
            .artifact(run.id(), ArtifactKind::Plan, "plan.json", 2)
            .unwrap()
            .payload,
        json!("retry")
    );
}

#[test]
fn publication_failure_reports_committed_run_and_recovers_after_obstruction_is_removed() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    let latest_path = dir
        .path()
        .join("runs")
        .join(run.id().to_string())
        .join("plan.json");
    fs::create_dir(&latest_path).unwrap();
    let result = store.put_artifact(
        run.id(),
        ArtifactKind::Plan,
        "plan.json",
        json!("committed"),
    );
    assert!(
        matches!(result, Err(StoreError::PublicationPending { run_id, .. }) if run_id == run.id())
    );
    assert_eq!(
        store
            .artifact(run.id(), ArtifactKind::Plan, "plan.json", 1)
            .unwrap()
            .payload,
        json!("committed")
    );
    drop(store);
    fs::remove_dir(&latest_path).unwrap();
    let _store = Store::open(dir.path()).unwrap();
    let latest: Artifact = serde_json::from_slice(&fs::read(latest_path).unwrap()).unwrap();
    assert_eq!(latest.payload, json!("committed"));
}

#[test]
fn unsupported_schema_and_mismatched_artifact_metadata_fail_explicitly() {
    let dir = tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let run = store.start("Task".into()).unwrap();
    let artifact_path = dir
        .path()
        .join("runs")
        .join(run.id().to_string())
        .join(".versions/task.json/1.json");
    let mut artifact: serde_json::Value =
        serde_json::from_slice(&fs::read(&artifact_path).unwrap()).unwrap();
    artifact["schema_version"] = json!(99);
    fs::write(artifact_path, serde_json::to_vec(&artifact).unwrap()).unwrap();
    assert!(matches!(
        store.artifact(run.id(), ArtifactKind::Task, "task.json", 1),
        Err(StoreError::CorruptArtifact(_))
    ));
    drop(store);
    assert!(matches!(
        Store::open(dir.path()),
        Err(StoreError::CorruptArtifact(_))
    ));
    let sql = Connection::open(dir.path().join("maverick.db")).unwrap();
    sql.execute_batch("PRAGMA user_version = 99;").unwrap();
    assert!(matches!(
        Store::open(dir.path()),
        Err(StoreError::SchemaVersion(99))
    ));
}

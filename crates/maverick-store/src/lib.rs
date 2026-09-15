//! Local SQLite index and inspectable JSON artifacts. No external services.
mod files;

use maverick_core::{Artifact, ArtifactError, ArtifactKind, Run, RunError, RunId, RunState};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Run(#[from] RunError),
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    #[error("run {0} not found")]
    NotFound(RunId),
    #[error("run {0} changed concurrently; reload status and retry")]
    Conflict(RunId),
    #[error("unsafe storage path: {0}")]
    UnsafePath(PathBuf),
    #[error("unsupported database schema version: {0}")]
    SchemaVersion(u32),
    #[error("artifact index and file disagree: {0}")]
    CorruptArtifact(PathBuf),
    #[error("artifact {name} revision {revision} not found for run {run_id}")]
    ArtifactNotFound {
        run_id: RunId,
        name: String,
        revision: u64,
    },
    #[error("integer outside SQLite range: {0}")]
    IntegerRange(#[from] std::num::TryFromIntError),
    #[error("system clock is before the Unix epoch")]
    Clock,
    #[error("run {run_id} was committed, but artifact views could not be published; reopen the store to repair them: {source}")]
    PublicationPending {
        run_id: RunId,
        #[source]
        source: Box<StoreError>,
    },
}

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub run_id: RunId,
    pub revision: u64,
    pub from: RunState,
    pub to: RunState,
    pub occurred_at: u64,
}

pub struct Store {
    connection: Connection,
    root: PathBuf,
}

fn now() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| StoreError::Clock)
}

impl Store {
    /// `root` is the storage directory itself (usually `.maverick`).
    /// Use a directory owned by the caller; hostile concurrent filesystem edits
    /// are outside this local store's trust boundary.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        std::fs::create_dir_all(root.as_ref())?;
        let root = root.as_ref().canonicalize()?;
        files::directory(&root.join("runs"))?;
        for name in [
            "maverick.db",
            "maverick.db-journal",
            "maverick.db-wal",
            "maverick.db-shm",
        ] {
            files::reject_symlink(&root.join(name))?;
        }
        let mut connection = Connection::open(root.join("maverick.db"))?;
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = FULL;")?;
        let tx = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: u32 = tx.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version > 1 {
            return Err(StoreError::SchemaVersion(version));
        }
        tx.execute_batch(include_str!("schema.sql"))?;
        // Immutable revision files are authoritative. Repair the latest views if
        // a process stopped after committing the index but before publishing them.
        files::recover(&tx, &root)?;
        tx.commit()?;
        Ok(Self { connection, root })
    }

    pub fn start(&mut self, task: String) -> Result<Run> {
        let run = Run::new(task, now()?)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO runs (id, task, state, revision, created_at) VALUES (?1, ?2, ?3, 0, ?4)",
            params![
                run.id().to_string(),
                run.task(),
                run.state().to_string(),
                sql_int(run.created_at())?
            ],
        )?;
        files::write_artifact(
            &tx,
            &self.root,
            run.id(),
            ArtifactKind::Task,
            "task.json",
            json!({"description": run.task()}),
        )?;
        files::directory(
            &self
                .root
                .join("runs")
                .join(run.id().to_string())
                .join("reviews"),
        )?;
        tx.commit()?;
        // Opening a new process repairs this view if publication fails.
        self.publish_committed(run.id())?;
        Ok(run)
    }

    pub fn get(&self, id: RunId) -> Result<Run> {
        load_run(&self.connection, id)
    }

    /// Compare both previous state and revision, including ABA changes through
    /// Reviewing -> Fixing -> Reviewing. The caller's snapshot is never mutated.
    pub fn transition(&mut self, previous: &Run, to: RunState) -> Result<Run> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut current = load_run(&tx, previous.id())?;
        if current.state() != previous.state() || current.revision() != previous.revision() {
            return Err(StoreError::Conflict(previous.id()));
        }
        current.transition(to)?;
        let changed = tx.execute(
            "UPDATE runs SET state = ?1, revision = ?2 WHERE id = ?3 AND state = ?4 AND revision = ?5",
            params![to.to_string(), sql_int(current.revision())?, previous.id().to_string(), previous.state().to_string(), sql_int(previous.revision())?])?;
        if changed != 1 {
            return Err(StoreError::Conflict(previous.id()));
        }
        tx.execute("INSERT INTO transitions (run_id, revision, from_state, to_state, occurred_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![previous.id().to_string(), sql_int(current.revision())?, previous.state().to_string(), to.to_string(), sql_int(now()?)?])?;
        tx.commit()?;
        Ok(current)
    }

    pub fn history(&self, id: RunId) -> Result<Vec<Transition>> {
        self.get(id)?;
        let mut stmt = self.connection.prepare(
            "SELECT revision, from_state, to_state, occurred_at FROM transitions WHERE run_id = ?1 ORDER BY revision")?;
        let rows = stmt.query_map([id.to_string()], |r| {
            Ok((
                read_u64(r, 0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                read_u64(r, 3)?,
            ))
        })?;
        rows.map(|row| {
            let (revision, from, to, occurred_at) = row?;
            Ok(Transition {
                run_id: id,
                revision,
                from: from.parse()?,
                to: to.parse()?,
                occurred_at,
            })
        })
        .collect()
    }

    pub fn put_artifact(
        &mut self,
        id: RunId,
        kind: ArtifactKind,
        name: &str,
        payload: Value,
    ) -> Result<Artifact> {
        kind.validate_name(name)?;
        kind.validate_payload(&payload)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        load_run(&tx, id)?;
        let artifact = files::write_artifact(&tx, &self.root, id, kind, name, payload)?;
        tx.commit()?;
        // Serialize latest-view publication with all other writers.
        self.publish_committed(id)?;
        Ok(artifact)
    }

    fn publish_committed(&mut self, run_id: RunId) -> Result<()> {
        let mut publish = || -> Result<()> {
            let tx = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            files::recover(&tx, &self.root)?;
            tx.commit()?;
            Ok(())
        };
        publish().map_err(|source| StoreError::PublicationPending {
            run_id,
            source: Box::new(source),
        })
    }

    pub fn artifact(
        &self,
        id: RunId,
        kind: ArtifactKind,
        name: &str,
        revision: u64,
    ) -> Result<Artifact> {
        kind.validate_name(name)?;
        self.get(id)?;
        let exists: bool = self.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM artifacts WHERE run_id = ?1 AND kind = ?2 AND name = ?3 AND revision = ?4)",
            params![id.to_string(), serde_json::to_string(&kind)?, name, sql_int(revision)?], |r| r.get(0))?;
        if !exists {
            return Err(StoreError::ArtifactNotFound {
                run_id: id,
                name: name.into(),
                revision,
            });
        }
        files::read_artifact(&self.root, id, kind, name, revision)
    }
}

fn load_run(connection: &Connection, id: RunId) -> Result<Run> {
    let row = connection
        .query_row(
            "SELECT task, state, revision, created_at FROM runs WHERE id = ?1",
            [id.to_string()],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    read_u64(r, 2)?,
                    read_u64(r, 3)?,
                ))
            },
        )
        .optional()?
        .ok_or(StoreError::NotFound(id))?;
    Ok(serde_json::from_value(
        json!({"id": id, "task": row.0, "state": row.1, "revision": row.2, "created_at": row.3}),
    )?)
}

fn sql_int(value: u64) -> Result<i64> {
    Ok(i64::try_from(value)?)
}

fn read_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

use crate::{read_u64, sql_int, Result, StoreError};
use maverick_core::{Artifact, ArtifactKind, RunId, ARTIFACT_SCHEMA_VERSION};
use rusqlite::{params, Connection};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

pub(crate) fn reject_symlink(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(StoreError::UnsafePath(path.into())),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn directory(path: &Path) -> Result<()> {
    reject_symlink(path)?;
    match fs::create_dir(path) {
        Ok(()) =>
        {
            #[cfg(unix)]
            if let Some(parent) = path.parent() {
                fs::File::open(parent)?.sync_all()?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && path.is_dir() => (),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

fn artifact_directory(root: &Path, id: RunId, kind: ArtifactKind, name: &str) -> Result<PathBuf> {
    kind.validate_name(name)?;
    let mut path = root.join("runs");
    directory(&path)?;
    path.push(id.to_string());
    directory(&path)?;
    if kind == ArtifactKind::Review {
        path.push("reviews");
        directory(&path)?;
    }
    Ok(path)
}

fn revision_path(
    root: &Path,
    id: RunId,
    kind: ArtifactKind,
    name: &str,
    revision: u64,
) -> Result<PathBuf> {
    let mut path = artifact_directory(root, id, kind, name)?;
    path.push(".versions");
    directory(&path)?;
    path.push(name);
    directory(&path)?;
    path.push(format!("{revision}.json"));
    reject_symlink(&path)?;
    Ok(path)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    reject_symlink(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| StoreError::UnsafePath(path.into()))?;
    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| e.error)?;
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

pub(crate) fn write_artifact(
    connection: &Connection,
    root: &Path,
    id: RunId,
    kind: ArtifactKind,
    name: &str,
    payload: Value,
) -> Result<Artifact> {
    kind.validate_name(name)?;
    kind.validate_payload(&payload)?;
    let encoded_kind = serde_json::to_string(&kind)?;
    let revision: u64 = connection.query_row(
        "SELECT COALESCE(MAX(revision), 0) + 1 FROM artifacts WHERE run_id = ?1 AND kind = ?2 AND name = ?3",
        params![id.to_string(), encoded_kind, name], |r| read_u64(r, 0))?;
    let latest = artifact_directory(root, id, kind, name)?.join(name);
    reject_symlink(&latest)?;
    let artifact = Artifact {
        schema_version: ARTIFACT_SCHEMA_VERSION,
        run_id: id,
        kind,
        name: name.into(),
        revision,
        payload,
    };
    let path = revision_path(root, id, kind, name, revision)?;
    // This revision is not indexed yet. A leftover from an interrupted transaction
    // can be replaced; indexed revisions are never written again.
    atomic_write(&path, &serde_json::to_vec_pretty(&artifact)?)?;
    connection.execute(
        "INSERT INTO artifacts (run_id, kind, name, revision) VALUES (?1, ?2, ?3, ?4)",
        params![id.to_string(), encoded_kind, name, sql_int(revision)?],
    )?;
    Ok(artifact)
}

pub(crate) fn read_artifact(
    root: &Path,
    id: RunId,
    kind: ArtifactKind,
    name: &str,
    revision: u64,
) -> Result<Artifact> {
    let path = revision_path(root, id, kind, name, revision)?;
    let artifact: Artifact = serde_json::from_slice(&fs::read(&path)?)?;
    if artifact.schema_version != ARTIFACT_SCHEMA_VERSION
        || artifact.run_id != id
        || artifact.kind != kind
        || artifact.name != name
        || artifact.revision != revision
    {
        return Err(StoreError::CorruptArtifact(path));
    }
    kind.validate_payload(&artifact.payload)?;
    Ok(artifact)
}

/// Call under an immediate transaction so no concurrent writer can publish an
/// older latest view after a newer commit.
pub(crate) fn recover(connection: &Connection, root: &Path) -> Result<()> {
    let mut stmt = connection.prepare(
        "SELECT run_id, kind, name, MAX(revision) FROM artifacts GROUP BY run_id, kind, name",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            read_u64(r, 3)?,
        ))
    })?;
    for row in rows {
        let (id, kind, name, revision) = row?;
        let id: RunId = serde_json::from_value(Value::String(id))?;
        let kind: ArtifactKind = serde_json::from_str(&kind)?;
        let artifact = read_artifact(root, id, kind, &name, revision)?;
        let latest = artifact_directory(root, id, kind, &name)?.join(&name);
        reject_symlink(&latest)?;
        let bytes = serde_json::to_vec_pretty(&artifact)?;
        match fs::read(&latest) {
            Ok(existing) if existing == bytes => (),
            Ok(_) => atomic_write(&latest, &bytes)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => atomic_write(&latest, &bytes)?,
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

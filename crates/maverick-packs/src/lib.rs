//! Declarative pack loading and file detection. No command or agent execution.
use maverick_core::pack::{PackError, PackManifest, ProjectAdapter};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid manifest {path}: {source}")]
    Manifest {
        path: PathBuf,
        #[source]
        source: PackError,
    },
    #[error("agent {agent} is missing, empty, or outside the agent directory: {path}")]
    MissingAgent { agent: String, path: PathBuf },
    #[error(transparent)]
    Contract(#[from] PackError),
}

fn read(path: &Path) -> Result<String, LoadError> {
    fs::read_to_string(path).map_err(|source| LoadError::Io {
        path: path.into(),
        source,
    })
}

/// Source packs keep prompts beside pack.json; installed manifests can use the
/// shared flat agent directory explicitly. All references are checked before routing.
pub fn load_pack(manifest: &Path, agents: &Path) -> Result<PackManifest, LoadError> {
    let pack = PackManifest::from_json(&read(manifest)?).map_err(|source| LoadError::Manifest {
        path: manifest.into(),
        source,
    })?;
    let root = agents.canonicalize().map_err(|source| LoadError::Io {
        path: agents.into(),
        source,
    })?;
    for agent in pack.agent_references() {
        let path = root.join(format!("{agent}.md"));
        let checked = path
            .canonicalize()
            .ok()
            .filter(|p| p.starts_with(&root) && p.is_file());
        if checked
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .is_none_or(|s| s.trim().is_empty())
        {
            return Err(LoadError::MissingAgent {
                agent: agent.into(),
                path,
            });
        }
    }
    Ok(pack)
}

/// Duplicate pack IDs are rejected even when their manifests have different paths.
pub fn load_packs(sources: &[(PathBuf, PathBuf)]) -> Result<Vec<PackManifest>, LoadError> {
    let mut packs = vec![];
    let mut ids = HashSet::new();
    for (manifest, agents) in sources {
        let pack = load_pack(manifest, agents)?;
        if !ids.insert(pack.id.clone()) {
            return Err(PackError::Duplicate {
                field: "pack id".into(),
                id: pack.id,
            }
            .into());
        }
        packs.push(pack);
    }
    Ok(packs)
}

pub fn load_adapter(path: &Path) -> Result<ProjectAdapter, LoadError> {
    ProjectAdapter::from_json(&read(path)?).map_err(|source| LoadError::Manifest {
        path: path.into(),
        source,
    })
}

pub fn detect_pack(pack: &PackManifest, project: &Path) -> Result<bool, LoadError> {
    pack.validate()?;
    let root = project.canonicalize().map_err(|source| LoadError::Io {
        path: project.into(),
        source,
    })?;
    let mut files = HashSet::new();
    for name in pack.detection.files.iter().chain(&pack.detection.any_files) {
        let path = root.join(name);
        match path.canonicalize() {
            Ok(path) if path.starts_with(&root) && path.is_file() => {
                files.insert(name.clone());
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(source) => return Err(LoadError::Io { path, source }),
        }
    }
    Ok(pack.detection.matches(&files))
}

/// Explicit adapter selection wins over detection. Output follows adapter order,
/// or the caller's manifest order when detecting. Unknown explicit packs fail.
pub fn select_packs<'a>(
    packs: &'a [PackManifest],
    adapter: Option<&ProjectAdapter>,
    project: &Path,
) -> Result<Vec<&'a PackManifest>, LoadError> {
    let mut ids = HashSet::new();
    for pack in packs {
        pack.validate()?;
        if !ids.insert(&pack.id) {
            return Err(PackError::Duplicate {
                field: "pack id".into(),
                id: pack.id.clone(),
            }
            .into());
        }
    }
    if let Some(adapter) = adapter {
        adapter.validate()?;
        return adapter
            .packs
            .iter()
            .map(|id| {
                packs
                    .iter()
                    .find(|p| &p.id == id)
                    .ok_or_else(|| PackError::UnknownPack(id.clone()).into())
            })
            .collect();
    }
    let mut selected = vec![];
    for pack in packs {
        if detect_pack(pack, project)? {
            selected.push(pack);
        }
    }
    Ok(selected)
}

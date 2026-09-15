use anyhow::{Context, Result};
use maverick_core::ModelSetup;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub user: PathBuf,
    pub project: Option<PathBuf>,
}

impl ConfigPaths {
    pub fn resolve(config: Option<PathBuf>, project: Option<PathBuf>, root: PathBuf) -> Self {
        let user = config.unwrap_or_else(default_user_config);
        let project = project.or_else(|| {
            let candidate = root.join("config.toml");
            candidate.exists().then_some(candidate)
        });
        Self { user, project }
    }
}

fn default_user_config() -> PathBuf {
    if let Ok(path) = env::var("MAVERICK_CONFIG") {
        return PathBuf::from(path);
    }
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("maverick/config.toml");
    }
    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".config/maverick/config.toml"))
        .unwrap_or_else(|| PathBuf::from("maverick-config.toml"))
}

pub fn load_setup(path: &Path) -> Result<ModelSetup> {
    if !path.exists() {
        return Ok(ModelSetup::default());
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    ModelSetup::from_toml(&text).with_context(|| format!("invalid model setup {}", path.display()))
}

pub fn save_setup(path: &Path, setup: &ModelSetup) -> Result<()> {
    setup.validate()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    fs::write(path, setup.to_toml()?).with_context(|| format!("cannot write {}", path.display()))
}

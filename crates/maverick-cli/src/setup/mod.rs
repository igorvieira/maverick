mod app;
mod config;
mod discover;
mod tui;

use config::load_setup;
pub use config::ConfigPaths;
use discover::{discover, SystemProbe};

use anyhow::{Context, Result};
use maverick_core::ModelSetup;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};

pub fn run_tui(paths: ConfigPaths) -> Result<()> {
    tui::run(paths)
}

pub fn run_entry(paths: ConfigPaths) -> Result<()> {
    if io::stdout().is_terminal() {
        return run_tui(paths);
    }
    eprintln!("interactive setup requires a TTY; use `maverick models` or `maverick --help`");
    std::process::exit(2);
}

pub fn print_models(paths: ConfigPaths) -> Result<()> {
    let setup = resolve(&paths)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "execution": "disabled",
            "setup": setup,
        }))?
    );
    Ok(())
}

pub fn resolve(paths: &ConfigPaths) -> Result<ModelSetup> {
    let user = load_setup(&paths.user).context("invalid user model config")?;
    let project = match &paths.project {
        Some(path) if path.exists() => {
            Some(load_setup(path).context("invalid project model config")?)
        }
        _ => None,
    };
    let discovered = discover(&SystemProbe)?;
    let mut setup = user.with_discovery(discovered)?;
    if let Some(project) = project {
        setup = setup.merge(&project)?;
    }
    Ok(setup)
}

pub fn paths_from(config: Option<PathBuf>, project: Option<PathBuf>, root: PathBuf) -> ConfigPaths {
    ConfigPaths::resolve(config, project, root)
}

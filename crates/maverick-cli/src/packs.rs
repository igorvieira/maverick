use anyhow::{Context, Result};
use clap::Args;
use maverick_core::pack::{Capability, ChangeClassification};
use maverick_packs::{load_adapter, load_packs, select_packs};
use serde_json::json;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Args)]
pub struct PackArgs {
    /// Structured manifest; repeat to load multiple packs in declaration order.
    #[arg(long, required = true)]
    manifest: Vec<PathBuf>,
    /// Shared prompt directory for installed manifests; defaults to each manifest's parent.
    #[arg(long)]
    agents_dir: Option<PathBuf>,
    /// Extensible snake_case capability; repeat for multiple capabilities.
    #[arg(long)]
    capability: Vec<Capability>,
    /// JSON object with a capabilities array (merged with --capability).
    #[arg(long)]
    classification: Option<PathBuf>,
    /// Detect applicable packs from files in this project; otherwise inspect all supplied packs.
    #[arg(long)]
    project: Option<PathBuf>,
    /// Structured project adapter; explicit selection overrides detection.
    #[arg(long)]
    adapter: Option<PathBuf>,
}

pub fn run(args: PackArgs) -> Result<()> {
    let sources: Vec<_> = args
        .manifest
        .iter()
        .map(|path| {
            let agents = args
                .agents_dir
                .clone()
                .unwrap_or_else(|| path.parent().unwrap_or(Path::new(".")).into());
            (path.clone(), agents)
        })
        .collect();
    let packs = load_packs(&sources)?;
    let adapter = args.adapter.as_deref().map(load_adapter).transpose()?;
    let mut capabilities: HashSet<_> = args.capability.into_iter().collect();
    if let Some(path) = args.classification {
        let classification: ChangeClassification = serde_json::from_str(
            &std::fs::read_to_string(&path)
                .with_context(|| format!("cannot read classification {}", path.display()))?,
        )
        .with_context(|| format!("invalid classification {}", path.display()))?;
        capabilities.extend(classification.capabilities);
    }
    let selected = if args.project.is_some() || adapter.is_some() {
        select_packs(
            &packs,
            adapter.as_ref(),
            args.project.as_deref().unwrap_or(Path::new(".")),
        )?
    } else {
        packs.iter().collect()
    };
    let output: Result<Vec<_>> = selected
        .into_iter()
        .map(|pack| {
            let commands = match &adapter {
                Some(adapter) => adapter.resolve_commands(pack)?,
                None => pack.commands.clone(),
            };
            Ok(json!({
                "id": pack.id,
                "display_name": pack.display_name,
                "commands": commands,
                "agents": pack.agents,
                "reviewers": pack.select_reviewers(&capabilities)?,
                "red_team_required": pack.requires_red_team(&capabilities)?,
                "review_policy": pack.review_policy,
            }))
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "execution": "disabled",
            "packs": output?,
            "project_commands": adapter.as_ref().map(|a| &a.project_commands),
        }))?
    );
    Ok(())
}

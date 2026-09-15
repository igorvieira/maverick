mod packs;
mod setup;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use maverick_core::{ArtifactKind, RunId, RunState};
use maverick_store::Store;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "maverick",
    version,
    about = "Experimental local execution kernel and model setup TUI"
)]
struct Cli {
    /// Storage directory (contains maverick.db and runs/).
    #[arg(long, global = true, default_value = ".maverick")]
    root: PathBuf,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Interactive Ratatui setup for local and Grok models.
    Setup,
    /// Print the resolved model setup as JSON; never executes providers.
    Models {
        /// User config TOML (defaults to $MAVERICK_CONFIG or ~/.config/maverick/config.toml).
        #[arg(long)]
        config: Option<PathBuf>,
        /// Optional project overlay TOML (defaults to <root>/config.toml when present).
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// Load and route declarative packs; never executes commands or creates run storage.
    Pack(packs::PackArgs),
    /// Create a run and its task artifact. Prints the run as JSON.
    Start {
        #[arg(long)]
        task: String,
    },
    /// Read the persisted run as JSON.
    Status { run_id: RunId },
    /// Validate and persist one lifecycle transition.
    Transition {
        run_id: RunId,
        #[arg(long)]
        to: RunState,
    },
    /// Save a versioned JSON artifact (review payloads must be structured).
    Artifact {
        run_id: RunId,
        #[arg(long)]
        kind: ArtifactKind,
        #[arg(long)]
        name: String,
        #[arg(long)]
        file: PathBuf,
    },
    /// Read the append-only transition history as JSON.
    History { run_id: RunId },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => {
            return setup::run_entry(setup::paths_from(None, None, cli.root));
        }
        Some(Command::Setup) => {
            return setup::run_tui(setup::paths_from(None, None, cli.root));
        }
        Some(Command::Models { config, project }) => {
            return setup::print_models(setup::paths_from(config, project, cli.root));
        }
        Some(Command::Pack(args)) => return packs::run(args),
        Some(_) => {}
    }
    // Validate input before opening/creating storage.
    let payload = if let Some(Command::Artifact {
        kind, name, file, ..
    }) = &cli.command
    {
        kind.validate_name(name)?;
        let bytes =
            std::fs::read(file).with_context(|| format!("cannot read {}", file.display()))?;
        let value = serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid JSON in {}", file.display()))?;
        kind.validate_payload(&value)?;
        Some(value)
    } else {
        None
    };
    let mut store = Store::open(&cli.root)
        .with_context(|| format!("cannot open store at {}", cli.root.display()))?;
    let output = match cli.command {
        None | Some(Command::Setup) | Some(Command::Models { .. }) | Some(Command::Pack(_)) => {
            unreachable!("setup and pack commands return before opening the store")
        }
        Some(Command::Start { task }) => serde_json::to_value(store.start(task)?)?,
        Some(Command::Status { run_id }) => serde_json::to_value(store.get(run_id)?)?,
        Some(Command::Transition { run_id, to }) => {
            let previous = store.get(run_id)?;
            serde_json::to_value(store.transition(&previous, to)?)?
        }
        Some(Command::Artifact {
            run_id, kind, name, ..
        }) => serde_json::to_value(store.put_artifact(
            run_id,
            kind,
            &name,
            payload.context("missing artifact payload")?,
        )?)?,
        Some(Command::History { run_id }) => serde_json::to_value(store.history(run_id)?)?,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

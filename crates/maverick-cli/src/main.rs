mod packs;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use maverick_core::{ArtifactKind, RunId, RunState};
use maverick_store::Store;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "maverick",
    version,
    about = "Experimental local execution kernel"
)]
struct Cli {
    /// Storage directory (contains maverick.db and runs/).
    #[arg(long, global = true, default_value = ".maverick")]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
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
    if let Command::Pack(args) = cli.command {
        return packs::run(args);
    }
    // Validate input before opening/creating storage.
    let payload = if let Command::Artifact {
        kind, name, file, ..
    } = &cli.command
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
        Command::Pack(_) => unreachable!("pack commands return before opening the store"),
        Command::Start { task } => serde_json::to_value(store.start(task)?)?,
        Command::Status { run_id } => serde_json::to_value(store.get(run_id)?)?,
        Command::Transition { run_id, to } => {
            let previous = store.get(run_id)?;
            serde_json::to_value(store.transition(&previous, to)?)?
        }
        Command::Artifact {
            run_id, kind, name, ..
        } => serde_json::to_value(store.put_artifact(
            run_id,
            kind,
            &name,
            payload.context("missing artifact payload")?,
        )?)?,
        Command::History { run_id } => serde_json::to_value(store.history(run_id)?)?,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

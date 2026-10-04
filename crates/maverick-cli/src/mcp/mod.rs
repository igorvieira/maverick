mod app;
mod catalog;
mod detect;
mod install;
mod tui;
mod view;

use crate::setup::discover::SystemProbe;
use anyhow::Result;
use app::{App, Cell};
use detect::{detect, Agent, Paths};
use std::io::{self, IsTerminal};

/// Interactive picker on a TTY; otherwise (or with `--list`) the status matrix as JSON.
pub fn run_entry(list: bool) -> Result<()> {
    if !list && io::stdout().is_terminal() {
        return tui::run();
    }
    print_status()
}

fn print_status() -> Result<()> {
    let entries = catalog::catalog();
    let detection = detect(&SystemProbe, &Paths::current(), &entries);
    let app = App::new(entries, detection);
    let servers: Vec<_> = app
        .catalog
        .iter()
        .enumerate()
        .map(|(row, entry)| {
            let agents: serde_json::Map<_, _> = Agent::ALL
                .into_iter()
                .map(|agent| {
                    let state = match app.cell(row, agent) {
                        Cell::NoCli => "cli-missing".to_string(),
                        Cell::Active(place) => format!("active ({place})"),
                        Cell::Disabled(place) => format!("disabled ({place})"),
                        Cell::Blocked(why) => format!("blocked: {why}"),
                        Cell::Stale(why) => format!("stale: {why}"),
                        Cell::Open | Cell::Marked => "available".to_string(),
                    };
                    (agent.bin().to_string(), state.into())
                })
                .collect();
            serde_json::json!({
                "name": entry.name,
                "summary": entry.summary,
                "scope": entry.scope.as_str(),
                "target": entry.target(),
                "agents": agents,
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&servers)?);
    Ok(())
}

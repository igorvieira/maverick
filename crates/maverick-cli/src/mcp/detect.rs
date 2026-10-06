use super::catalog::Entry;
use crate::setup::discover::Probe;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Agent {
    Claude,
    Codex,
    Grok,
}

impl Agent {
    pub const ALL: [Agent; 3] = [Agent::Claude, Agent::Codex, Agent::Grok];

    pub fn bin(self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Codex => "codex",
            Agent::Grok => "grok",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Agent::Claude => "Claude",
            Agent::Codex => "Codex",
            Agent::Grok => "Grok",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// Loaded by the agent; the string says where it is configured.
    Active(String),
    /// Configured but switched off in the agent.
    Disabled(String),
    /// Present only somewhere the agent never reads (e.g. ~/.claude/settings.json).
    Stale(String),
}

#[derive(Debug, Clone, Default)]
pub struct AgentState {
    pub on_path: bool,
    pub servers: BTreeMap<String, Status>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Detection {
    agents: BTreeMap<Agent, AgentState>,
    /// Catalog entry name → binaries or env vars it needs that are absent.
    pub missing: BTreeMap<String, Vec<String>>,
}

impl Detection {
    pub fn agent(&self, agent: Agent) -> &AgentState {
        static EMPTY: std::sync::OnceLock<AgentState> = std::sync::OnceLock::new();
        self.agents
            .get(&agent)
            .unwrap_or_else(|| EMPTY.get_or_init(AgentState::default))
    }

    pub fn status(&self, agent: Agent, name: &str) -> Option<&Status> {
        self.agent(agent).servers.get(name)
    }

    pub fn missing(&self, name: &str) -> &[String] {
        self.missing.get(name).map(Vec::as_slice).unwrap_or(&[])
    }
}

pub struct Paths {
    pub home: PathBuf,
    pub cwd: PathBuf,
}

impl Paths {
    pub fn current() -> Self {
        Self {
            home: env::var_os("HOME").map(PathBuf::from).unwrap_or_default(),
            cwd: env::current_dir().unwrap_or_default(),
        }
    }
}

pub fn detect(probe: &impl Probe, paths: &Paths, catalog: &[Entry]) -> Detection {
    let mut agents = BTreeMap::new();
    for agent in Agent::ALL {
        let on_path = probe.binary_exists(agent.bin());
        let mut state = AgentState {
            on_path,
            ..Default::default()
        };
        let listed = match agent {
            Agent::Claude => Ok(claude_servers(paths)),
            Agent::Codex | Agent::Grok if on_path => probe
                .output(agent.bin(), &["mcp", "list", "--json"])
                .and_then(|out| parse_listing(&out)),
            Agent::Codex | Agent::Grok => Ok(BTreeMap::new()),
        };
        match listed {
            Ok(servers) => state.servers = servers,
            Err(e) => state.error = Some(e),
        }
        agents.insert(agent, state);
    }

    let missing = catalog
        .iter()
        .map(|entry| {
            let mut needs = vec![];
            if let Some(command) = entry.str_field("command") {
                if !probe.binary_exists(command) {
                    needs.push(command.to_string());
                }
            }
            needs.extend(
                entry
                    .env_refs()
                    .into_iter()
                    .filter(|var| probe.env(var).is_none()),
            );
            (entry.name.clone(), needs)
        })
        .collect();

    Detection { agents, missing }
}

/// Claude Code reads MCP servers from ~/.claude.json (user, and local per project path) and
/// ./.mcp.json (project). It never reads `mcpServers` from ~/.claude/settings.json.
fn claude_servers(paths: &Paths) -> BTreeMap<String, Status> {
    let mut found: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut add = |names: Vec<String>, place: &'static str| {
        for name in names {
            found.entry(name).or_default().push(place);
        }
    };
    let config = read_json(&paths.home.join(".claude.json"));
    add(server_names(config.get("mcpServers")), "user");
    let cwd = paths.cwd.to_string_lossy();
    add(
        server_names(
            config
                .get("projects")
                .and_then(|p| p.get(cwd.as_ref()))
                .and_then(|p| p.get("mcpServers")),
        ),
        "local",
    );
    add(
        server_names(read_json(&paths.cwd.join(".mcp.json")).get("mcpServers")),
        "project",
    );

    let mut servers: BTreeMap<String, Status> = found
        .into_iter()
        .map(|(name, places)| (name, Status::Active(places.join(", "))))
        .collect();
    let settings = read_json(&paths.home.join(".claude/settings.json"));
    for name in server_names(settings.get("mcpServers")) {
        servers
            .entry(name)
            .or_insert_with(|| Status::Stale("only in ~/.claude/settings.json".into()));
    }
    servers
}

/// `codex mcp list --json` and `grok mcp list --json` both return an array of objects with
/// `name`; tolerate an `{"servers": [...]}` wrapper and either `enabled` or `disabled` flags.
fn parse_listing(out: &str) -> Result<BTreeMap<String, Status>, String> {
    let value: Value = serde_json::from_str(out.trim()).map_err(|e| e.to_string())?;
    let items = value
        .as_array()
        .or_else(|| value.get("servers").and_then(Value::as_array))
        .ok_or("unexpected `mcp list --json` output")?;
    Ok(items
        .iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.to_string();
            let disabled = item.get("enabled").and_then(Value::as_bool) == Some(false)
                || item.get("disabled").and_then(Value::as_bool) == Some(true);
            let place = item
                .get("scope")
                .and_then(Value::as_str)
                .unwrap_or("configured")
                .to_string();
            let status = if disabled {
                Status::Disabled(format!("{place}, disabled"))
            } else {
                Status::Active(place)
            };
            Some((name, status))
        })
        .collect())
}

fn read_json(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

fn server_names(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_object)
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::catalog::catalog;
    use crate::setup::discover::MapProbe;
    use std::collections::{HashMap, HashSet};

    fn probe(binaries: &[&str], outputs: &[(&str, &str)], env: &[&str]) -> MapProbe {
        MapProbe {
            binaries: binaries
                .iter()
                .map(|s| s.to_string())
                .collect::<HashSet<_>>(),
            outputs: outputs
                .iter()
                .map(|(bin, out)| {
                    (
                        (
                            bin.to_string(),
                            vec!["mcp".into(), "list".into(), "--json".into()],
                        ),
                        out.to_string(),
                    )
                })
                .collect::<HashMap<_, _>>(),
            env: env.iter().map(|v| (v.to_string(), "x".into())).collect(),
        }
    }

    #[test]
    fn claude_reads_user_local_and_project_and_flags_settings_json_as_stale() {
        let home = tempfile::tempdir().unwrap();
        let cwd = tempfile::tempdir().unwrap();
        let cwd_key = cwd.path().to_string_lossy().to_string();
        fs::write(
            home.path().join(".claude.json"),
            serde_json::json!({
                "mcpServers": {"serena": {}},
                "projects": {cwd_key: {"mcpServers": {"linear": {}}}}
            })
            .to_string(),
        )
        .unwrap();
        fs::create_dir(home.path().join(".claude")).unwrap();
        fs::write(
            home.path().join(".claude/settings.json"),
            r#"{"mcpServers": {"figma": {}, "serena": {}}}"#,
        )
        .unwrap();
        fs::write(
            cwd.path().join(".mcp.json"),
            r#"{"mcpServers": {"serena": {}}}"#,
        )
        .unwrap();
        let paths = Paths {
            home: home.path().into(),
            cwd: cwd.path().into(),
        };

        let det = detect(&probe(&["claude"], &[], &[]), &paths, &catalog());
        assert_eq!(
            det.status(Agent::Claude, "serena"),
            Some(&Status::Active("user, project".into()))
        );
        assert_eq!(
            det.status(Agent::Claude, "linear"),
            Some(&Status::Active("local".into()))
        );
        assert!(matches!(
            det.status(Agent::Claude, "figma"),
            Some(Status::Stale(_))
        ));
        assert!(!det.agent(Agent::Codex).on_path);
    }

    #[test]
    fn codex_and_grok_listings_and_missing_requirements() {
        let empty = tempfile::tempdir().unwrap();
        let paths = Paths {
            home: empty.path().into(),
            cwd: empty.path().into(),
        };
        let p = probe(
            &["codex", "grok", "npx"],
            &[
                (
                    "codex",
                    r#"[{"name":"linear","enabled":true},{"name":"figma","enabled":false}]"#,
                ),
                ("grok", r#"[{"name":"github","scope":"project"}]"#),
            ],
            &[],
        );
        let det = detect(&p, &paths, &catalog());
        assert_eq!(
            det.status(Agent::Codex, "linear"),
            Some(&Status::Active("configured".into()))
        );
        assert!(matches!(
            det.status(Agent::Codex, "figma"),
            Some(Status::Disabled(_))
        ));
        assert_eq!(
            det.status(Agent::Grok, "github"),
            Some(&Status::Active("project".into()))
        );
        assert_eq!(det.missing("github"), ["GITHUB_PAT".to_string()]);
        assert_eq!(det.missing("serena"), ["uvx".to_string()]);
        assert!(det.missing("chrome-devtools").is_empty());
    }

    #[test]
    fn unparseable_listing_is_reported_not_fatal() {
        let paths = Paths {
            home: PathBuf::from("/nonexistent"),
            cwd: PathBuf::from("/nonexistent"),
        };
        let det = detect(
            &probe(&["grok"], &[("grok", "oops")], &[]),
            &paths,
            &catalog(),
        );
        assert!(det.agent(Agent::Grok).error.is_some());
    }
}

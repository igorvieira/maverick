use serde_json::Value;

// The JSON snippets are the single source of truth: setup.sh and the README point at the same files.
const GLOBAL: &str = include_str!("../../../../claude/mcp-servers/global.json");
const PROJECT: &str = include_str!("../../../../claude/mcp-servers/project.json");

const SUMMARIES: &[(&str, &str)] = &[
    ("serena", "Semantic code navigation and symbol-level edits."),
    (
        "figma",
        "Design context, variables and screenshots for design-to-code. OAuth on first use.",
    ),
    (
        "linear",
        "Read tickets and post updates for the Linear workflow. OAuth on first use.",
    ),
    (
        "github",
        "Official remote GitHub server: PRs, issues, repos. Token read from GITHUB_PAT.",
    ),
    (
        "chrome-devtools",
        "Drive Chrome for visual QA, console and network debugging.",
    ),
    (
        "basic-memory",
        "Persistent local knowledge base across sessions.",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    User,
    Project,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::User => "user",
            Scope::Project => "project",
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Scope::User => Scope::Project,
            Scope::Project => Scope::User,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub summary: &'static str,
    pub scope: Scope,
    /// Claude-style server definition (`type` + `url`/`headers` or `command`/`args`/`env`).
    pub server: Value,
}

impl Entry {
    pub fn is_http(&self) -> bool {
        matches!(self.str_field("type"), Some("http" | "sse"))
    }

    pub fn str_field(&self, key: &str) -> Option<&str> {
        self.server.get(key).and_then(Value::as_str)
    }

    pub fn args(&self) -> Vec<String> {
        string_list(self.server.get("args"))
    }

    pub fn pairs(&self, key: &str) -> Vec<(String, String)> {
        self.server
            .get(key)
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string())))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// `url` for remote servers, `command args…` for stdio ones.
    pub fn target(&self) -> String {
        if self.is_http() {
            return self.str_field("url").unwrap_or_default().into();
        }
        let mut parts = vec![self.str_field("command").unwrap_or_default().to_string()];
        parts.extend(self.args());
        parts.join(" ")
    }

    /// Environment variables referenced as `${VAR}` anywhere in the definition.
    pub fn env_refs(&self) -> Vec<String> {
        let mut out = vec![];
        collect_env_refs(&self.server, &mut out);
        out
    }
}

pub fn catalog() -> Vec<Entry> {
    let mut entries: Vec<Entry> = vec![];
    for (source, scope) in [(GLOBAL, Scope::User), (PROJECT, Scope::Project)] {
        let parsed: Value =
            serde_json::from_str(source).expect("embedded MCP catalog is valid JSON");
        let Some(servers) = parsed.get("mcpServers").and_then(Value::as_object) else {
            continue;
        };
        for (name, server) in servers {
            // A server listed globally wins over its per-project duplicate (figma).
            if entries.iter().any(|e| &e.name == name) {
                continue;
            }
            entries.push(Entry {
                name: name.clone(),
                summary: SUMMARIES
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, s)| *s)
                    .unwrap_or(""),
                scope,
                server: server.clone(),
            });
        }
    }
    entries
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn collect_env_refs(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            let mut rest = s.as_str();
            while let Some(start) = rest.find("${") {
                let after = &rest[start + 2..];
                let Some(end) = after.find('}') else { break };
                let name = after[..end].split(":-").next().unwrap_or_default();
                if !name.is_empty() && !out.iter().any(|n| n == name) {
                    out.push(name.to_string());
                }
                rest = &after[end + 1..];
            }
        }
        Value::Array(items) => items.iter().for_each(|v| collect_env_refs(v, out)),
        Value::Object(map) => map.values().for_each(|v| collect_env_refs(v, out)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_catalog_parses_and_every_entry_is_described() {
        let entries = catalog();
        assert!(entries.len() >= 6);
        for entry in &entries {
            assert!(!entry.summary.is_empty(), "{} has no summary", entry.name);
            assert!(!entry.target().is_empty(), "{} has no target", entry.name);
        }
        assert_eq!(
            entries.iter().filter(|e| e.name == "figma").count(),
            1,
            "figma is deduplicated"
        );
        let figma = entries.iter().find(|e| e.name == "figma").unwrap();
        assert_eq!(figma.scope, Scope::User);
    }

    #[test]
    fn github_uses_the_remote_server_and_references_its_token_by_name() {
        let github = catalog().into_iter().find(|e| e.name == "github").unwrap();
        assert!(github.is_http());
        assert_eq!(github.target(), "https://api.githubcopilot.com/mcp/");
        assert_eq!(github.env_refs(), vec!["GITHUB_PAT".to_string()]);
    }
}

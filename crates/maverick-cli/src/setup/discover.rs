use anyhow::Result;
use maverick_core::{ProviderConfig, ProviderKind};
#[cfg(test)]
use std::collections::{HashMap, HashSet};
use std::{env, fs, process::Command};

pub trait Probe {
    fn binary_exists(&self, name: &str) -> bool;
    fn output(&self, name: &str, args: &[&str]) -> Result<String, String>;
    fn env(&self, name: &str) -> Option<String>;
}

pub struct SystemProbe;

impl Probe for SystemProbe {
    fn binary_exists(&self, name: &str) -> bool {
        env::var_os("PATH")
            .map(|paths| {
                env::split_paths(&paths).any(|dir| {
                    let path = dir.join(name);
                    path.is_file()
                        || fs::symlink_metadata(&path)
                            .map(|m| m.file_type().is_symlink())
                            .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    fn output(&self, name: &str, args: &[&str]) -> Result<String, String> {
        let output = Command::new(name)
            .args(args)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().into());
        }
        Ok(String::from_utf8_lossy(&output.stdout).into())
    }

    fn env(&self, name: &str) -> Option<String> {
        env::var(name).ok().filter(|value| !value.is_empty())
    }
}

#[cfg(test)]
pub struct MapProbe {
    pub binaries: HashSet<String>,
    pub outputs: HashMap<(String, Vec<String>), String>,
    pub env: HashMap<String, String>,
}

#[cfg(test)]
impl Probe for MapProbe {
    fn binary_exists(&self, name: &str) -> bool {
        self.binaries.contains(name)
    }
    fn output(&self, name: &str, args: &[&str]) -> Result<String, String> {
        self.outputs
            .get(&(name.into(), args.iter().map(|s| (*s).to_string()).collect()))
            .cloned()
            .ok_or_else(|| format!("{name} not stubbed"))
    }
    fn env(&self, name: &str) -> Option<String> {
        self.env.get(name).cloned()
    }
}

pub fn discover(probe: &impl Probe) -> Result<Vec<ProviderConfig>> {
    let mut providers = vec![];
    if probe.binary_exists("grok") {
        let models = probe
            .output("grok", &["models"])
            .map(|out| parse_grok_models(&out))
            .unwrap_or_default();
        providers.push(ProviderConfig {
            id: "grok".into(),
            kind: ProviderKind::Grok,
            display_name: "Grok CLI".into(),
            enabled: true,
            base_url: None,
            auth_env: None,
            models,
        });
    }
    if probe.env("XAI_API_KEY").is_some() {
        providers.push(ProviderConfig {
            id: "xai-api".into(),
            kind: ProviderKind::XaiApi,
            display_name: "xAI API".into(),
            enabled: true,
            base_url: Some("https://api.x.ai/v1".into()),
            auth_env: Some("XAI_API_KEY".into()),
            models: vec!["grok-4.6".into()],
        });
    }
    if probe.binary_exists("claude") {
        providers.push(cli_provider("claude", "Claude Code", ProviderKind::Claude));
    }
    if probe.binary_exists("codex") {
        providers.push(cli_provider("codex", "Codex", ProviderKind::Codex));
    }
    if probe.binary_exists("ollama") {
        let models = probe
            .output("ollama", &["list"])
            .map(|out| parse_ollama_list(&out))
            .unwrap_or_default();
        providers.push(ProviderConfig {
            id: "ollama".into(),
            kind: ProviderKind::Ollama,
            display_name: "Ollama".into(),
            enabled: true,
            base_url: Some("http://127.0.0.1:11434/v1".into()),
            auth_env: None,
            models,
        });
    }
    Ok(providers)
}

fn cli_provider(id: &str, name: &str, kind: ProviderKind) -> ProviderConfig {
    ProviderConfig {
        id: id.into(),
        kind,
        display_name: name.into(),
        enabled: true,
        base_url: None,
        auth_env: None,
        models: vec![],
    }
}

pub fn parse_grok_models(stdout: &str) -> Vec<String> {
    let mut models = vec![];
    for line in stdout.lines() {
        let line = line.trim();
        let rest = line
            .strip_prefix("* ")
            .or_else(|| line.strip_prefix("- "))
            .unwrap_or("");
        if rest.is_empty() {
            continue;
        }
        let id = rest.split_whitespace().next().unwrap_or("");
        if maverick_core::ModelRef::new("grok", id).is_ok() && !models.iter().any(|m| m == id) {
            models.push(id.into());
        }
    }
    models
}

pub fn parse_ollama_list(stdout: &str) -> Vec<String> {
    let mut models = vec![];
    for line in stdout.lines().skip(1) {
        let id = line.split_whitespace().next().unwrap_or("");
        if maverick_core::ModelRef::new("ollama", id).is_ok() && !models.iter().any(|m| m == id) {
            models.push(id.into());
        }
    }
    models
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grok_and_openai_compatible_discovery_share_the_same_path() {
        let grok_out = "\
You are logged in with grok.com.\n\nDefault model: grok-4.6\n\nAvailable models:\n  * grok-4.6 (default)\n  - grok-4.5\n";
        assert_eq!(parse_grok_models(grok_out), ["grok-4.6", "grok-4.5"]);
        let mut probe = MapProbe {
            binaries: HashSet::from(["grok".into(), "claude".into(), "codex".into()]),
            outputs: HashMap::from([(("grok".into(), vec!["models".into()]), grok_out.into())]),
            env: HashMap::new(),
        };
        let found = discover(&probe).unwrap();
        assert!(found
            .iter()
            .any(|p| p.id == "grok" && p.models == ["grok-4.6", "grok-4.5"]));
        assert!(found
            .iter()
            .any(|p| p.id == "claude" && p.models.is_empty()));
        assert!(found.iter().all(|p| p.id != "ollama"));
        probe
            .env
            .insert("XAI_API_KEY".into(), "secret-should-not-be-stored".into());
        let with_api = discover(&probe).unwrap();
        let api = with_api.iter().find(|p| p.id == "xai-api").unwrap();
        assert_eq!(api.auth_env.as_deref(), Some("XAI_API_KEY"));
        assert_eq!(api.models, ["grok-4.6"]);
        assert!(!serde_json::to_string(api)
            .unwrap()
            .contains("secret-should-not-be-stored"));
    }

    #[test]
    fn ollama_list_parses_tags_without_failing_when_absent() {
        let list = "NAME\tID\nllama3.2:latest\tabc\n";
        assert_eq!(parse_ollama_list(list), ["llama3.2:latest"]);
        let probe = MapProbe {
            binaries: HashSet::new(),
            outputs: HashMap::new(),
            env: HashMap::new(),
        };
        assert!(discover(&probe).unwrap().is_empty());
    }
}

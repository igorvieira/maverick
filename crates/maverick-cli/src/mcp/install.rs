use super::{
    catalog::{Entry, Scope},
    detect::Agent,
};
use std::process::{Command, Stdio};

/// The argv that installs `entry` into `agent`, or why that agent cannot take it.
/// Secrets stay as `${VAR}` references (Claude and Grok expand them at load time; Codex reads
/// the bearer token from the named variable), so nothing sensitive is written to disk.
pub fn command_for(agent: Agent, entry: &Entry, scope: Scope) -> Result<Vec<String>, String> {
    let name = entry.name.clone();
    let argv = match agent {
        Agent::Claude => vec![
            "claude".into(),
            "mcp".into(),
            "add-json".into(),
            "--scope".into(),
            scope.as_str().into(),
            name,
            entry.server.to_string(),
        ],
        Agent::Codex => {
            let mut argv: Vec<String> = vec!["codex".into(), "mcp".into(), "add".into()];
            if entry.is_http() {
                argv.push("--url".into());
                argv.push(entry.str_field("url").unwrap_or_default().into());
                for (key, value) in entry.pairs("headers") {
                    let var = key
                        .eq_ignore_ascii_case("authorization")
                        .then(|| value.strip_prefix("Bearer ${")?.strip_suffix('}'))
                        .flatten()
                        .ok_or("codex only supports a bearer token header")?;
                    argv.push("--bearer-token-env-var".into());
                    argv.push(var.into());
                }
                argv.push(name);
            } else {
                for (key, value) in entry.pairs("env") {
                    if value.contains("${") {
                        return Err("codex does not expand env references".into());
                    }
                    argv.push("--env".into());
                    argv.push(format!("{key}={value}"));
                }
                argv.push(name);
                argv.push("--".into());
                argv.extend(stdio_command(entry, agent));
            }
            argv
        }
        Agent::Grok => {
            let mut argv: Vec<String> = vec![
                "grok".into(),
                "mcp".into(),
                "add".into(),
                "--scope".into(),
                scope.as_str().into(),
            ];
            if entry.is_http() {
                argv.push("--transport".into());
                argv.push(entry.str_field("type").unwrap_or("http").into());
                for (key, value) in entry.pairs("headers") {
                    argv.push("--header".into());
                    argv.push(format!("{key}: {value}"));
                }
                argv.push(name);
                argv.push(entry.str_field("url").unwrap_or_default().into());
            } else {
                for (key, value) in entry.pairs("env") {
                    argv.push("--env".into());
                    argv.push(format!("{key}={value}"));
                }
                argv.push(name);
                argv.push("--".into());
                argv.extend(stdio_command(entry, agent));
            }
            argv
        }
    };
    Ok(argv)
}

/// The catalog is written for Claude Code; Serena picks its tool set from `--context`.
fn stdio_command(entry: &Entry, agent: Agent) -> Vec<String> {
    let mut parts = vec![entry.str_field("command").unwrap_or_default().to_string()];
    parts.extend(entry.args().into_iter().map(|arg| match agent {
        Agent::Codex if arg == "--context=claude-code" => "--context=codex".into(),
        _ => arg,
    }));
    parts
}

/// Runs without a shell and without stdin, so nothing can prompt inside the TUI.
pub fn run(argv: &[String]) -> Result<(), String> {
    let output = Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = if stderr.trim().is_empty() {
        stdout
    } else {
        stderr
    };
    Err(message
        .trim()
        .lines()
        .next()
        .unwrap_or("failed")
        .to_string())
}

/// Shell-quoted rendering for review screens; never executed through a shell.
pub fn display(argv: &[String]) -> String {
    argv.iter()
        .map(|arg| {
            let plain = !arg.is_empty()
                && arg
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./:=@,+".contains(c));
            if plain {
                arg.clone()
            } else {
                format!("'{}'", arg.replace('\'', r"'\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::catalog::catalog;

    fn entry(name: &str) -> Entry {
        catalog().into_iter().find(|e| e.name == name).unwrap()
    }

    fn cmd(agent: Agent, name: &str, scope: Scope) -> String {
        display(&command_for(agent, &entry(name), scope).unwrap())
    }

    #[test]
    fn claude_adds_the_catalog_json_verbatim_with_scope() {
        assert_eq!(
            cmd(Agent::Claude, "figma", Scope::User),
            r#"claude mcp add-json --scope user figma '{"type":"http","url":"https://mcp.figma.com/mcp"}'"#
        );
        assert!(cmd(Agent::Claude, "github", Scope::Project)
            .contains(r#""Authorization":"Bearer ${GITHUB_PAT}""#));
    }

    #[test]
    fn codex_maps_http_bearer_and_stdio_and_rewrites_serena_context() {
        assert_eq!(
            cmd(Agent::Codex, "github", Scope::Project),
            "codex mcp add --url https://api.githubcopilot.com/mcp/ --bearer-token-env-var GITHUB_PAT github"
        );
        assert_eq!(
            cmd(Agent::Codex, "chrome-devtools", Scope::User),
            "codex mcp add chrome-devtools -- npx -y chrome-devtools-mcp@latest"
        );
        let serena = cmd(Agent::Codex, "serena", Scope::User);
        assert!(serena.contains("--context=codex"), "{serena}");
    }

    #[test]
    fn grok_puts_options_before_positionals_and_keeps_env_references() {
        assert_eq!(
            cmd(Agent::Grok, "github", Scope::Project),
            "grok mcp add --scope project --transport http --header 'Authorization: Bearer ${GITHUB_PAT}' github https://api.githubcopilot.com/mcp/"
        );
        assert_eq!(
            cmd(Agent::Grok, "basic-memory", Scope::User),
            "grok mcp add --scope user basic-memory -- uvx basic-memory mcp"
        );
    }

    #[test]
    fn codex_rejects_headers_it_cannot_express() {
        let mut custom = entry("figma");
        custom.server["headers"] = serde_json::json!({"X-Team": "a"});
        assert!(command_for(Agent::Codex, &custom, Scope::User).is_err());
    }
}

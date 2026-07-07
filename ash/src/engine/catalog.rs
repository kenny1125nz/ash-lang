/// Agent catalog — the source of truth for known agent definitions.
///
/// The canonical catalog lives at `CATALOG_URL` (`web-site/agents.json`).
/// During discovery, ash attempts to fetch the remote catalog and falls
/// back to the copy embedded at compile time.
use serde::Deserialize;

use crate::engine::config::{AgentConfig, ApiEndpoint, ContainerConfig};
use crate::engine::types::AgentType;

pub const CATALOG_URL: &str = "https://ash.opencode.ai/agents.json";

/// Raw JSON entry from the catalog — maps directly to the JSON format.
#[derive(Debug, Clone, Deserialize)]
struct CatalogEntry {
    name: String,
    #[serde(default)]
    binary: String,
    cmd: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    model_flag: Option<String>,
    #[serde(default)]
    session_flag: Option<String>,
    #[serde(default)]
    message_flag: Option<String>,
    #[serde(default)]
    stdin_prompt: bool,
    #[serde(default)]
    yes_flag: Option<String>,
    #[serde(default)]
    install_hint: String,
}

/// A single entry in the agent catalog.
#[derive(Debug, Clone)]
pub struct AgentEntry {
    pub name: String,
    pub binary: String,
    pub cmd: String,
    pub args: Vec<String>,
    pub model_flag: Option<String>,
    pub session_flag: Option<String>,
    pub message_flag: Option<String>,
    pub stdin_prompt: bool,
    pub yes_flag: Option<String>,
    pub install_hint: String,
}

impl AgentEntry {
    pub fn to_config(&self) -> AgentConfig {
        AgentConfig {
            name: self.name.clone(),
            agent_type: AgentType::LocalCli,
            driver: Some(self.name.clone()),
            cmd: self.cmd.clone(),
            args: self.args.clone(),
            model_flag: self.model_flag.clone(),
            session_flag: self.session_flag.clone(),
            message_flag: self.message_flag.clone(),
            stdin_prompt: self.stdin_prompt,
            yes_flag: self.yes_flag.clone(),
            base_url: String::new(),
            auth: None,
            endpoint: ApiEndpoint {
                method: String::new(),
                path: String::new(),
            },
            container: ContainerConfig {
                runtime: String::new(),
                image: String::new(),
                mode: String::new(),
                volumes: Vec::new(),
            },
        }
    }
}

fn parse_catalog_json(json: &str) -> Result<Vec<AgentEntry>, String> {
    let raw: Vec<CatalogEntry> =
        serde_json::from_str(json).map_err(|e| format!("catalog parse error: {}", e))?;
    Ok(raw
        .into_iter()
        .map(|e| AgentEntry {
            name: e.name,
            binary: e.binary,
            cmd: e.cmd,
            args: e.args,
            model_flag: e.model_flag,
            session_flag: e.session_flag,
            message_flag: e.message_flag,
            stdin_prompt: e.stdin_prompt,
            yes_flag: e.yes_flag,
            install_hint: e.install_hint,
        })
        .collect())
}

/// Fetch the agent catalog from the remote URL.
///
/// Uses the `online-catalog` feature (requires `ureq`). When the feature
/// is disabled or the fetch fails, returns `None`.
#[cfg(feature = "online-catalog")]
pub fn fetch_catalog(url: &str) -> Option<Vec<AgentEntry>> {
    let agent = ureq::Agent::new_with_defaults();
    let response = match agent
        .get(url)
        .header("User-Agent", "ash-agent-catalog/0.1")
        .call()
    {
        Ok(r) => r,
        Err(_) => return None,
    };
    let body = match response.into_body().read_to_vec() {
        Ok(b) => b,
        Err(_) => return None,
    };
    let text = String::from_utf8(body).ok()?;
    parse_catalog_json(&text).ok()
}

/// When the `online-catalog` feature is disabled, fetching always returns `None`.
#[cfg(not(feature = "online-catalog"))]
pub fn fetch_catalog(_url: &str) -> Option<Vec<AgentEntry>> {
    None
}

/// Hardcoded fallback catalog — always available, no filesystem or network needed.
/// Used when both the remote fetch and embedded JSON parsing fail.
fn hardcoded_catalog() -> Vec<AgentEntry> {
    vec![
        AgentEntry {
            name: "echo".into(),
            binary: "echo".into(),
            cmd: "echo".into(),
            args: vec![],
            model_flag: None,
            session_flag: None,
            message_flag: None,
            stdin_prompt: false,
            yes_flag: None,
            install_hint: String::new(),
        },
        AgentEntry {
            name: "opencode".into(),
            binary: "opencode".into(),
            cmd: "opencode".into(),
            args: vec!["run".into()],
            model_flag: Some("--model".into()),
            session_flag: Some("--continue".into()),
            message_flag: None,
            stdin_prompt: false,
            yes_flag: None,
            install_hint: "npm i -g @anomalyco/opencode".into(),
        },
        AgentEntry {
            name: "claude-code".into(),
            binary: "claude".into(),
            cmd: "claude".into(),
            args: vec![],
            model_flag: Some("--model".into()),
            session_flag: Some("--continue".into()),
            message_flag: Some("--msg".into()),
            stdin_prompt: false,
            yes_flag: None,
            install_hint: "npm i -g @anthropic-ai/claude-code".into(),
        },
        AgentEntry {
            name: "aider".into(),
            binary: "aider".into(),
            cmd: "aider".into(),
            args: vec![],
            model_flag: Some("--model".into()),
            session_flag: Some("--restore-chat-history".into()),
            message_flag: Some("--msg".into()),
            stdin_prompt: false,
            yes_flag: Some("--yes".into()),
            install_hint: "pip install aider-chat".into(),
        },
        AgentEntry {
            name: "codex".into(),
            binary: "codex".into(),
            cmd: "codex".into(),
            args: vec!["run".into()],
            model_flag: Some("--model".into()),
            session_flag: None,
            message_flag: None,
            stdin_prompt: false,
            yes_flag: Some("--yolo".into()),
            install_hint: "npm i -g @openai/codex".into(),
        },
        AgentEntry {
            name: "gemini-cli".into(),
            binary: "gemini".into(),
            cmd: "gemini".into(),
            args: vec!["-p".into()],
            model_flag: Some("--model".into()),
            session_flag: Some("--continue".into()),
            message_flag: None,
            stdin_prompt: false,
            yes_flag: None,
            install_hint: "npm i -g @google/gemini-cli".into(),
        },
        AgentEntry {
            name: "kimi".into(),
            binary: "kimi".into(),
            cmd: "kimi".into(),
            args: vec![],
            model_flag: Some("--model".into()),
            session_flag: Some("--continue".into()),
            message_flag: Some("-p".into()),
            stdin_prompt: false,
            yes_flag: Some("--yolo".into()),
            install_hint: "curl -fsSL https://code.kimi.com/kimi-code/install.sh | bash".into(),
        },
    ]
}

/// Return the embedded catalog (compiled into the binary).
/// Falls back to hardcoded Rust constants if the embedded JSON fails to parse.
pub fn embedded_catalog() -> Vec<AgentEntry> {
    let json = include_str!("../../../web-site/agents.json");
    match parse_catalog_json(json) {
        Ok(entries) if !entries.is_empty() => entries,
        _ => hardcoded_catalog(),
    }
}

/// Try remote, then embedded JSON, then hardcoded Rust constants.
pub fn load_catalog() -> Vec<AgentEntry> {
    #[cfg(feature = "online-catalog")]
    {
        match fetch_catalog(CATALOG_URL) {
            Some(entries) if !entries.is_empty() => return entries,
            _ => {}
        }
    }
    embedded_catalog()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agents_json() {
        let json = r#"[
  { "name": "echo", "binary": "echo", "cmd": "echo", "args": [], "model_flag": null, "session_flag": null, "message_flag": null, "stdin_prompt": false, "yes_flag": null, "install_hint": "" },
  { "name": "opencode", "binary": "opencode", "cmd": "opencode", "args": ["run"], "model_flag": "--model", "session_flag": "--continue", "message_flag": null, "stdin_prompt": false, "yes_flag": null, "install_hint": "npm i -g @anomalyco/opencode" }
]"#;
        let entries = parse_catalog_json(json).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "echo");
        assert_eq!(entries[0].cmd, "echo");
        assert_eq!(entries[1].name, "opencode");
        assert_eq!(entries[1].args, vec!["run"]);
        assert_eq!(entries[1].model_flag.as_deref(), Some("--model"));
    }

    #[test]
    fn test_parse_with_nulls() {
        let json = r#"[
  { "name": "test", "binary": "test", "cmd": "test", "args": [], "model_flag": null, "session_flag": "abc", "message_flag": null, "stdin_prompt": false, "yes_flag": null, "install_hint": "" }
]"#;
        let entries = parse_catalog_json(json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test");
        assert!(entries[0].model_flag.is_none());
        assert_eq!(entries[0].session_flag.as_deref(), Some("abc"));
        assert!(entries[0].message_flag.is_none());
    }

    #[test]
    fn test_parse_empty_array() {
        let entries = parse_catalog_json("[]").unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_embedded_catalog_contains_all_known() {
        let entries = embedded_catalog();
        assert!(entries.len() >= 5, "expected at least 5 agents, got {}", entries.len());
        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"echo"));
        assert!(names.contains(&"opencode"));
        assert!(names.contains(&"claude-code"));
        assert!(names.contains(&"aider"));
        assert!(names.contains(&"codex"));
    }

    #[test]
    fn test_to_config_roundtrip() {
        let entries = embedded_catalog();
        for entry in &entries {
            let config = entry.to_config();
            assert_eq!(config.name, entry.name);
            assert_eq!(config.cmd, entry.cmd);
        }
    }

    #[test]
    fn test_parse_invalid_returns_error() {
        let result = parse_catalog_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_entries_have_required_fields() {
        let entries = embedded_catalog();
        for entry in &entries {
            assert!(!entry.name.is_empty(), "agent missing name");
            assert!(!entry.cmd.is_empty(), "agent {} missing cmd", entry.name);
        }
    }
}

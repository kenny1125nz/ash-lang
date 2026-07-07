use std::process::Command;
use std::sync::mpsc;

use super::catalog;
use super::config::{AgentConfig, ApiEndpoint, ContainerConfig};
use super::from_config;
use super::register;
use super::types::AgentType;

#[derive(Debug, Clone)]
pub struct DiscoveredAgent {
    pub name: String,
    pub path: String,
    pub version: Option<String>,
    pub supports_model: bool,
    pub supports_session: bool,
    pub found: bool,
    pub install_hint: String,
}

#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub agents: Vec<DiscoveredAgent>,
}

fn find_binary(name: &str) -> Option<String> {
    if let Ok(output) = Command::new("which").arg(name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }
    None
}

fn probe_version(binary: &str) -> Option<String> {
    let output = Command::new("timeout")
        .args(["5", binary, "--version"])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !version.is_empty() {
                return Some(version.lines().next().unwrap_or("").to_string());
            }
        }
    }
    None
}

fn probe_capabilities(binary: &str) -> (bool, bool) {
    let mut supports_model = false;
    let mut supports_session = false;

    let output = Command::new("timeout")
        .args(["5", binary, "--help"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}\n{}", stdout, stderr);

        supports_model = combined.contains("--model");
        supports_session =
            combined.contains("--continue") || combined.contains("--restore-chat-history");
    }

    (supports_model, supports_session)
}

struct ProbeResult {
    name: String,
    path: Option<String>,
    version: Option<String>,
    supports_model: bool,
    supports_session: bool,
    found: bool,
    install_hint: String,
}

/// Load the agent catalog: try remote, fall back to embedded.
pub fn discover() -> DiscoveryResult {
    let catalog = catalog::load_catalog();
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    for entry in &catalog {
        let tx = tx.clone();
        let name = entry.name.clone();
        let binary = entry.binary.clone();
        let install_hint = entry.install_hint.clone();

        handles.push(std::thread::spawn(move || {
            let found_path = find_binary(&binary);

            let (version, supports_model, supports_session) = if found_path.is_some() {
                let v = probe_version(&binary);
                let (m, s) = probe_capabilities(&binary);
                (v, m, s)
            } else {
                (None, false, false)
            };

            let found = found_path.is_some() || name == "echo";

            let _ = tx.send(ProbeResult {
                name,
                path: found_path,
                version,
                supports_model,
                supports_session,
                found,
                install_hint,
            });
        }));
    }

    drop(tx);

    let mut agents: Vec<DiscoveredAgent> = Vec::new();
    for result in rx {
        let path = result.path.unwrap_or_else(|| {
            if result.name == "echo" {
                "shell builtin".to_string()
            } else {
                String::new()
            }
        });

        agents.push(DiscoveredAgent {
            name: result.name,
            path,
            version: result.version,
            supports_model: result.supports_model,
            supports_session: result.supports_session,
            found: result.found,
            install_hint: result.install_hint,
        });
    }

    // Re-sort by catalog order for deterministic output
    let mut ordered: Vec<DiscoveredAgent> = Vec::new();
    let catalog_names: Vec<&str> = catalog.iter().map(|e| e.name.as_str()).collect();
    for cat_name in &catalog_names {
        if let Some(agent) = agents.iter().find(|a| a.name.as_str() == *cat_name) {
            ordered.push(agent.clone());
        }
    }
    // Append any agents not in catalog (shouldn't happen, but be safe)
    for agent in &agents {
        if !catalog_names.contains(&agent.name.as_str()) {
            ordered.push(agent.clone());
        }
    }

    DiscoveryResult { agents: ordered }
}

fn yaml_line(key: &str, value: &str) -> String {
    format!("    {}: {}\n", key, value)
}

fn yaml_line_opt(key: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("    {}: {}\n", key, v),
        None => String::new(),
    }
}

fn yaml_line_bool(key: &str, value: bool) -> String {
    format!("    {}: {}\n", key, value)
}

fn yaml_line_args(key: &str, args: &[String]) -> String {
    if args.is_empty() {
        return String::new();
    }
    let items: Vec<String> = args.iter().map(|a| format!("\"{}\"", a)).collect();
    format!("    {}: [{}]\n", key, items.join(", "))
}

pub fn generate_yaml(result: &DiscoveryResult) -> String {
    let mut yaml = String::from("# Generated by `ash discover --write`\nagents:\n");

    let found_names: std::collections::HashSet<&str> = result
        .agents
        .iter()
        .filter(|a| a.found && a.name != "echo")
        .map(|a| a.name.as_str())
        .collect();

    if found_names.is_empty() {
        yaml.push_str("  # No agents discovered\n");
        return yaml;
    }

    let catalog = catalog::load_catalog();
    for entry in &catalog {
        if !found_names.contains(entry.name.as_str()) {
            continue;
        }
        yaml.push_str(&format!("  {}:\n", entry.name));
        yaml.push_str(&yaml_line("type", "local-cli"));
        yaml.push_str(&yaml_line("cmd", &entry.cmd));
        yaml.push_str(&yaml_line_args("args", &entry.args));
        yaml.push_str(&yaml_line_opt("model_flag", entry.model_flag.as_deref()));
        yaml.push_str(&yaml_line_opt("session_flag", entry.session_flag.as_deref()));
        yaml.push_str(&yaml_line_opt("message_flag", entry.message_flag.as_deref()));
        yaml.push_str(&yaml_line_bool("stdin_prompt", entry.stdin_prompt));
        yaml.push_str(&yaml_line_opt("yes_flag", entry.yes_flag.as_deref()));
    }

    yaml
}

pub fn write_config(path: &str, result: &DiscoveryResult, force: bool) -> Result<(), String> {
    let p = std::path::Path::new(path);
    if p.exists() && !force {
        return Err(format!(
            "{} already exists. Use --force to overwrite.",
            path
        ));
    }

    let has_found = result.agents.iter().any(|a| a.found && a.name != "echo");
    if !has_found {
        return Err("no agents found to write to config".to_string());
    }

    let yaml = generate_yaml(result);
    std::fs::write(path, &yaml).map_err(|e| format!("failed to write {}: {}", path, e))
}

pub fn print_discovery(result: &DiscoveryResult) {
    let found: Vec<&DiscoveredAgent> = result.agents.iter().filter(|a| a.found).collect();
    let not_found: Vec<&DiscoveredAgent> = result.agents.iter().filter(|a| !a.found).collect();
    let has_ai_agents = found.iter().any(|a| a.name != "echo");

    if !has_ai_agents && not_found.is_empty() {
        println!("No agents found. Install one: npm i -g @anthropic-ai/claude-code");
        return;
    }

    for agent in &found {
        if agent.name == "echo" {
            continue;
        }
        print!("  {}  {:28}", '\u{2713}', agent.name);
        print!("  {}", agent.path);
        if let Some(ref v) = agent.version {
            print!("  ({})", v);
        }
        println!();
        if agent.supports_model || agent.supports_session {
            print!("      capabilities:");
            if agent.supports_model {
                print!(" model");
            }
            if agent.supports_session {
                print!(" session");
            }
            println!();
        }
    }

    if !not_found.is_empty() {
        if found.iter().any(|a| a.name != "echo") {
            println!();
        }
        for agent in &not_found {
            println!("  {}  {:28}  not found", '\u{2717}', agent.name);
            if !agent.install_hint.is_empty() {
                println!("      install: {}", agent.install_hint);
            }
        }
    }
}

pub fn discovery_summary(result: &DiscoveryResult) -> String {
    let found: Vec<&DiscoveredAgent> = result.agents.iter().filter(|a| a.found).collect();
    let names: Vec<&str> = found.iter().map(|a| a.name.as_str()).collect();

    if names.is_empty() {
        "No agents found. Install one: npm i -g @anthropic-ai/claude-code".to_string()
    } else {
        format!(
            "Found: {}. Run `ash discover` for details.",
            names.join(", ")
        )
    }
}

pub fn discover_and_register() -> DiscoveryResult {
    let result = discover();
    let catalog = catalog::load_catalog();

    for entry in &catalog {
        let found = result.agents.iter().any(|a| a.name == entry.name && a.found);
        if !found {
            continue;
        }

        let config = entry.to_config();
        let adapter = from_config(&config);
        register(&config.name, adapter);
    }

    result
}

/// YAML config file structure — maps directly to the `ash.yml` format.
#[derive(Debug, Clone, serde::Deserialize)]
struct ConfigFile {
    #[serde(default)]
    agents: std::collections::BTreeMap<String, AgentDef>,
}

/// A single agent definition in the config YAML.
#[derive(Debug, Clone, serde::Deserialize)]
struct AgentDef {
    #[serde(rename = "type", default)]
    agent_type: Option<String>,
    driver: Option<String>,
    cmd: Option<String>,
    args: Option<Vec<String>>,
    model_flag: Option<String>,
    session_flag: Option<String>,
    message_flag: Option<String>,
    stdin_prompt: Option<bool>,
    yes_flag: Option<String>,
}

fn merge_into_config(name: &str, def: AgentDef) -> AgentConfig {
    let agent_type = match def.agent_type.as_deref() {
        Some("api") => AgentType::Api,
        Some("container") => AgentType::Container,
        Some("browser") => AgentType::Browser,
        _ => AgentType::LocalCli,
    };
    let args = def.args.unwrap_or_default();

    // For `local-cli` with a known `driver:` name (old format), leave cmd empty
    // so `from_config()` dispatches to the hardcoded driver. Otherwise use `cmd`.
    let has_driver = def.driver.is_some() && agent_type == AgentType::LocalCli;
    let cmd = if has_driver {
        def.cmd.unwrap_or_default()
    } else {
        def.cmd.unwrap_or_default()
    };

    AgentConfig {
        name: name.to_string(),
        agent_type,
        driver: def.driver,
        cmd,
        args,
        model_flag: def.model_flag,
        session_flag: def.session_flag,
        message_flag: def.message_flag,
        stdin_prompt: def.stdin_prompt.unwrap_or(false),
        yes_flag: def.yes_flag,
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

/// Parse the agent config file and return the list of configured agents.
///
/// Supports both the old format with `driver:` (backward compat) and the new
/// format with structured fields (`cmd`, `args`, `model_flag`, etc.).
pub fn read_config(path: &str) -> Result<Vec<AgentConfig>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path, e))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let config: ConfigFile =
        serde_yaml::from_str(&content).map_err(|e| format!("failed to parse {}: {}", path, e))?;

    Ok(config
        .agents
        .into_iter()
        .map(|(name, def)| merge_into_config(&name, def))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn write_tmp_yaml(content: &str) -> std::path::PathBuf {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("ash-test-config-{}-{}", std::process::id(), id));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_read_config_new_format() {
        let yaml = r#"# Generated by ash
agents:
  opencode:
    type: local-cli
    cmd: opencode
    args: ["run"]
    model_flag: --model
    session_flag: --continue
    stdin_prompt: false
"#;
        let path = write_tmp_yaml(yaml);
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "opencode");
        assert_eq!(agents[0].cmd, "opencode");
        assert_eq!(agents[0].args, vec!["run"]);
        assert_eq!(agents[0].model_flag.as_deref(), Some("--model"));
        assert_eq!(agents[0].session_flag.as_deref(), Some("--continue"));
        assert!(agents[0].message_flag.is_none());
        assert_eq!(agents[0].stdin_prompt, false);
        assert!(agents[0].driver.is_none());
    }

    #[test]
    fn test_read_config_old_format() {
        let yaml = r#"agents:
  opencode:
    type: local-cli
    driver: opencode
"#;
        let path = write_tmp_yaml(yaml);
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "opencode");
        assert_eq!(agents[0].driver.as_deref(), Some("opencode"));
    }

    #[test]
    fn test_read_config_custom_agent() {
        let yaml = r#"agents:
  copilot:
    type: local-cli
    cmd: gh
    args: ["copilot", "suggest"]
    model_flag: --model
    message_flag: --prompt
"#;
        let path = write_tmp_yaml(yaml);
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "copilot");
        assert_eq!(agents[0].cmd, "gh");
        assert_eq!(agents[0].args, vec!["copilot", "suggest"]);
        assert_eq!(agents[0].model_flag.as_deref(), Some("--model"));
        assert_eq!(agents[0].message_flag.as_deref(), Some("--prompt"));
        assert!(agents[0].session_flag.is_none());
    }

    #[test]
    fn test_read_config_multiple_agents() {
        let yaml = r#"agents:
  opencode:
    type: local-cli
    cmd: opencode
  aider:
    type: local-cli
    cmd: aider
    message_flag: --msg
"#;
        let path = write_tmp_yaml(yaml);
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(agents.len(), 2);
        // BTreeMap sorts by name, so aider < opencode
        assert_eq!(agents[0].name, "aider");
        assert_eq!(agents[0].message_flag.as_deref(), Some("--msg"));
        assert_eq!(agents[1].name, "opencode");
    }

    #[test]
    fn test_read_config_empty_file() {
        let path = write_tmp_yaml("");
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_read_config_no_agents_key() {
        let yaml = "# just a comment\nsome_other_key: value\n";
        let path = write_tmp_yaml(yaml);
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert!(agents.is_empty());
    }

    #[test]
    fn test_read_config_mixed_old_and_new() {
        let yaml = r#"agents:
  test-agent:
    type: local-cli
    driver: opencode
    cmd: custom-bin
    model_flag: --model
"#;
        let path = write_tmp_yaml(yaml);
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].driver.as_deref(), Some("opencode"));
        assert_eq!(agents[0].cmd, "custom-bin");
        assert_eq!(agents[0].model_flag.as_deref(), Some("--model"));
    }

    #[test]
    fn test_discover_echo_always_found() {
        let result = discover();
        let echo = result.agents.iter().find(|a| a.name == "echo").unwrap();
        assert!(echo.found);
        assert!(!echo.path.is_empty());
    }

    #[test]
    fn test_discover_includes_all_catalog_agents() {
        let result = discover();
        let catalog = super::catalog::embedded_catalog();
        for entry in &catalog {
            let agent = result.agents.iter().find(|a| a.name == entry.name);
            assert!(agent.is_some(), "agent '{}' from catalog not in discovery result", entry.name);
        }
    }

    #[test]
    fn test_read_config_with_yes_flag() {
        let yaml = r#"agents:
  codex:
    type: local-cli
    cmd: codex
    args: ["run"]
    yes_flag: --yolo
"#;
        let path = write_tmp_yaml(yaml);
        let agents = read_config(path.to_str().unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "codex");
        assert_eq!(agents[0].yes_flag.as_deref(), Some("--yolo"));
    }
}

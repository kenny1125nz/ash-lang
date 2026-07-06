use std::process::Command;

use super::config::{AgentConfig, ApiEndpoint, ContainerConfig};
use super::from_config;
use super::register;
use super::types::AgentType;

struct DriverInfo {
    pub agent_name: &'static str,
    pub binary_name: &'static str,
    pub driver_name: &'static str,
    pub cmd: &'static str,
    pub args: &'static [&'static str],
    pub model_flag: Option<&'static str>,
    pub session_flag: Option<&'static str>,
    pub message_flag: Option<&'static str>,
    pub stdin_prompt: bool,
    pub install_hint: &'static str,
}

const KNOWN_DRIVERS: &[DriverInfo] = &[
    DriverInfo {
        agent_name: "echo",
        binary_name: "echo",
        driver_name: "echo",
        cmd: "echo",
        args: &[],
        model_flag: None,
        session_flag: None,
        message_flag: None,
        stdin_prompt: false,
        install_hint: "",
    },
    DriverInfo {
        agent_name: "opencode",
        binary_name: "opencode",
        driver_name: "opencode",
        cmd: "opencode",
        args: &["run"],
        model_flag: Some("--model"),
        session_flag: Some("--continue"),
        message_flag: None,
        stdin_prompt: false,
        install_hint: "npm i -g @anomalyco/opencode",
    },
    DriverInfo {
        agent_name: "claude-code",
        binary_name: "claude",
        driver_name: "claude-code",
        cmd: "claude",
        args: &[],
        model_flag: Some("--model"),
        session_flag: Some("--continue"),
        message_flag: Some("--msg"),
        stdin_prompt: false,
        install_hint: "npm i -g @anthropic-ai/claude-code",
    },
    DriverInfo {
        agent_name: "aider",
        binary_name: "aider",
        driver_name: "aider",
        cmd: "aider",
        args: &[],
        model_flag: Some("--model"),
        session_flag: Some("--restore-chat-history"),
        message_flag: Some("--msg"),
        stdin_prompt: false,
        install_hint: "pip install aider-chat",
    },
];

#[derive(Debug, Clone)]
pub struct DiscoveredAgent {
    pub name: String,
    pub driver: String,
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

pub fn discover() -> DiscoveryResult {
    let mut agents = Vec::new();

    for info in KNOWN_DRIVERS {
        let found_path = find_binary(info.binary_name);

        let (version, supports_model, supports_session) = if found_path.is_some() {
            let v = probe_version(info.binary_name);
            let (m, s) = probe_capabilities(info.binary_name);
            (v, m, s)
        } else {
            (None, false, false)
        };

        let found = found_path.is_some() || info.agent_name == "echo";

        let path = found_path.unwrap_or_else(|| {
            if info.agent_name == "echo" {
                "shell builtin".to_string()
            } else {
                String::new()
            }
        });

        agents.push(DiscoveredAgent {
            name: info.agent_name.to_string(),
            driver: info.driver_name.to_string(),
            path,
            version,
            supports_model,
            supports_session,
            found,
            install_hint: info.install_hint.to_string(),
        });
    }

    DiscoveryResult { agents }
}

fn build_agent_config(info: &DriverInfo) -> AgentConfig {
    AgentConfig {
        name: info.agent_name.to_string(),
        agent_type: AgentType::LocalCli,
        driver: Some(info.driver_name.to_string()),
        cmd: info.cmd.to_string(),
        args: info.args.iter().map(|s| s.to_string()).collect(),
        model_flag: info.model_flag.map(|s| s.to_string()),
        session_flag: info.session_flag.map(|s| s.to_string()),
        message_flag: info.message_flag.map(|s| s.to_string()),
        stdin_prompt: info.stdin_prompt,
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

    for info in KNOWN_DRIVERS {
        if !found_names.contains(info.agent_name) {
            continue;
        }
        yaml.push_str(&format!("  {}:\n", info.agent_name));
        let cfg = build_agent_config(info);
        yaml.push_str(&yaml_line("type", "local-cli"));
        // Emit the structured fields. Omit driver: so read_config() uses GenericDriver.
        yaml.push_str(&yaml_line("cmd", &cfg.cmd));
        yaml.push_str(&yaml_line_args("args", &cfg.args));
        yaml.push_str(&yaml_line_opt("model_flag", info.model_flag));
        yaml.push_str(&yaml_line_opt("session_flag", info.session_flag));
        yaml.push_str(&yaml_line_opt("message_flag", info.message_flag));
        yaml.push_str(&yaml_line_bool("stdin_prompt", cfg.stdin_prompt));
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

    for info in KNOWN_DRIVERS {
        let found = result.agents.iter().any(|a| a.name == info.agent_name && a.found);
        if !found {
            continue;
        }

        let config = build_agent_config(info);
        let adapter = from_config(&config);
        register(&config.name, adapter);
    }

    result
}

fn parse_yaml_array(s: &str) -> Vec<String> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Vec::new();
    }
    let inner = s[1..s.len() - 1].trim();
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(|item| {
            let item = item.trim().trim_matches('"').trim_matches('\'');
            item.to_string()
        })
        .collect()
}

/// Parse `ash-project.yaml` and return the list of configured agents.
///
/// Supports both the old format with `driver:` (backward compat) and the new
/// format with structured fields (`cmd`, `args`, `model_flag`, etc.).
pub fn read_config(path: &str) -> Result<Vec<AgentConfig>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path, e))?;

    let mut agents = Vec::new();
    let mut current_name: Option<String> = None;
    let mut in_agents = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "agents:" {
            in_agents = true;
            continue;
        }

        if !in_agents {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        // Indent 0 means we've left the agents section
        if indent == 0 {
            in_agents = false;
            current_name = None;
            continue;
        }

        // Indent 2: agent name line, e.g. "  opencode:"
        if indent == 2 && trimmed.ends_with(':') {
            current_name = Some(trimmed.trim_end_matches(':').to_string());
            continue;
        }

        // Indent 4: key: value pairs
        if indent == 4 {
            if let Some(ref name) = current_name {
                if let Some((key, value)) = trimmed.split_once(':') {
                    let key = key.trim();
                    let raw_value = value.trim();

                    // Find existing agent or create a new one
                    let idx = agents.iter().position(|a: &AgentConfig| a.name == *name);
                    let idx = idx.unwrap_or_else(|| {
                        agents.push(AgentConfig {
                            name: name.clone(),
                            agent_type: AgentType::LocalCli,
                            driver: None,
                            cmd: String::new(),
                            args: Vec::new(),
                            model_flag: None,
                            session_flag: None,
                            message_flag: None,
                            stdin_prompt: false,
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
                        });
                        agents.len() - 1
                    });
                    let agent = &mut agents[idx];

                    match key {
                        "type" => {
                            agent.agent_type = match raw_value {
                                "api" => AgentType::Api,
                                "container" => AgentType::Container,
                                "browser" => AgentType::Browser,
                                _ => AgentType::LocalCli,
                            };
                        }
                        "driver" => {
                            let v = raw_value.trim_matches('"').to_string();
                            if !v.is_empty() {
                                agent.driver = Some(v);
                            }
                        }
                        "cmd" => {
                            agent.cmd = raw_value.trim_matches('"').to_string();
                        }
                        "args" => {
                            agent.args = parse_yaml_array(raw_value);
                        }
                        "model_flag" => {
                            let v = raw_value.trim_matches('"').to_string();
                            if !v.is_empty() {
                                agent.model_flag = Some(v);
                            }
                        }
                        "session_flag" => {
                            let v = raw_value.trim_matches('"').to_string();
                            if !v.is_empty() {
                                agent.session_flag = Some(v);
                            }
                        }
                        "message_flag" => {
                            let v = raw_value.trim_matches('"').to_string();
                            if !v.is_empty() {
                                agent.message_flag = Some(v);
                            }
                        }
                        "stdin_prompt" => {
                            agent.stdin_prompt = raw_value == "true";
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(agents)
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
        assert_eq!(agents[0].name, "opencode");
        assert_eq!(agents[1].name, "aider");
        assert_eq!(agents[1].message_flag.as_deref(), Some("--msg"));
    }

    #[test]
    fn test_parse_yaml_array() {
        assert_eq!(parse_yaml_array(r#"["run"]"#), vec!["run"]);
        assert_eq!(parse_yaml_array(r#"["copilot", "suggest"]"#), vec!["copilot", "suggest"]);
        assert_eq!(parse_yaml_array(r#"[]"#), Vec::<String>::new());
        assert_eq!(parse_yaml_array(""), Vec::<String>::new());
    }
}

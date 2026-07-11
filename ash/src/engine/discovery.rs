use std::process::Command;
use std::sync::mpsc;

use super::catalog;
use super::config::AgentConfig;
use super::from_config;
use super::register;

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
    pub timestamp: String,
}

pub fn discover() -> DiscoveryResult {
    eprintln!("discovering agents available...");
    let catalog = catalog::embedded_catalog();
    let timestamp = crate::runtime::date::timestamp_now();
    let mut agents: Vec<DiscoveredAgent> = Vec::new();

    // Pre-collect owned data to avoid borrowing catalog across thread boundaries
    let entries: Vec<(String, String, String)> = catalog
        .iter()
        .map(|e| (e.name.clone(), e.cmd.clone(), e.install_hint.clone()))
        .collect();
    let num_entries = entries.len();

    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    for (name, cmd, install_hint) in entries {
        let tx = tx.clone();
        handles.push(std::thread::spawn(move || {
            let path = which(&cmd);
            let (supports_model, supports_session) = if let Some(ref p) = path {
                probe_capabilities(p, &name)
            } else {
                (false, false)
            };
            let version = path.as_ref().and_then(|p| get_version(p));
            let found = path.is_some();
            let path_str = path.unwrap_or_default();
            let _ = tx.send(DiscoveredAgent {
                name,
                path: path_str,
                version,
                supports_model,
                supports_session,
                found,
                install_hint,
            });
        }));
    }

    drop(tx);

    for _ in 0..num_entries {
        if let Ok(agent) = rx.recv() {
            agents.push(agent);
        }
    }

    // Add echo even if it wasn't probed
    if let Some(echo_path) = which("echo") {
        agents.push(DiscoveredAgent {
            name: "echo".to_string(),
            path: echo_path,
            version: None,
            supports_model: false,
            supports_session: false,
            found: true,
            install_hint: String::new(),
        });
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));

    DiscoveryResult {
        agents,
        timestamp,
    }
}

/// Convert discovery results to a list of agent configs suitable for writing.
pub fn discovered_to_configs(result: &DiscoveryResult) -> Vec<AgentConfig> {
    let catalog = catalog::embedded_catalog();
    result
        .agents
        .iter()
        .filter(|a| a.found && a.name != "echo")
        .filter_map(|a| catalog.iter().find(|e| e.name == a.name).map(|e| e.to_config()))
        .collect()
}

pub fn discover_and_register() -> DiscoveryResult {
    let result = discover();
    let catalog = catalog::embedded_catalog();

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

// --- Helpers ---

fn which(name: &str) -> Option<String> {
    let cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    let output = Command::new(cmd).arg(name).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().lines().next().unwrap_or("").to_string())
    } else {
        None
    }
}

fn get_version(path: &str) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn probe_capabilities(path: &str, name: &str) -> (bool, bool) {
    let has_model = match name {
        "opencode" => true,
        "claude-code" | "claude" => true,
        "aider" => true,
        "gemini-cli" => true,
        "kimi" => true,
        "codex" => true,
        _ => {
            Command::new(path)
                .arg("--help")
                .output()
                .ok()
                .map(|o| {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.contains("--model") || s.contains("-m ")
                })
                .unwrap_or(false)
        }
    };
    let has_session = match name {
        "opencode" => true,
        "claude-code" | "claude" => true,
        "aider" => true,
        "gemini-cli" => true,
        "kimi" => true,
        "codex" => true,
        _ => {
            Command::new(path)
                .arg("--help")
                .output()
                .ok()
                .map(|o| {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.contains("--continue") || s.contains("--resume") || s.contains("--restore")
                })
                .unwrap_or(false)
        }
    };
    (has_model, has_session)
}


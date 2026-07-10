use std::collections::HashSet;
use std::io::Read;

use ash::engine;
use ash::eval::{EvalError, Evaluator};
use ash::lang::ast::{Node, Script};
use ash::lang::parser::parse_str;
use ash::runtime::tree::{self, ParallelMode, WalkConfig};
use ash::AshError;

use log::info;

fn collect_agent_names<'a>(nodes: &'a [Node], names: &mut HashSet<&'a str>) {
    for node in nodes {
        match node {
            Node::AgentCall(n) => {
                if let Some(ref agent) = n.agent {
                    names.insert(agent);
                }
            }
            Node::Block(n) => collect_agent_names(&n.statements, names),
            Node::IfStmt(n) => {
                collect_agent_names(std::slice::from_ref(&n.body), names);
                for ei in &n.else_ifs {
                    collect_agent_names(std::slice::from_ref(&ei.body), names);
                }
                if let Some(ref body) = n.else_body {
                    collect_agent_names(std::slice::from_ref(body), names);
                }
            }
            Node::ForStmt(n) => {
                collect_agent_names(std::slice::from_ref(&n.body), names);
            }
            Node::WhileStmt(n) => {
                collect_agent_names(std::slice::from_ref(&n.body), names);
            }
            Node::FnDecl(n) => {
                collect_agent_names(std::slice::from_ref(&n.body), names);
            }
            Node::BinaryTry(n) => {
                match &*n.body {
                    Node::Block(b) => collect_agent_names(&b.statements, names),
                    other => collect_agent_names(std::slice::from_ref(other), names),
                }
                if let Some(ref fail) = n.fail {
                    match &**fail {
                        Node::Block(b) => collect_agent_names(&b.statements, names),
                        other => collect_agent_names(std::slice::from_ref(other), names),
                    }
                }
            }
            Node::EvalTry(n) => {
                match &*n.body {
                    Node::Block(b) => collect_agent_names(&b.statements, names),
                    other => collect_agent_names(std::slice::from_ref(other), names),
                }
                match &*n.eval {
                    Node::Block(b) => collect_agent_names(&b.statements, names),
                    other => collect_agent_names(std::slice::from_ref(other), names),
                }
                if let Some(ref accept) = n.accept {
                    match &**accept {
                        Node::Block(b) => collect_agent_names(&b.statements, names),
                        other => collect_agent_names(std::slice::from_ref(other), names),
                    }
                }
                if let Some(ref partial) = n.partial {
                    match &**partial {
                        Node::Block(b) => collect_agent_names(&b.statements, names),
                        other => collect_agent_names(std::slice::from_ref(other), names),
                    }
                }
                if let Some(ref fail) = n.fail {
                    match &**fail {
                        Node::Block(b) => collect_agent_names(&b.statements, names),
                        other => collect_agent_names(std::slice::from_ref(other), names),
                    }
                }
            }
            Node::WaitBlock(n) => {
                if let Some(ref body) = n.body {
                    if let Node::Block(b) = &**body {
                        collect_agent_names(&b.statements, names);
                    }
                }
            }
            Node::Background(n) => {
                collect_agent_names(std::slice::from_ref(&n.stmt), names);
            }
            Node::DirBlock(n) => {
                collect_agent_names(std::slice::from_ref(&n.body), names);
            }
            Node::SessionBlock(n) => {
                collect_agent_names(std::slice::from_ref(&n.body), names);
            }
            Node::SessionToggle(_) => {}
            Node::WithinToggle(_) => {}
            _ => {}
        }
    }
}

const DEFAULT_CONFIG_FILENAME: &str = "ash.yml";

fn global_config_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(home).join(".ash")
}

fn global_config_path() -> std::path::PathBuf {
    global_config_dir().join(DEFAULT_CONFIG_FILENAME)
}

fn resolve_config_path(custom: Option<&str>) -> Option<std::path::PathBuf> {
    if let Some(path) = custom {
        return Some(std::path::PathBuf::from(path));
    }
    let cwd = std::path::Path::new(DEFAULT_CONFIG_FILENAME);
    if cwd.exists() {
        return Some(cwd.to_path_buf());
    }
    let global = global_config_path();
    if global.exists() {
        return Some(global);
    }
    None
}

fn validate_agents(script: &Script, config_path: Option<&str>) -> Result<(), AshError> {
    let mut used_agents = HashSet::new();
    collect_agent_names(&script.body, &mut used_agents);

    if let Some(ref shebang) = script.shebang {
        used_agents.insert(&shebang.engine);
    }

    let builtin: HashSet<&str> = ["echo"].iter().copied().collect();

    used_agents.retain(|name| !builtin.contains(name));

    if used_agents.is_empty() {
        return Ok(());
    }

    let resolved = resolve_config_path(config_path);
    let config_path_str = match resolved {
        Some(ref p) => p.to_str().unwrap().to_string(),
        None => {
            eprintln!(
                "warning: script references agent(s) ({}) but no {} found — \
                 agent names will be resolved against the default registry",
                used_agents.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "),
                DEFAULT_CONFIG_FILENAME,
            );
            return Ok(());
        }
    };

    let configured = match ash::config::read_config(&config_path_str) {
        Ok((agents, _)) => agents.into_iter().map(|a| a.name).collect::<HashSet<_>>(),
        Err(e) => {
            return Err(AshError::Msg(format!("failed to read {}: {}", config_path_str, e)));
        }
    };

    let mut unknown: Vec<&str> = Vec::new();
    for name in &used_agents {
        if !configured.contains(*name) {
            unknown.push(name);
        }
    }

    if !unknown.is_empty() {
        return Err(AshError::Msg(format!(
            "agents not configured in {}: {}\n\
             hint: add them under the 'agents:' section, e.g.:\n\
             agents:\n\
             {}",
            config_path_str,
            unknown.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "),
            unknown.iter().map(|s| format!("  {}:\n    type: local-cli\n    cmd: <binary>\n", s)).collect::<Vec<_>>().join("")
        )));
    }

    Ok(())
}

fn parse_agent_spec(spec: Option<&str>) -> (String, String) {
    let spec = match spec {
        Some(s) => s,
        None => return ("echo".to_string(), String::new()),
    };
    let parts: Vec<&str> = spec.split(':').collect();
    let agent = parts.first().map(|s| s.to_string()).unwrap_or_else(|| "echo".to_string());
    let model = match parts.len() {
        2 => parts[1].to_string(),
        n if n >= 3 => parts[n - 1].to_string(),
        _ => String::new(),
    };
    (agent, model)
}

fn cmd_discover(args: &[String]) -> i32 {
    let mut write = false;
    let mut force = false;
    for arg in args {
        match arg.as_str() {
            "--write" => write = true,
            "--force" => force = true,
            _ => {}
        }
    }

    let result = engine::discover();

    if write {
        let agents = engine::discovered_to_configs(&result);
        match ash::config::write_config(DEFAULT_CONFIG_FILENAME, &agents, None, force) {
            Ok(()) => println!("Generated {}", DEFAULT_CONFIG_FILENAME),
            Err(e) => {
                eprintln!("error: {}", e);
                return 1;
            }
        }
    }

    engine::print_discovery(&result);
    0
}

fn ensure_agents_registered(config_path: Option<&str>) {
    engine::register_defaults();

    let resolved = resolve_config_path(config_path);

    if let Some(ref path) = resolved {
        if path.exists() {
            match ash::config::read_config(path.to_str().unwrap()) {
                Ok((agents, telemetry_config)) => {
                    for config in agents {
                        let adapter = engine::from_config(&config);
                        engine::register(&config.name, adapter);
                    }
                    if let Some(tc) = telemetry_config {
                        init_telemetry(tc);
                    }
                    engine::print_agents_banner();
                    return;
                }
                Err(e) => eprintln!("warning: failed to read config: {}", e),
            }
        }

        if config_path.is_some() {
            eprintln!(
                "error: config file not found: {}",
                path.display()
            );
            return;
        }
    }

    let result = engine::discover_and_register();
    let summary = engine::discovery_summary(&result);
    eprintln!("{}", summary);
    engine::print_agents_banner();

    let dir = global_config_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("warning: failed to create {}: {}", dir.display(), e);
        return;
    }
    let path = dir.join(DEFAULT_CONFIG_FILENAME);
    let discovered_agents = engine::discovered_to_configs(&result);
    if let Err(e) = ash::config::write_config(path.to_str().unwrap(), &discovered_agents, None, false) {
        eprintln!("warning: failed to save config: {}", e);
    }
}

fn init_telemetry(config: ash::telemetry::config::TelemetryConfig) {
    if config.file.is_none() {
        return;
    }
    if let Err(e) = ash::telemetry::init(config) {
        eprintln!("warning: failed to init telemetry: {}", e);
    }
}

fn run() -> i32 {
    let _ = ash::log::init();
    info!("engine — starting up");

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "discover" {
        info!("engine — discover command");
        let code = cmd_discover(&args[2..]);
        ash::telemetry::shutdown();
        return code;
    }

    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("ash {}", env!("CARGO_PKG_VERSION"));
        ash::telemetry::shutdown();
        return 0;
    }

    let mut check_only = false;
    let mut dry_run = false;
    let mut continue_on_error = false;
    let mut yes_mode = false;
    let mut agent_spec: Option<String> = None;
    let mut config_override: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--check" | "-c" => check_only = true,
            "--dry-run" => dry_run = true,
            "--continue-on-error" | "-k" => continue_on_error = true,
            "--yes" | "-y" => yes_mode = true,
            "--agent" => {
                if i + 1 < args.len() {
                    i += 1;
                    agent_spec = Some(args[i].clone());
                }
            }
            "--config" => {
                if i + 1 < args.len() {
                    i += 1;
                    config_override = Some(args[i].clone());
                }
            }
            _ => positional.push(args[i].clone()),
        }
        i += 1;
    }

    let (default_agent, default_model) = parse_agent_spec(agent_spec.as_deref());

    if !positional.is_empty() {
        let path = std::path::Path::new(&positional[0]);
        if path.is_dir() {
            ensure_agents_registered(config_override.as_deref());
            let mut eval = Evaluator::new();
            let code = tree::run_tree(WalkConfig {
                root: path.to_path_buf(),
                dry_run: dry_run || check_only,
                continue_on_error,
                default_agent,
                default_model,
                parallel: if yes_mode { ParallelMode::Allow } else { ParallelMode::Prompt },
            }, &mut eval);
            ash::telemetry::shutdown();
            return code;
        }
    }

    let (src, script_args) = match get_input(&positional, config_override.as_deref(), &default_agent, &default_model) {
        Some(pair) => pair,
        None => {
            ash::telemetry::shutdown();
            return 1;
        }
    };

    let script = match parse_str(&src) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("parse error: {}", e);
            ash::telemetry::shutdown();
            return 1;
        }
    };

    if let Err(e) = validate_agents(&script, config_override.as_deref()) {
        eprintln!("error: {}", e);
        ash::telemetry::shutdown();
        return 1;
    }

    if check_only {
        println!("OK");
        ash::telemetry::shutdown();
        return 0;
    }

    ensure_agents_registered(config_override.as_deref());

    let mut eval = Evaluator::new();
    eval.set_args(script_args);
    if let Some(ref shebang) = script.shebang {
        eval.set_default_agent(&shebang.engine);
        if !shebang.model.is_empty() {
            eval.set_default_model(&shebang.model);
        }
    }

    if !positional.is_empty() {
        eval.source_path = Some(std::path::PathBuf::from(&positional[0]));
    }

    let exit_code = match eval.eval_script(&script) {
        Ok(()) => 0,
        Err(EvalError::Exit(ex)) => ex.code,
        Err(EvalError::Msg(e)) => {
            eprintln!("eval error: {}", e);
            3
        }
    };

    ash::telemetry::shutdown();
    exit_code
}

fn get_input(positional: &[String], config_override: Option<&str>, default_agent: &str, default_model: &str) -> Option<(String, Vec<String>)> {
    if !positional.is_empty() {
        let src = match std::fs::read_to_string(&positional[0]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error reading {}: {}", positional[0], e);
                return None;
            }
        };
        let args = positional[1..].to_vec();
        return Some((src, args));
    }

    #[cfg(feature = "repl")]
    if ash::repl::is_tty() {
        ensure_agents_registered(config_override);
        let mut eval = Evaluator::new();
        eval.set_default_agent(default_agent);
        if !default_model.is_empty() {
            eval.set_default_model(default_model);
        }
        let code = ash::repl::run_repl(&mut eval);
        ash::telemetry::shutdown();
        std::process::exit(code);
    }

    let mut input = String::new();
    match std::io::stdin().read_to_string(&mut input) {
        Ok(_) => Some((input, Vec::new())),
        Err(e) => {
            eprintln!("error reading stdin: {}", e);
            None
        }
    }
}

fn main() {
    let exit_code = run();
    std::process::exit(exit_code);
}

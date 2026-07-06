use std::collections::HashSet;
use std::io::Read;

use ash::ast::{Node, Script};
use ash::engine;
use ash::eval::{EvalError, Evaluator};
use ash::parser::parse_str;
use ash::tree::{self, WalkConfig};

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

fn validate_agents(script: &Script) -> Result<(), String> {
    let mut used_agents = HashSet::new();
    collect_agent_names(&script.body, &mut used_agents);

    if let Some(ref shebang) = script.shebang {
        used_agents.insert(&shebang.engine);
    }

    if used_agents.is_empty() {
        return Ok(());
    }

    let config_path = std::path::Path::new("ash-project.yaml");
    let config_exists = config_path.exists();

    if !config_exists {
        eprintln!(
            "warning: script references agent(s) ({}) but no ash-project.yaml found — \
             agent names will be resolved against the default registry",
            used_agents.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", ")
        );
        return Ok(());
    }

    let config_src = match std::fs::read_to_string(config_path) {
        Ok(s) => s,
        Err(e) => {
            return Err(format!("failed to read ash-project.yaml: {}", e));
        }
    };

    let configured = match config_src.lines().find(|l| l.trim() == "agents:") {
        Some(_) => {
            let mut names = HashSet::new();
            let mut in_agents = false;
            for line in config_src.lines() {
                let trimmed = line.trim();
                if trimmed == "agents:" {
                    in_agents = true;
                    continue;
                }
                if in_agents {
                    if trimmed.starts_with('-') {
                        continue;
                    }
                    if !trimmed.starts_with('#') && trimmed.contains(':') {
                        if let Some(name) = trimmed.split(':').next() {
                            names.insert(name.trim().to_string());
                        }
                    }
                    if !trimmed.starts_with(' ') && !trimmed.starts_with('\t') && trimmed != "" && !trimmed.starts_with('#') {
                        break;
                    }
                }
            }
            names
        }
        None => HashSet::new(),
    };

    let mut unknown: Vec<&str> = Vec::new();
    for name in &used_agents {
        if !configured.contains(*name) {
            unknown.push(name);
        }
    }

    if !unknown.is_empty() {
        return Err(format!(
            "agents not configured in ash-project.yaml: {}\n\
             hint: add them under the 'agents:' section, e.g.:\n\
             agents:\n\
             {}",
            unknown.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "),
            unknown.iter().map(|s| format!("  {}:\n    type: local-cli\n    driver: opencode\n", s)).collect::<Vec<_>>().join("")
        ));
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

fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    let mut check_only = false;
    let mut dry_run = false;
    let mut continue_on_error = false;
    let mut agent_spec: Option<String> = None;
    let mut positional: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--check" | "-c" => check_only = true,
            "--dry-run" => dry_run = true,
            "--continue-on-error" | "-k" => continue_on_error = true,
            "--agent" => {
                if i + 1 < args.len() {
                    i += 1;
                    agent_spec = Some(args[i].clone());
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
            engine::register_defaults();
            let mut eval = Evaluator::new();
            return tree::run_tree(WalkConfig {
                root: path.to_path_buf(),
                dry_run: dry_run || check_only,
                continue_on_error,
                default_agent,
                default_model,
            }, &mut eval);
        }
    }

    let src = if !positional.is_empty() {
        match std::fs::read_to_string(&positional[0]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error reading {}: {}", positional[0], e);
                return 1;
            }
        }
    } else {
        if ash::repl::is_tty() {
            engine::register_defaults();
            let mut eval = Evaluator::new();
            eval.set_default_agent(&default_agent);
            if !default_model.is_empty() {
                eval.set_default_model(&default_model);
            }
            return ash::repl::run_repl(&mut eval);
        }
        let mut input = String::new();
        match std::io::stdin().read_to_string(&mut input) {
            Ok(_) => input,
            Err(e) => {
                eprintln!("error reading stdin: {}", e);
                return 1;
            }
        }
    };

    let script = match parse_str(&src) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("parse error: {}", e);
            return 1;
        }
    };

    if let Err(e) = validate_agents(&script) {
        eprintln!("error: {}", e);
        return 1;
    }

    if check_only {
        println!("OK");
        return 0;
    }

    engine::register_defaults();

    let mut eval = Evaluator::new();
    if let Some(ref shebang) = script.shebang {
        eval.set_default_agent(&shebang.engine);
        if !shebang.model.is_empty() {
            eval.set_default_model(&shebang.model);
        }
    }

    match eval.eval_script(&script) {
        Ok(()) => 0,
        Err(EvalError::Exit(ex)) => ex.code,
        Err(EvalError::Msg(e)) => {
            eprintln!("eval error: {}", e);
            3
        }
    }
}

fn main() {
    let exit_code = run();
    std::process::exit(exit_code);
}

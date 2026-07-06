use std::io::{self, BufRead, IsTerminal, Write};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::ast::Node;
use crate::eval::{EvalError, Evaluator};
use crate::lexer;
use crate::parser::parse_str;
use crate::scope::Scope;
use crate::token::TokenKind;
use crate::value::Value;

pub fn is_tty() -> bool {
    io::stdin().is_terminal()
}

pub fn run_repl(eval: &mut Evaluator) -> i32 {
    println!("ash REPL. Type .help for commands, Ctrl-D to exit.");
    println!();

    if is_tty() {
        run_repl_tty(eval)
    } else {
        run_repl_piped(eval)
    }
}

fn run_repl_piped(eval: &mut Evaluator) -> i32 {
    let mut stdin = io::stdin().lock();

    loop {
        let accumulated = match read_input(&mut |p| read_line(&mut stdin, p), "ash> ") {
            Some(input) => input,
            None => {
                println!();
                return 0;
            }
        };

        if accumulated.is_empty() {
            continue;
        }

        let trimmed = accumulated.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('.') {
            if let Some(exit_code) = handle_dot_command(trimmed, eval) {
                return exit_code;
            }
            continue;
        }

        if eval_input(accumulated, eval) {
            return 0;
        }
    }
}

fn run_repl_tty(eval: &mut Evaluator) -> i32 {
    let mut rl = DefaultEditor::new().expect("failed to create line editor");

    loop {
        let accumulated = match read_input(&mut |p| read_line_tty(&mut rl, p), "ash> ") {
            Some(input) => input,
            None => {
                println!();
                return 0;
            }
        };

        if accumulated.is_empty() {
            continue;
        }

        let trimmed = accumulated.trim();
        if trimmed.is_empty() {
            continue;
        }

        rl.add_history_entry(&accumulated).ok();

        if trimmed.starts_with('.') {
            if let Some(exit_code) = handle_dot_command(trimmed, eval) {
                return exit_code;
            }
            continue;
        }

        if eval_input(accumulated, eval) {
            return 0;
        }
    }
}

fn eval_input(input: String, eval: &mut Evaluator) -> bool {
    let script = match parse_str(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {}", e);
            return false;
        }
    };

    for stmt in &script.body {
        match eval.eval_statement(stmt) {
            Ok(val) => {
                if is_expression_node(stmt) && val != Value::Nil {
                    println!("{}", val);
                }
            }
            Err(EvalError::Exit(_)) => {
                return true;
            }
            Err(EvalError::Msg(e)) => {
                eprintln!("error: {}", e);
            }
        }
    }

    false
}

fn is_expression_node(node: &Node) -> bool {
    matches!(
        node,
        Node::VarAssign(_)
            | Node::BinaryExpr(_)
            | Node::UnaryExpr(_)
            | Node::VarRef(_)
            | Node::StringLiteral(_)
            | Node::TextBlock(_)
            | Node::IntLiteral(_)
            | Node::FloatLiteral(_)
            | Node::BoolLiteral(_)
            | Node::CommandSubst(_)
            | Node::FnCall(_)
            | Node::ArrayLiteral(_)
            | Node::IndexExpr(_)
            | Node::GroupExpr(_)
            | Node::FilePath(_)
    )
}

fn read_input(read_line: &mut dyn FnMut(&str) -> Option<String>, prompt: &str) -> Option<String> {
    let mut accumulated: Vec<String> = Vec::new();

    let first_line = match read_line(prompt) {
        Some(l) if l.is_empty() => return Some(String::new()),
        Some(l) => l,
        None => return None,
    };
    accumulated.push(first_line);

    loop {
        let combined = accumulated.join("\n");

        match continuation_type(&combined) {
            Continuation::None => {
                return Some(combined);
            }
            Continuation::Backslash => {
                accumulated.pop();
                let stripped = combined.trim_end_matches('\\');
                accumulated.push(stripped.trim_end().to_string());

                let next = match read_line("... ") {
                    Some(l) if l.is_empty() => "".to_string(),
                    Some(l) => l,
                    None => return None,
                };

                let last = accumulated.last_mut().unwrap();
                last.push_str(&next);
            }
            Continuation::Brace => {
                let next = match read_line("... ") {
                    Some(l) if l.is_empty() => "".to_string(),
                    Some(l) => l,
                    None => return None,
                };

                accumulated.push(next);
            }
        }
    }
}

enum Continuation {
    None,
    Brace,
    Backslash,
}

fn continuation_type(input: &str) -> Continuation {
    if let Some(last_line) = input.lines().last() {
        let backslash_count = last_line.chars().rev().take_while(|&c| c == '\\').count();
        if backslash_count % 2 == 1 {
            return Continuation::Backslash;
        }
    }

    let tokens = match lexer::tokenize(input) {
        Ok(t) => t,
        Err(_) => return Continuation::None,
    };

    let mut open = 0usize;
    for tok in &tokens {
        if tok.kind == TokenKind::TkLBrace {
            open += 1;
        } else if tok.kind == TokenKind::TkRBrace {
            if open > 0 {
                open -= 1;
            }
        }
    }

    if open > 0 {
        Continuation::Brace
    } else {
        Continuation::None
    }
}

fn read_line(stdin: &mut io::StdinLock, prompt: &str) -> Option<String> {
    let mut stdout = io::stdout().lock();
    let _ = write!(stdout, "{}", prompt);
    let _ = stdout.flush();

    let mut line = String::new();
    match stdin.read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => Some(line.trim_end_matches('\n').trim_end_matches('\r').to_string()),
        Err(e) if e.kind() == io::ErrorKind::Interrupted => {
            println!();
            Some(String::new())
        }
        Err(_) => None,
    }
}

fn read_line_tty(rl: &mut DefaultEditor, prompt: &str) -> Option<String> {
    match rl.readline(prompt) {
        Ok(line) => Some(line),
        Err(ReadlineError::Eof) => None,
        Err(ReadlineError::Interrupted) => {
            println!();
            Some(String::new())
        }
        Err(_) => None,
    }
}

fn handle_dot_command(cmd: &str, eval: &mut Evaluator) -> Option<i32> {
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let name = parts[0];

    match name {
        ".help" => {
            println!();
            println!("ash REPL — interactive mode");
            println!();
            println!("Statements are evaluated one at a time. Variables persist across lines.");
            println!();
            println!("Commands:");
            println!("  .help        Show this help");
            println!("  .clear       Clear all variables and reset scope");
            println!("  .vars        List all variables and their values");
            println!();
            println!("Block constructs (if, for, while, fn, try, session, within,");
            println!("wait) support multi-line entry. Just start typing and press Enter —");
            println!("the REPL will show '... ' until the block is complete.");
            println!();
            println!("Use trailing \\ for manual line continuation:");
            println!("  ash> do \"long prompt \\");
            println!("  ... with examples\" with opencode");
            println!();
            println!("Press Ctrl-C to cancel multi-line input.");
            println!("Press Ctrl-D or type '.exit' to exit.");
            println!();
        }
        ".clear" => {
            eval.current_scope = Scope::new();
            eval.global_scope = eval.current_scope.clone();
            eval.session_depth = 0;
            eval.within_stack.clear();
            println!("Scope cleared.");
        }
        ".vars" => {
            let vars = eval.current_scope.lock().unwrap().get_all();
            if vars.is_empty() {
                println!("(no variables)");
            } else {
                let mut keys: Vec<&String> = vars.keys().collect();
                keys.sort();
                for key in keys {
                    let val = &vars[key];
                    match val {
                        Value::String(s) => println!("  {} = \"{}\"", key, s),
                        _ => println!("  {} = {}", key, val),
                    }
                }
            }
        }
        ".exit" => {
            return Some(0);
        }
        _ => {
            eprintln!("unknown command: {}. Type .help for available commands.", name);
        }
    }

    None
}

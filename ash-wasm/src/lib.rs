use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

use wasm_bindgen::prelude::*;

use ash::engine::{BrowserAdapter, ExecuteRequest, ExecuteResponse};
use ash::eval::{EvalError, Evaluator, SharedWriter};
use ash::scope::Scope;
use ash::value::Value;

fn agent_callback() -> &'static Mutex<Option<js_sys::Function>> {
    static CB: OnceLock<Mutex<Option<js_sys::Function>>> = OnceLock::new();
    CB.get_or_init(|| Mutex::new(None))
}

fn output_callback() -> &'static Mutex<Option<js_sys::Function>> {
    static CB: OnceLock<Mutex<Option<js_sys::Function>>> = OnceLock::new();
    CB.get_or_init(|| Mutex::new(None))
}

#[wasm_bindgen]
pub fn set_output_callback(cb: js_sys::Function) {
    *output_callback().lock().unwrap() = Some(cb);
}

fn captured_writer(buf: Arc<Mutex<Vec<u8>>>) -> SharedWriter {
    Arc::new(Mutex::new(Box::new(CaptureWriter(buf)) as Box<dyn Write + Send>))
}

struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        if let Some(cb) = output_callback().lock().unwrap().as_ref() {
            let s = String::from_utf8_lossy(buf);
            let this = JsValue::null();
            let arg = JsValue::from_str(&s);
            let _ = cb.call1(&this, &arg);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[wasm_bindgen]
pub fn register_agent_callback(cb: js_sys::Function) {
    *agent_callback().lock().unwrap() = Some(cb);
}

fn call_js_agent(prompt: &str, model: &str) -> ExecuteResponse {
    let cb = agent_callback().lock().unwrap().clone();
    match cb {
        Some(f) => {
            let this = JsValue::null();
            let prompt_arg = JsValue::from_str(prompt);
            let model_arg = JsValue::from_str(model);
            match f.call2(&this, &prompt_arg, &model_arg) {
                Ok(val) => {
                    let s = val.as_string().unwrap_or_default();
                    ExecuteResponse {
                        stdout: s,
                        stderr: String::new(),
                        exit_code: 0,
                    }
                }
                Err(e) => ExecuteResponse {
                    stdout: String::new(),
                    stderr: format!("JS agent error: {:?}", e),
                    exit_code: -1,
                },
            }
        }
        None => ExecuteResponse {
            stdout: String::new(),
            stderr: "no JS agent callback registered".to_string(),
            exit_code: -1,
        },
    }
}

#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    match ash::parser::parse_str(source) {
        Ok(script) => format!("{:#?}", script),
        Err(e) => format!("Parse error: {}", e),
    }
}

#[wasm_bindgen]
pub fn run(source: &str) -> String {
    let script = match ash::parser::parse_str(source) {
        Ok(s) => s,
        Err(e) => return format!("Parse error: {}", e),
    };

    ash::engine::register_defaults();

    let browser_adapter = Arc::new(BrowserAdapter::new(
        "pageagent",
        Arc::new(move |req: &ExecuteRequest| -> ExecuteResponse {
            call_js_agent(&req.prompt, &req.model)
        }),
    ));
    ash::engine::register("pageagent", browser_adapter.clone());
    ash::engine::register("browser", browser_adapter.clone());
    ash::engine::register("js-echo", browser_adapter);

    let out_buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let out = captured_writer(out_buf.clone());

    let mut eval = Evaluator::new();
    eval.stdout = out.clone();

    if let Some(ref shebang) = script.shebang {
        eval.set_default_agent(&shebang.engine);
        if !shebang.model.is_empty() {
            eval.set_default_model(&shebang.model);
        }
    }

    match eval.eval_script(&script) {
        Ok(()) => {}
        Err(EvalError::Exit(ex)) => {
            let msg = format!("\nexit code {}", ex.code);
            let _ = write!(out.lock().unwrap(), "{}", msg);
        }
        Err(EvalError::Msg(e)) => {
            let msg = format!("\nError: {}", e);
            let _ = write!(out.lock().unwrap(), "{}", msg);
        }
    }
    let bytes = out_buf.lock().unwrap().clone();
    String::from_utf8_lossy(&bytes).to_string()
}

fn repl_eval_state() -> &'static Mutex<Option<Evaluator>> {
    static EVAL: OnceLock<Mutex<Option<Evaluator>>> = OnceLock::new();
    EVAL.get_or_init(|| Mutex::new(None))
}

#[wasm_bindgen]
pub fn repl_init() {
    ash::engine::register_defaults();

    let browser_adapter = Arc::new(BrowserAdapter::new(
        "pageagent",
        Arc::new(move |req: &ExecuteRequest| -> ExecuteResponse {
            call_js_agent(&req.prompt, &req.model)
        }),
    ));
    ash::engine::register("pageagent", browser_adapter.clone());
    ash::engine::register("browser", browser_adapter.clone());
    ash::engine::register("js-echo", browser_adapter);

    let mut eval = Evaluator::new();
    let out_buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    eval.stdout = captured_writer(out_buf);
    *repl_eval_state().lock().unwrap() = Some(eval);
}

#[wasm_bindgen]
pub fn repl_eval(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut guard = repl_eval_state().lock().unwrap();
    let eval = match guard.as_mut() {
        Some(e) => e,
        None => return "Error: REPL not initialized. Call repl_init() first.".to_string(),
    };

    if trimmed.starts_with('.') {
        return handle_repl_dot_cmd(trimmed, eval);
    }

    let script = match ash::parser::parse_str(line) {
        Ok(s) => s,
        Err(e) => return format!("error: {}", e),
    };

    let out_buf: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    let saved = std::mem::replace(&mut eval.stdout, captured_writer(out_buf.clone()));

    for stmt in &script.body {
        match eval.eval_statement(stmt) {
            Ok(val) => {
                if is_repl_expr_node(stmt) && val != Value::Nil {
                    let _ = write!(out_buf.lock().unwrap(), "{}", val);
                }
            }
            Err(EvalError::Exit(_)) => {
                let _ = write!(out_buf.lock().unwrap(), "\n[exit REPL]");
            }
            Err(EvalError::Msg(e)) => {
                let _ = write!(out_buf.lock().unwrap(), "error: {}", e);
            }
        }
    }

    eval.stdout = saved;
    let bytes = out_buf.lock().unwrap().clone();
    String::from_utf8_lossy(&bytes).to_string()
}

fn handle_repl_dot_cmd(cmd: &str, eval: &mut Evaluator) -> String {
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    match parts[0] {
        ".clear" => {
            eval.current_scope = Scope::new();
            eval.global_scope = eval.current_scope.clone();
            eval.session_depth = 0;
            "Scope cleared.".to_string()
        }
        ".vars" => {
            let vars = eval.current_scope.lock().unwrap().get_all();
            if vars.is_empty() {
                "(no variables)".to_string()
            } else {
                let mut keys: Vec<&String> = vars.keys().collect();
                keys.sort();
                let mut out = String::new();
                for key in keys {
                    let val = &vars[key];
                    match val {
                        Value::String(s) => { out.push_str(&format!("{} = \"{}\"\n", key, s)); }
                        _ => { out.push_str(&format!("{} = {}\n", key, val)); }
                    }
                }
                out
            }
        }
        ".exit" => "\n[exit REPL]".to_string(),
        _ => format!("unknown command: {}. Type .help for available commands.", parts[0]),
    }
}

fn is_repl_expr_node(node: &ash::ast::Node) -> bool {
    use ash::ast::Node;
    matches!(
        node,
        Node::VarAssign(_)
            | Node::BinaryExpr(_)
            | Node::UnaryExpr(_)
            | Node::VarRef(_)
            | Node::StringLiteral(_)
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

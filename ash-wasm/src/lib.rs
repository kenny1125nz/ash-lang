use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

use wasm_bindgen::prelude::*;

use ash::engine::{BrowserAdapter, ExecuteRequest, ExecuteResponse};
use ash::eval::{EvalError, Evaluator, SharedWriter};

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

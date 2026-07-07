use std::sync::Arc;

use log::{debug, info};

use crate::lang::ast::*;
use crate::runtime::value::Value;
use crate::telemetry;

use super::{EvalError, Evaluator, ExitError};

impl Evaluator {
    pub(super) fn eval_agent_call(&mut self, n: &AgentCall) -> Result<Value, EvalError> {
        let prompt_val = self.eval_expr(&n.prompt)?;
        let prompt_str = match &prompt_val {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };

        let model_str = if let Some(ref m) = n.model {
            let val = self.eval_expr(m)?;
            match &val {
                Value::String(s) => s.clone(),
                other => format!("{}", other),
            }
        } else {
            self.default_model.clone()
        };

        let dir_str = if let Some(ref d) = n.dir {
            let val = self.eval_expr(d)?;
            let s = match &val {
                Value::String(s) => s.clone(),
                other => format!("{}", other),
            };
            let md = std::fs::metadata(&s)
                .map_err(|e| EvalError::Msg(format!("directory does not exist: {}: {}", s, e)))?;
            if !md.is_dir() {
                return Err(EvalError::Msg(format!("not a directory: {}", s)));
            }
            s
        } else {
            String::new()
        };

        let req = crate::engine::ExecuteRequest {
            prompt: prompt_str.clone(),
            model: model_str.clone(),
            dir: dir_str.clone(),
            session: self.session_depth > 0,
            yes: false,
        };

        let agent_name = n
            .agent
            .as_deref()
            .unwrap_or(&self.default_agent)
            .to_string();

        info!("agent — calling {} with agent {}", agent_name, agent_name);
        debug!("agent — prompt: {} chars", prompt_str.len());

        let child_ctx = self.telemetry_ctx.as_ref().map(|c| c.child());
        if let Some(ref ctx) = child_ctx {
            let mut payload = serde_json::json!({
                "agent": agent_name,
                "model": model_str,
                "request_len": prompt_str.len(),
            });
            if telemetry::capture_payload() {
                let obj = payload.as_object_mut().unwrap();
                obj.insert("request".to_string(), serde_json::Value::String(prompt_str.clone()));
            }
            telemetry::emit(
                ctx.clone(),
                telemetry::event::EventKind::AgentCall,
                payload,
            );
        }

        let start = std::time::Instant::now();
        let eng = crate::engine::get(&agent_name);
        let result = if let Some(eng) = eng {
            eng.execute(&req)
        } else {
            let mut cmd = std::process::Command::new(&agent_name);
            cmd.arg(&prompt_str);
            if !model_str.is_empty() {
                cmd.arg("--model");
                cmd.arg(&model_str);
            }
            if !dir_str.is_empty() {
                cmd.current_dir(&dir_str);
            }
            match cmd.output() {
                Ok(output) => crate::engine::ExecuteResponse {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code().unwrap_or(-1),
                },
                Err(e) => crate::engine::ExecuteResponse {
                    stdout: String::new(),
                    stderr: format!("failed to execute {}: {}", agent_name, e),
                    exit_code: -1,
                },
            }
        };
        let duration_ms = start.elapsed().as_millis() as u64;

        if let Some(ref ctx) = child_ctx {
            let mut payload = serde_json::json!({
                "agent": agent_name,
                "duration_ms": duration_ms,
                "exit_code": result.exit_code,
                "stdout_len": result.stdout.len(),
                "stderr_len": result.stderr.len(),
            });
            if telemetry::capture_payload() {
                let obj = payload.as_object_mut().unwrap();
                obj.insert("stdout".to_string(), serde_json::Value::String(result.stdout.clone()));
                obj.insert("stderr".to_string(), serde_json::Value::String(result.stderr.clone()));
            }
            telemetry::emit(
                ctx.clone(),
                telemetry::event::EventKind::AgentResponse,
                payload,
            );
        }

        {
            let mut scope = self.current_scope.lock().unwrap();
            scope.set_local("stdout", Value::String(result.stdout.clone()));
            scope.set_local("stderr", Value::String(result.stderr.clone()));
            scope.set_local("?", Value::Int(result.exit_code as i64));
        }

        if let Some(ref compact) = n.compact {
            if let Ok(compact_val) = self.eval_expr(compact) {
                let s = format!("{}", compact_val);
                if let Ok(d) = crate::runtime::compact::Directive::parse(&s) {
                    d.apply(&mut self.compact_config);
                }
            }
        }

        if result.exit_code != 0 {
            if let Some(ref ctx) = child_ctx {
                telemetry::emit(
                    ctx.clone(),
                    telemetry::event::EventKind::Error,
                    serde_json::json!({
                        "source": "agent_call",
                        "agent": agent_name,
                        "exit_code": result.exit_code,
                        "stderr": result.stderr,
                    }),
                );
            }
            return Err(EvalError::Exit(ExitError {
                code: result.exit_code,
            }));
        }

        Ok(Value::String(result.stdout))
    }

    fn eval_max_retries(&mut self, expr: &Node) -> Result<usize, EvalError> {
        let val = self.eval_expr(expr)?;
        match val {
            Value::Int(i) if i > 0 => Ok(i as usize),
            _ => Ok(1),
        }
    }

    pub(super) fn eval_binary_try(&mut self, n: &BinaryTry) -> Result<Value, EvalError> {
        let max = self.eval_max_retries(&n.max)?;
        self.push_scope();

        for _ in 0..max {
            let mut last_val = Value::Nil;
            let mut ok = true;

            match &*n.body {
                Node::Block(block) => {
                    for stmt in &block.statements {
                        match self.eval_statement(stmt) {
                            Ok(v) => last_val = v,
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                }
                other => {
                    match self.eval_expr(other) {
                        Ok(v) => last_val = v,
                        Err(_) => ok = false,
                    }
                }
            }

            if ok && last_val.is_truthy() {
                self.set_exit_code(0);
                self.pop_scope();
                return Ok(last_val);
            }

            self.set_exit_code(1);
        }

        if let Some(fail) = &n.fail {
            let result = self.eval_statement(fail)?;
            self.pop_scope();
            return Ok(result);
        }

        self.pop_scope();
        Ok(Value::Int(0))
    }

    pub(super) fn eval_eval_try(&mut self, n: &EvalTry) -> Result<Value, EvalError> {
        let max = self.eval_max_retries(&n.max)?;
        self.push_scope();

        for _ in 0..max {
            let mut body_ok = true;
            match &*n.body {
                Node::Block(block) => {
                    for stmt in &block.statements {
                        if let Err(e) = self.eval_statement(stmt) {
                            self.current_scope.lock().unwrap()
                                .set_local("error", Value::String(e.to_string()));
                            self.set_exit_code(1);
                            body_ok = false;
                            break;
                        }
                    }
                }
                other => {
                    if let Err(e) = self.eval_expr(other) {
                        self.current_scope.lock().unwrap()
                            .set_local("error", Value::String(e.to_string()));
                        self.set_exit_code(1);
                        body_ok = false;
                    }
                }
            }

            // Body failed — skip evaluator, run fail block, retry
            if !body_ok {
                // Before retrying, run the fail block for cleanup
                if let Some(fail) = &n.fail {
                    let _ = self.eval_statement(fail);
                }
                self.set_exit_code(0);
                continue;
            }

            let prev_exit = self.get_var("?").unwrap_or(Value::Int(0));

            let (eval_val, report) = {
                struct CapWriter {
                    buf: Arc<std::sync::Mutex<Vec<u8>>>,
                }
                impl std::io::Write for CapWriter {
                    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                        std::io::Write::write(&mut *self.buf.lock().unwrap(), buf)
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        Ok(())
                    }
                }
                let cap_arc = Arc::new(std::sync::Mutex::new(Vec::new()));
                let cap_writer: Box<dyn std::io::Write + Send> = Box::new(CapWriter { buf: cap_arc.clone() });
                let mut guard = self.stdout.lock().unwrap();
                let saved = std::mem::replace(&mut *guard, cap_writer);
                drop(guard);

                let mut eval_val = Value::Nil;
                let mut eval_ok = true;
                match &*n.eval {
                    Node::Block(block) => {
                        for stmt in &block.statements {
                            match self.eval_statement(stmt) {
                                Ok(v) => eval_val = v,
                                Err(_) => { eval_ok = false; break; }
                            }
                        }
                    }
                    other => {
                        match self.eval_expr(other) {
                            Ok(v) => eval_val = v,
                            Err(_) => { eval_ok = false; }
                        }
                    }
                }

                let mut guard = self.stdout.lock().unwrap();
                let _ = std::mem::replace(&mut *guard, saved);
                drop(guard);

                if !eval_ok {
                    self.current_scope.lock().unwrap()
                        .set_local("report", Value::String(String::new()));
                    // Evaluator errored — treat as exit 2+ (fail)
                    if let Some(fail) = &n.fail {
                        let _ = self.eval_statement(fail);
                    }
                    self.set_exit_code(0);
                    continue;
                }

                let report = String::from_utf8_lossy(&cap_arc.lock().unwrap()).to_string();
                (eval_val, report)
            };
            self.current_scope.lock().unwrap()
                .set_local("report", Value::String(report));

            let after_exit = self.get_var("?").unwrap_or(Value::Int(0));

            // 3-way routing based on evaluator exit code.
            // If the eval block explicitly set $? (prev_exit differs), use exit code:
            //   0 → accept, 1 → partial, 2+ → fail
            // If the eval block didn't set $? (pure expression), derive from truthiness:
            //   truthy → accept, falsy → partial

            #[derive(PartialEq)]
            enum Outcome { Accept, Partial, Fail }

            let outcome = if prev_exit != after_exit {
                match after_exit {
                    Value::Int(0) => Outcome::Accept,
                    Value::Int(1) => Outcome::Partial,
                    _ => Outcome::Fail,
                }
            } else {
                if eval_val.is_truthy() {
                    Outcome::Accept
                } else {
                    Outcome::Partial
                }
            };

            match outcome {
                Outcome::Accept => {
                    if let Some(accept) = &n.accept {
                        let r = self.eval_statement(accept)?;
                        self.pop_scope();
                        return Ok(r);
                    }
                    self.pop_scope();
                    return Ok(Value::Int(0));
                }
                Outcome::Partial => {
                    if let Some(partial) = &n.partial {
                        let _ = self.eval_statement(partial);
                    }
                }
                Outcome::Fail => {
                    if let Some(fail) = &n.fail {
                        let _ = self.eval_statement(fail);
                    }
                }
            }
        }

        // All retries exhausted — no special post-loop block;
        // Partial/fail already ran on each retry.
        self.pop_scope();
        Ok(Value::Int(0))
    }
}

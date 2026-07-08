use std::sync::Arc;

use log::{debug, info};
use regex::Regex;

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

    pub(super) fn eval_evaluate(&mut self, n: &Evaluate) -> Result<Value, EvalError> {
        let max = self.eval_max_retries(&n.upto)?;
        let threshold_val = self.eval_expr(&n.accept_by)?;
        let threshold = match threshold_val {
            Value::Int(i) => i,
            Value::Float(f) => f as i64,
            _ => {
                return Err(EvalError::Msg(format!(
                    "accept by threshold must be an integer, got {}",
                    threshold_val.type_name()
                )))
            }
        };

        let mut last_score: i64 = 0;
        let mut feedback = String::new();
        let mut last_evaluator_output = String::new();
        let mut accepted = false;

        'outer: for attempt in 1..=max {
            self.push_scope();
            self.set_var("_attempt", Value::Int(attempt as i64))
                .map_err(EvalError::Msg)?;
            self.set_var("_max_attempts", Value::Int(max as i64))
                .map_err(EvalError::Msg)?;
            self.set_var("_feedback", Value::String(feedback.clone()))
                .map_err(EvalError::Msg)?;

            // Evaluate body
            let body_result = match &*n.body {
                Node::Block(block) => {
                    let mut last = Value::Nil;
                    for stmt in &block.statements {
                        last = self.eval_statement(stmt)?;
                        if self.signal.is_some() {
                            break;
                        }
                    }
                    Ok(last)
                }
                other => self.eval_expr(other),
            };

            if let Err(e) = body_result {
                self.pop_scope();
                return Err(e);
            }

            // Dispatch to evaluator
            let (evaluator_output, score_val) = self.eval_evaluator(&n.evaluator, attempt)?;

            let score = match score_val {
                Value::Int(i) => i,
                _ => {
                    self.pop_scope();
                    return Err(EvalError::Msg(
                        "evaluator did not produce a valid integer score".to_string(),
                    ));
                }
            };

            if score < 0 || score > 100 {
                self.pop_scope();
                return Err(EvalError::Msg(format!(
                    "score out of range: {} (must be 0-100)",
                    score
                )));
            }

            last_evaluator_output = evaluator_output.clone();
            self.set_var("_evaluator_output", Value::String(evaluator_output.clone()))
                .map_err(EvalError::Msg)?;

            feedback = Self::extract_findings(&evaluator_output);

            if score >= threshold {
                accepted = true;
                last_score = score;
                self.pop_scope();
                break 'outer;
            }

            last_score = score;
            self.pop_scope();
        }

        self.set_var("score", Value::Int(last_score))
            .map_err(EvalError::Msg)?;
        self.set_var("accepted", Value::Bool(accepted))
            .map_err(EvalError::Msg)?;
        self.set_var("_evaluator_output", Value::String(last_evaluator_output))
            .map_err(EvalError::Msg)?;

        Ok(Value::Int(last_score))
    }

    fn eval_evaluator(
        &mut self,
        evaluator: &EvaluateEvaluator,
        attempt: usize,
    ) -> Result<(String, Value), EvalError> {
        match evaluator {
            EvaluateEvaluator::Agent(agent_call) => {
                let augmented = self.augment_evaluator_prompt(agent_call, attempt)?;
                let mut modified = agent_call.clone();
                modified.prompt = Box::new(Node::StringLiteral(StringLiteral {
                    pos: agent_call.prompt.pos().clone(),
                    value: augmented,
                    interps: vec![],
                }));
                let result = self.eval_agent_call(&modified)?;
                let output = format!("{}", result);
                let score = Self::parse_agent_score(&output)?;
                Ok((output, Value::Int(score)))
            }
            EvaluateEvaluator::FnCall(fn_call) => {
                let result = self.eval_fn_call(fn_call)?;
                let score_val = match result {
                    Value::Int(i) => i,
                    Value::Float(f) => f as i64,
                    ref other => {
                        return Err(EvalError::Msg(format!(
                            "function evaluator must return int or float, got {}",
                            other.type_name()
                        )))
                    }
                };
                let output = format!("{}", result);
                Ok((output, Value::Int(score_val)))
            }
            EvaluateEvaluator::Exec(exec) => {
                let result = self.eval_exec(exec)?;
                let output = format!("{}", result);
                let score = Self::parse_command_score(&output)?;
                Ok((output, Value::Int(score)))
            }
        }
    }

    fn augment_evaluator_prompt(
        &mut self,
        agent_call: &AgentCall,
        attempt: usize,
    ) -> Result<String, EvalError> {
        let prompt_val = self.eval_expr(&agent_call.prompt)?;
        let prompt_str = format!("{}", prompt_val);

        let scoring_template = r#"You are evaluating the quality of the following work.
Please provide a numerical score on a scale of 0 to 100.

Output your evaluation in exactly this format:

SCORE: <0-100 integer>
FINDINGS:
<actionable improvement feedback>

---

"#;

        let mut augmented = String::new();
        augmented.push_str(scoring_template);
        augmented.push_str(&prompt_str);

        // Add git diff context (read-only, for agent evaluators)
        if attempt > 1 {
            let diff = self.get_git_diff();
            if let Some(diff_str) = diff {
                augmented.push_str("\n\n--- Changes made ---\n");
                augmented.push_str(&diff_str);
            }
        }

        Ok(augmented)
    }

    fn get_git_diff(&self) -> Option<String> {
        let result = std::process::Command::new("git")
            .args(["diff", "--", ".", "':!.ash/'"])
            .output()
            .ok()?;
        if result.status.success() {
            let out = String::from_utf8_lossy(&result.stdout).to_string();
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        } else {
            None
        }
    }

    fn parse_agent_score(output: &str) -> Result<i64, EvalError> {
        let re =
            Regex::new(r"(?im)^SCORE:\s*(\d+)\s*$").unwrap();
        if let Some(caps) = re.captures(output) {
            let score: i64 = caps[1].parse().map_err(|_| {
                EvalError::Msg("failed to parse SCORE value as integer".to_string())
            })?;
            Ok(score)
        } else {
            Err(EvalError::Msg(
                "agent evaluator output missing SCORE: <N> line".to_string(),
            ))
        }
    }

    fn parse_command_score(output: &str) -> Result<i64, EvalError> {
        // First attempt: structured SCORE: format
        let re =
            Regex::new(r"(?im)^SCORE:\s*(\d+)\s*$").unwrap();
        if let Some(caps) = re.captures(output) {
            let score: i64 = caps[1].parse().map_err(|_| {
                EvalError::Msg("failed to parse SCORE value as integer".to_string())
            })?;
            return Ok(score);
        }

        // Fallback: first standalone integer line
        let line_re = Regex::new(r"^\s*(\d+)\s*$").unwrap();
        for line in output.lines() {
            if let Some(caps) = line_re.captures(line) {
                let score: i64 = caps[1].parse().unwrap_or(0);
                return Ok(score);
            }
        }

        Err(EvalError::Msg(
            "command evaluator output contains no parseable score".to_string(),
        ))
    }

    fn extract_findings(output: &str) -> String {
        let re = Regex::new(r"(?im)^[ \t]*FINDINGS:[ \t]*$").unwrap();
        if let Some(m) = re.find(output) {
            let after = &output[m.end()..];
            let trimmed = after.trim();
            if trimmed.is_empty() {
                output.to_string()
            } else {
                trimmed.to_string()
            }
        } else {
            output.to_string()
        }
    }
}

use std::io::Write;
use std::sync::Arc;

use log::{debug, info};
use regex::Regex;

use crate::lang::ast::*;
use crate::runtime::tree::{self, ParallelMode, WalkConfig};
use crate::runtime::value::Value;
use crate::telemetry;
use crate::util::lock_guard;

use super::{EvalError, Evaluator, ExitError};

impl Evaluator {
    pub(super) fn eval_agent_call(&mut self, n: &AgentCall) -> Result<Value, EvalError> {
        self.check_control_state()?;
        let mut agent_name = n
            .agent
            .as_deref()
            .unwrap_or(&self.default_agent)
            .to_string();

        // Print progress hint
        let hint = self.agent_hint.take().unwrap_or_else(|| {
            if let Node::FilePath(fp) = &*n.prompt {
                if let Ok(path_val) = self.eval_expr(&fp.path) {
                    let s = match &path_val {
                        Value::String(s) => s.clone(),
                        other => format!("{}", other),
                    };
                    format!("@{}", s)
                } else {
                    String::new()
                }
            } else if let Ok(prompt_val) = self.eval_expr(&n.prompt) {
                format!("{}", prompt_val)
            } else {
                String::new()
            }
        });
        if !hint.is_empty() {
            let _ = std::io::stderr().write_all(agent_name.as_bytes());
            let _ = std::io::stderr().write_all(b": ");
            let _ = std::io::stderr().write_all(hint.as_bytes());
            let _ = std::io::stderr().write_all(b"\n");
            let _ = std::io::stderr().flush();
        }

        // --- Resolve prompt and parse frontmatter ---
        let prompt_str;
        let model_str;
        let mut file_fm: Option<tree::Frontmatter> = None;

        if let Node::FilePath(fp) = &*n.prompt {
            let path_val = self.eval_expr(&fp.path)?;
            let path_str = match &path_val {
                Value::String(s) => s.clone(),
                other => {
                    return Err(EvalError::Msg(format!(
                        "file path must be a string, got {}",
                        other.type_name()
                    )))
                }
            };
            let resolved = self.resolve_include_path(&path_str);
            if let Ok(md) = std::fs::metadata(&resolved) {
                if md.is_dir() {
                    let resolved_str = resolved.to_string_lossy().to_string();
                    return self.eval_tree_dir(&resolved_str, n);
                }
            }
            // Read file and parse frontmatter
            let content = std::fs::read_to_string(&resolved)
                .map_err(|e| EvalError::Msg(format!("failed to read file '{}': {}", resolved.display(), e)))?;
            let (fm_opt, body) = tree::parse_frontmatter(&content);
            prompt_str = self.resolve_interpolations(body, &[])?;
            file_fm = fm_opt;

            // Apply frontmatter settings (as fallback if not explicitly set in `do` call)
            if let Some(ref fm) = file_fm {
                if n.agent.is_none() {
                    if let Some(ref a) = fm.agent {
                        agent_name = a.clone();
                    }
                }
            }

            model_str = if let Some(ref m) = n.model {
                let val = self.eval_expr(m)?;
                match &val {
                    Value::String(s) => s.clone(),
                    other => format!("{}", other),
                }
            } else if let Some(ref fm) = file_fm {
                fm.model.clone().unwrap_or_else(|| self.default_model.clone())
            } else {
                self.default_model.clone()
            };
        } else {
            let prompt_val = self.eval_expr(&n.prompt)?;
            prompt_str = match &prompt_val {
                Value::String(s) => s.clone(),
                other => format!("{}", other),
            };

            model_str = if let Some(ref m) = n.model {
                let val = self.eval_expr(m)?;
                match &val {
                    Value::String(s) => s.clone(),
                    other => format!("{}", other),
                }
            } else {
                self.default_model.clone()
            };
        }

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

        let start = if child_ctx.is_some() {
            Some(std::time::Instant::now())
        } else {
            None
        };
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
        let duration_ms = start.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0);

        self.check_control_state()?;
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
            let mut scope = lock_guard(&self.current_scope);
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
        } else if let Some(ref fm) = file_fm {
            if let Some(ref c) = fm.compact {
                if let Ok(d) = crate::runtime::compact::Directive::parse(c) {
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

    fn eval_tree_dir(&mut self, path: &str, n: &AgentCall) -> Result<Value, EvalError> {
        let agent_name = n
            .agent
            .as_deref()
            .unwrap_or(&self.default_agent)
            .to_string();

        let model_str = if let Some(ref m) = n.model {
            let val = self.eval_expr(m)?;
            match &val {
                Value::String(s) => s.clone(),
                other => format!("{}", other),
            }
        } else {
            self.default_model.clone()
        };

        let config = WalkConfig {
            root: std::path::PathBuf::from(path),
            dry_run: false,
            continue_on_error: false,
            default_agent: agent_name,
            default_model: model_str,
            parallel: ParallelMode::Prompt,
            session: false,
        };

        let exit_code = tree::run_tree(config, self);

        {
            let mut scope = lock_guard(&self.current_scope);
            scope.set_local("?", Value::Int(exit_code as i64));
            let summary = if exit_code == 0 {
                "all tasks passed"
            } else {
                "some tasks failed"
            };
            scope.set_local("stdout", Value::String(summary.to_string()));
        }

        if exit_code != 0 {
            Err(EvalError::Exit(ExitError { code: exit_code }))
        } else {
            Ok(Value::String("all tasks passed".to_string()))
        }
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
                            lock_guard(&self.current_scope)
                                .set_local("error", Value::String(e.to_string()));
                            self.set_exit_code(1);
                            body_ok = false;
                            break;
                        }
                    }
                }
                other => {
                    if let Err(e) = self.eval_expr(other) {
                        lock_guard(&self.current_scope)
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
                        std::io::Write::write(&mut *lock_guard(&self.buf), buf)
                    }
                    fn flush(&mut self) -> std::io::Result<()> {
                        Ok(())
                    }
                }
                let cap_arc = Arc::new(std::sync::Mutex::new(Vec::new()));
                let cap_writer: Box<dyn std::io::Write + Send> = Box::new(CapWriter { buf: cap_arc.clone() });
                let mut guard = lock_guard(&self.stdout);
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

                let mut guard = lock_guard(&self.stdout);
                let _ = std::mem::replace(&mut *guard, saved);
                drop(guard);

                if !eval_ok {
                    lock_guard(&self.current_scope)
                        .set_local("report", Value::String(String::new()));
                    // Evaluator errored — treat as exit 2+ (fail)
                    if let Some(fail) = &n.fail {
                        let _ = self.eval_statement(fail);
                    }
                    self.set_exit_code(0);
                    continue;
                }

                let report = String::from_utf8_lossy(&lock_guard(&cap_arc)).to_string();
                (eval_val, report)
            };
            lock_guard(&self.current_scope)
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
            self.set_var("_attempt", Value::Int(attempt as i64))?;
            self.set_var("_max_attempts", Value::Int(max as i64))?;
            self.set_var("_feedback", Value::String(feedback.clone()))?;

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

            {
                let _ = write!(std::io::stderr(), "score: {}/{}\n", score, threshold);
                let _ = std::io::stderr().flush();
            }

            last_evaluator_output = evaluator_output.clone();
            self.set_var("_evaluator_output", Value::String(evaluator_output.clone()))?;

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

        self.set_var("score", Value::Int(last_score))?;
        self.set_var("accepted", Value::Bool(accepted))?;
        self.set_var("_evaluator_output", Value::String(last_evaluator_output))?;

        Ok(Value::Int(last_score))
    }

    fn eval_evaluator(
        &mut self,
        evaluator: &EvaluateEvaluator,
        attempt: usize,
    ) -> Result<(String, Value), EvalError> {
        match evaluator {
            EvaluateEvaluator::Agent(agent_call) => {
                let hint = if let Node::FilePath(fp) = &*agent_call.prompt {
                    match self.eval_expr(&fp.path) {
                        Ok(Value::String(s)) => format!("@{}", s),
                        _ => String::new(),
                    }
                } else {
                    match self.eval_expr(&agent_call.prompt) {
                        Ok(val) => format!("{}", val),
                        _ => String::new(),
                    }
                };
                self.agent_hint = Some(hint);
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

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;
    use std::io::Write;
    use crate::lang::ast::*;
    use crate::engine::{self, EchoDriver, LocalCliAdapter};
    use crate::util::lock_guard;

    fn make_evaluator() -> Evaluator {
        Evaluator::new()
    }

    fn int_lit(v: i64) -> Node {
        Node::IntLiteral(IntLiteral {
            pos: Pos { line: 1, col: 1 },
            value: v,
        })
    }

    fn bool_lit(v: bool) -> Node {
        Node::BoolLiteral(BoolLiteral {
            pos: Pos { line: 1, col: 1 },
            value: v,
        })
    }

    fn string_lit(s: &str) -> Node {
        Node::StringLiteral(StringLiteral {
            pos: Pos { line: 1, col: 1 },
            value: s.to_string(),
            interps: vec![],
        })
    }

    fn var_ref(name: &str) -> Node {
        Node::VarRef(VarRef {
            pos: Pos { line: 1, col: 1 },
            name: name.to_string(),
        })
    }

    fn var_assign(name: &str, val: Node) -> Node {
        Node::VarAssign(VarAssign {
            pos: Pos { line: 1, col: 1 },
            name: name.to_string(),
            value: Box::new(val),
        })
    }

    fn bin_op(left: Node, op: &str, right: Node) -> Node {
        Node::BinaryExpr(BinaryExpr {
            pos: Pos { line: 1, col: 1 },
            left: Box::new(left),
            op: op.to_string(),
            right: Box::new(right),
        })
    }

    fn block(stmts: Vec<Node>) -> Node {
        Node::Block(Block {
            pos: Pos { line: 1, col: 1 },
            statements: stmts,
        })
    }

    fn print_node(msg: Node) -> Node {
        Node::Print(Print {
            pos: Pos { line: 1, col: 1 },
            message: Box::new(msg),
        })
    }

    fn exit_node(code: Node) -> Node {
        Node::Exit(Exit {
            pos: Pos { line: 1, col: 1 },
            code: Box::new(code),
        })
    }

    fn fn_decl_node(name: &str, params: Vec<String>, body: Node) -> Node {
        Node::FnDecl(FnDecl {
            pos: Pos { line: 1, col: 1 },
            name: name.to_string(),
            params,
            body: Box::new(body),
        })
    }

    fn return_node(val: Option<Node>) -> Node {
        Node::Return(Return {
            pos: Pos { line: 1, col: 1 },
            value: val.map(Box::new),
        })
    }

    fn shared_writer(buf: Arc<Mutex<Vec<u8>>>) -> SharedWriter {
        Arc::new(Mutex::new(Box::new(StdoutWriter(buf))))
    }

    struct StdoutWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for StdoutWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            lock_guard(&self.0).write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            lock_guard(&self.0).flush()
        }
    }

    fn agent_call_node(prompt: Node) -> Node {
        Node::AgentCall(AgentCall {
            pos: Pos { line: 1, col: 1 },
            prompt: Box::new(prompt),
            agent: None,
            subagent: String::new(),
            model: None,
            dir: None,
            compact: None,
        })
    }

    fn register_echo() {
        engine::register(
            "echo",
            Arc::new(LocalCliAdapter::new("echo", Arc::new(EchoDriver))),
        );
    }

    fn binary_try_node(body: Node, fail: Option<Node>, max: usize) -> Node {
        Node::BinaryTry(BinaryTry {
            pos: Pos { line: 1, col: 1 },
            body: Box::new(body),
            fail: fail.map(Box::new),
            max: Box::new(int_lit(max as i64)),
        })
    }

    fn eval_try_node(body: Node, eval: Node, accept: Option<Node>, partial: Option<Node>, fail: Option<Node>, max: usize) -> Node {
        Node::EvalTry(EvalTry {
            pos: Pos { line: 1, col: 1 },
            body: Box::new(body),
            eval: Box::new(eval),
            accept: accept.map(Box::new),
            partial: partial.map(Box::new),
            fail: fail.map(Box::new),
            max: Box::new(int_lit(max as i64)),
        })
    }

    fn compact_stmt_node(arg: Node) -> Node {
        Node::CompactStmt(CompactStmt {
            pos: Pos { line: 1, col: 1 },
            arg: Box::new(arg),
        })
    }

    fn dir_block_node(dir: Node, body: Node) -> Node {
        Node::DirBlock(DirBlock {
            pos: Pos { line: 1, col: 1 },
            dir: Box::new(dir),
            body: Box::new(body),
        })
    }

    fn wait_block_node(body: Option<Node>) -> Node {
        Node::WaitBlock(WaitBlock {
            pos: Pos { line: 1, col: 1 },
            body: body.map(Box::new),
        })
    }

    fn background_node(stmt: Node) -> Node {
        Node::Background(Background {
            pos: Pos { line: 1, col: 1 },
            stmt: Box::new(stmt),
        })
    }

    fn evaluate_node(
        body: Node,
        evaluator: EvaluateEvaluator,
        accept_by: Node,
        upto: Node,
    ) -> Node {
        Node::Evaluate(Evaluate {
            pos: Pos { line: 1, col: 1 },
            body: Box::new(body),
            evaluator,
            accept_by: Box::new(accept_by),
            upto: Box::new(upto),
        })
    }

    fn fn_evaluator(name: &str, args: Vec<Node>) -> EvaluateEvaluator {
        EvaluateEvaluator::FnCall(FnCall {
            pos: Pos { line: 1, col: 1 },
            name: name.to_string(),
            args,
        })
    }

    fn exec_evaluator(cmd: Node) -> EvaluateEvaluator {
        EvaluateEvaluator::Exec(Exec {
            pos: Pos { line: 1, col: 1 },
            cmd: Box::new(cmd),
        })
    }

    #[test]
    fn test_binary_try_success() {
        let mut ev = make_evaluator();
        let body = block(vec![var_assign("X", int_lit(42))]);
        let stmt = binary_try_node(body, None, 1);
        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(42));
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(0));
    }

    #[test]
    fn test_binary_try_fallback() {
        let mut ev = make_evaluator();
        let body = block(vec![exit_node(int_lit(1))]);
        let fail_body = block(vec![var_assign("Y", int_lit(99))]);
        let stmt = binary_try_node(body, Some(fail_body), 1);
        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(99));
        assert_eq!(ev.get_var("Y").unwrap(), Value::Int(99));
    }

    #[test]
    fn test_eval_try_accept() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let body = block(vec![var_assign("X", int_lit(1))]);
        let eval_body = block(vec![
            bin_op(var_ref("X"), "==", int_lit(1)),
        ]);
        let accept = block(vec![var_assign("accepted", bool_lit(true))]);
        let stmt = eval_try_node(body, eval_body, Some(accept), None, None, 1);
        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Bool(true));
        assert_eq!(ev.get_var("accepted").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_try_reject() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let body = block(vec![var_assign("X", int_lit(0))]);
        let eval_body = block(vec![
            bin_op(var_ref("X"), "==", int_lit(1)),
        ]);
        let accept = block(vec![var_assign("accepted", bool_lit(true))]);
        let fail_body = block(vec![var_assign("failed", bool_lit(true))]);
        let stmt = eval_try_node(body, eval_body, Some(accept), None, Some(fail_body), 1);
        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(0));
        assert!(ev.get_var("failed").is_err());
    }

    #[test]
    fn test_compact_truncate() {
        let mut ev = make_evaluator();
        let stmt = compact_stmt_node(string_lit("truncate 32000"));
        ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.compact_config.strategy, "truncate");
        assert_eq!(ev.compact_config.window, "32000");
    }

    #[test]
    fn test_compact_summarize() {
        let mut ev = make_evaluator();
        let stmt = compact_stmt_node(string_lit("summarize"));
        ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.compact_config.strategy, "summarize");
    }

    #[test]
    fn test_background_and_wait() {
        let mut ev = make_evaluator();
        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let bg_stmt = background_node(print_node(string_lit("bg task")));
        ev.eval_statement(&bg_stmt).unwrap();

        let wait_stmt = wait_block_node(None);
        ev.eval_statement(&wait_stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "bg task\n");
    }

    #[test]
    fn test_dir_block() {
        let mut ev = make_evaluator();
        let tmp = std::env::temp_dir();
        let stmt = dir_block_node(
            string_lit(tmp.to_str().unwrap()),
            var_assign("CWD", string_lit("changed")),
        );
        ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.get_var("CWD").unwrap(), Value::String("changed".to_string()));
    }

    #[test]
    fn test_agent_call_echo() {
        register_echo();
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let result = ev.eval_statement(&agent_call_node(string_lit("hello"))).unwrap();
        assert_eq!(result, Value::String("hello\n".to_string()));
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(0));
        assert_eq!(
            ev.get_var("stdout").unwrap(),
            Value::String("hello\n".to_string())
        );
    }

    #[test]
    fn test_agent_call_with_clause_overrides_default() {
        register_echo();
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));
        ev.default_agent = "nonexistent".to_string();

        let mut node = agent_call_node(string_lit("world"));
        if let Node::AgentCall(ref mut ac) = node {
            ac.agent = Some("echo".to_string());
        }

        let result = ev.eval_statement(&node).unwrap();
        assert_eq!(result, Value::String("world\n".to_string()));
    }

    #[test]
    fn test_agent_call_using_model() {
        register_echo();
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let model_node = string_lit("sonnet");
        let mut node = agent_call_node(string_lit("test"));
        if let Node::AgentCall(ref mut ac) = node {
            ac.model = Some(Box::new(model_node));
        }

        let _ = ev.eval_statement(&node).unwrap();
    }

    #[test]
    fn test_agent_call_variable_prompt() {
        register_echo();
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        ev.eval_statement(&var_assign("MSG", string_lit("hi from var")))
            .unwrap();

        let result = ev.eval_statement(&agent_call_node(var_ref("MSG"))).unwrap();
        assert_eq!(result, Value::String("hi from var\n".to_string()));
    }

    #[test]
    fn test_agent_call_session_flag() {
        register_echo();
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));
        ev.session_depth = 1;

        let _ = ev.eval_statement(&agent_call_node(string_lit("in session"))).unwrap();
    }

    #[test]
    fn test_agent_call_not_registered_spawns_fallback() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let result = ev.eval_statement(&agent_call_node(string_lit("test")));
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_fn_accept() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let fn_body = return_node(Some(int_lit(90)));
        ev.eval_statement(&fn_decl_node("return_score", vec![], fn_body))
            .unwrap();

        let stmt = evaluate_node(
            block(vec![var_assign("X", int_lit(1))]),
            fn_evaluator("return_score", vec![]),
            int_lit(85),
            int_lit(3),
        );

        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(90));
        assert_eq!(ev.get_var("score").unwrap(), Value::Int(90));
        assert_eq!(ev.get_var("accepted").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_evaluate_fn_exhaustion() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let fn_body = return_node(Some(int_lit(50)));
        ev.eval_statement(&fn_decl_node("low_score", vec![], fn_body))
            .unwrap();

        let stmt = evaluate_node(
            block(vec![var_assign("Y", int_lit(2))]),
            fn_evaluator("low_score", vec![]),
            int_lit(85),
            int_lit(3),
        );

        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(50));
        assert_eq!(ev.get_var("score").unwrap(), Value::Int(50));
        assert_eq!(ev.get_var("accepted").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_evaluate_loop_variables() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let fn_body = return_node(Some(var_ref("_attempt")));
        ev.eval_statement(&fn_decl_node("check_attempt", vec![], fn_body))
            .unwrap();

        let stmt = evaluate_node(
            block(vec![var_assign("CAPTURED_ATTEMPT", var_ref("_attempt"))]),
            fn_evaluator("check_attempt", vec![]),
            int_lit(100),
            int_lit(3),
        );

        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(3));
        assert_eq!(ev.get_var("score").unwrap(), Value::Int(3));
        assert_eq!(ev.get_var("accepted").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_evaluate_body_side_effects() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let fn_body = return_node(Some(int_lit(95)));
        ev.eval_statement(&fn_decl_node("score_sidefx", vec![], fn_body))
            .unwrap();

        let stmt = evaluate_node(
            block(vec![var_assign("SIDE", int_lit(42))]),
            fn_evaluator("score_sidefx", vec![]),
            int_lit(90),
            int_lit(2),
        );

        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(95));
        assert_eq!(ev.get_var("SIDE").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_evaluate_exec_evaluator() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let stmt = evaluate_node(
            block(vec![var_assign("Z", int_lit(3))]),
            exec_evaluator(string_lit("echo SCORE: 88")),
            int_lit(80),
            int_lit(2),
        );

        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Int(88));
        assert_eq!(ev.get_var("score").unwrap(), Value::Int(88));
        assert_eq!(ev.get_var("accepted").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_evaluate_extract_findings() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let output = "SCORE: 75\nFINDINGS:\nImprove variable naming\nAdd error handling\n";
        let cmd = format!("echo '{}'", output.replace('\'', "'\\''"));
        let stmt = evaluate_node(
            block(vec![]),
            exec_evaluator(string_lit(&cmd)),
            int_lit(80),
            int_lit(2),
        );

        let _result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.get_var("accepted").unwrap(), Value::Bool(false));
        let score = ev.get_var("score").unwrap();
        assert_eq!(score, Value::Int(75));
    }

    #[test]
    fn test_agent_call_with_dir_runs_tree() {
        register_echo();

        let dir = std::env::temp_dir().join(format!(
            "ash-dir-test-{}",
            std::sync::atomic::AtomicUsize::new(0).fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("01-task.md"),
            "---\nagent: echo\n---\n\nHello from directory task",
        )
        .unwrap();

        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let fp_path = Node::StringLiteral(StringLiteral {
            pos: Pos { line: 1, col: 1 },
            value: dir.to_str().unwrap().to_string(),
            interps: vec![],
        });
        let prompt = Node::FilePath(FilePath {
            pos: Pos { line: 1, col: 1 },
            path: Box::new(fp_path),
        });
        let mut call = agent_call_node(prompt);
        if let Node::AgentCall(ref mut ac) = call {
            ac.agent = Some("echo".to_string());
        }

        let result = ev.eval_statement(&call).unwrap();
        assert_eq!(result, Value::String("all tasks passed".to_string()));
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(0));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_agent_call_with_file_path_reads_file() {
        register_echo();

        let dir = std::env::temp_dir().join("ash-file-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("prompt.txt");
        std::fs::write(&file_path, "custom prompt content").unwrap();

        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let fp_path = Node::StringLiteral(StringLiteral {
            pos: Pos { line: 1, col: 1 },
            value: file_path.to_str().unwrap().to_string(),
            interps: vec![],
        });
        let prompt = Node::FilePath(FilePath {
            pos: Pos { line: 1, col: 1 },
            path: Box::new(fp_path),
        });
        let mut call = agent_call_node(prompt);
        if let Node::AgentCall(ref mut ac) = call {
            ac.agent = Some("echo".to_string());
        }

        let result = ev.eval_statement(&call).unwrap();
        assert_eq!(result, Value::String("custom prompt content\n".to_string()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}

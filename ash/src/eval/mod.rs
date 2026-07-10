pub mod agent;
pub mod conc;
pub mod control;
pub mod expr;
pub mod scope;

use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::util::lock_guard;

use log::info;

use crate::error::AshError;
use crate::lang::ast::*;
use crate::runtime::compact::Config as CompactConfig;
use crate::runtime::executor::Executor;
use crate::runtime::scope::{Scope, ScopeRef};
use crate::runtime::tree::{ExecError, TaskExecutor};
use crate::runtime::value::Value;
use crate::telemetry;

const DEFAULT_AGENT: &str = "echo";

pub type SharedWriter = Arc<Mutex<Box<dyn Write + Send>>>;

// --- Error types ---

#[derive(Debug)]
pub struct ExitError {
    pub code: i32,
}

impl fmt::Display for ExitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit code {}", self.code)
    }
}

#[derive(Debug)]
pub enum EvalError {
    Exit(ExitError),
    Msg(String),
}

impl From<String> for EvalError {
    fn from(s: String) -> Self {
        EvalError::Msg(s)
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::Exit(ex) => write!(f, "{}", ex),
            EvalError::Msg(s) => write!(f, "{}", s),
        }
    }
}

impl From<AshError> for EvalError {
    fn from(e: AshError) -> Self {
        match e {
            AshError::Msg(s) | AshError::Parse(s) | AshError::Eval(s) => EvalError::Msg(s),
            AshError::Io(e) => EvalError::Msg(e.to_string()),
        }
    }
}

// --- Flow signals ---

#[derive(Debug, Clone)]
pub enum SignalKind {
    Return,
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct FlowSignal {
    pub kind: SignalKind,
    pub value: Option<Value>,
}

// --- Evaluator ---

pub struct Evaluator {
    pub global_scope: ScopeRef,
    pub current_scope: ScopeRef,
    pub stdout: SharedWriter,
    pub stderr: SharedWriter,
    pub executor: Executor,
    pub compact_config: CompactConfig,
    pub signal: Option<FlowSignal>,
    pub bg_handles: Arc<Mutex<Vec<mpsc::Receiver<()>>>>,
    pub default_agent: String,
    pub default_model: String,
    pub session_depth: usize,
    pub within_stack: Vec<PathBuf>,
    pub source_path: Option<PathBuf>,
    pub telemetry_ctx: Option<telemetry::context::SpanContext>,
    pub(crate) script_args: Vec<String>,
    pub agent_hint: Option<String>,
}

impl Evaluator {
    pub fn new() -> Self {
        let scope = Scope::new();
        Evaluator {
            global_scope: scope.clone(),
            current_scope: scope,
            stdout: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
            stderr: Arc::new(Mutex::new(Box::new(std::io::stderr()))),
            executor: Executor::new(),
            compact_config: CompactConfig::new(),
            signal: None,
            bg_handles: Arc::new(Mutex::new(Vec::new())),
            default_agent: DEFAULT_AGENT.to_string(),
            default_model: String::new(),
            session_depth: 0,
            within_stack: Vec::new(),
            source_path: None,
            telemetry_ctx: None,
            script_args: Vec::new(),
            agent_hint: None,
        }
    }

    pub fn fork(&self) -> Evaluator {
        Evaluator {
            current_scope: self.current_scope.clone(),
            global_scope: self.global_scope.clone(),
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
            executor: Executor::new(),
            compact_config: self.compact_config.clone(),
            signal: self.signal.clone(),
            bg_handles: self.bg_handles.clone(),
            default_agent: self.default_agent.clone(),
            default_model: self.default_model.clone(),
            session_depth: 0,
            within_stack: Vec::new(),
            source_path: self.source_path.clone(),
            telemetry_ctx: None,
            script_args: self.script_args.clone(),
            agent_hint: None,
        }
    }

    pub fn set_default_agent(&mut self, name: &str) {
        self.default_agent = name.to_string();
    }

    pub fn set_default_model(&mut self, name: &str) {
        self.default_model = name.to_string();
    }

    pub fn set_args(&mut self, args: Vec<String>) {
        self.script_args = args;
    }

    // --- Public API ---

    pub fn eval_script(&mut self, script: &Script) -> Result<(), EvalError> {
        info!("engine — evaluating script ({} statements)", script.body.len());

        if self.telemetry_ctx.is_none() && telemetry::is_enabled() {
            self.telemetry_ctx = Some(telemetry::context::SpanContext::root());
        }

        if let Some(ref ctx) = self.telemetry_ctx {
            telemetry::emit(
                ctx.clone(),
                telemetry::event::EventKind::SessionStart,
                serde_json::json!({"script_len": script.body.len()}),
            );
        }

        {
            let mut scope = lock_guard(&self.global_scope);
            for (i, arg) in self.script_args.iter().enumerate() {
                scope.set_local(&(i + 1).to_string(), Value::String(arg.clone()));
            }
            scope.set_local("#", Value::Int(self.script_args.len() as i64));
        }

        let start = if self.telemetry_ctx.is_some() {
            Some(std::time::Instant::now())
        } else {
            None
        };
        let result = self.eval_statements(&script.body);
        let duration_ms = start.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0);

        if let Some(ref ctx) = self.telemetry_ctx {
            let exit_code = self.get_var("?").unwrap_or(crate::runtime::value::Value::Int(0));
            let code = match exit_code {
                crate::runtime::value::Value::Int(i) => i,
                _ => 0,
            };
            telemetry::emit(
                ctx.clone(),
                telemetry::event::EventKind::SessionEnd,
                serde_json::json!({
                    "duration_ms": duration_ms,
                    "exit_code": code,
                    "ok": result.is_ok(),
                }),
            );
        }

        result
    }

    pub fn eval_statements(&mut self, stmts: &[Node]) -> Result<(), EvalError> {
        for stmt in stmts {
            self.eval_statement(stmt)?;
            if self.signal.is_some() {
                break;
            }
        }
        Ok(())
    }

    // --- Statement dispatch ---

    pub fn eval_statement(&mut self, node: &Node) -> Result<Value, EvalError> {
        match node {
            Node::Print(n) => self.eval_print(n),
            Node::Exit(n) => self.eval_exit(n),
            Node::Exec(n) => self.eval_exec(n),
            Node::Env(n) => self.eval_env(n),
            Node::Block(n) => self.eval_block(n),
            Node::IfStmt(n) => self.eval_if(n),
            Node::ForStmt(n) => self.eval_for(n),
            Node::WhileStmt(n) => self.eval_while(n),
            Node::FnDecl(n) => self.eval_fn_decl(n),
            Node::Return(n) => self.eval_return(n),
            Node::Break(_) => self.eval_break(),
            Node::Continue(_) => self.eval_continue(),
            Node::AgentCall(n) => self.eval_agent_call(n),
            Node::BinaryTry(n) => self.eval_binary_try(n),
            Node::EvalTry(n) => self.eval_eval_try(n),
            Node::Evaluate(n) => self.eval_evaluate(n),
            Node::WaitBlock(n) => self.eval_wait(n),
            Node::Background(n) => self.eval_background(n),
            Node::Include(n) => self.eval_include(n),
            Node::DirBlock(n) => self.eval_dir_block(n),
            Node::CompactStmt(n) => self.eval_compact_stmt(n),
            Node::SessionBlock(n) => self.eval_session_block(n),
            Node::SessionToggle(n) => self.eval_session_toggle(n),
            Node::WithinToggle(n) => self.eval_within_toggle(n),
            Node::UseAgent(n) => self.eval_use_agent(n),
            _ => self.eval_expr(node),
        }
    }

    // --- Statements ---

    fn eval_print(&mut self, n: &Print) -> Result<Value, EvalError> {
        let val = self.eval_expr(&n.message)?;
        writeln!(lock_guard(&self.stdout), "{}", val)
            .map_err(|e| EvalError::Msg(e.to_string()))?;
        self.set_exit_code(0);
        Ok(val)
    }

    pub(super) fn eval_exec(&mut self, n: &Exec) -> Result<Value, EvalError> {
        let cmd_val = self.eval_expr(&n.cmd)?;
        let cmd_str = match &cmd_val {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };
        let cmd_str = self.resolve_exec_paths(&cmd_str);

        let child_ctx = self.telemetry_ctx.as_ref().map(|c| c.child());
        if let Some(ref ctx) = child_ctx {
            telemetry::emit(
                ctx.clone(),
                telemetry::event::EventKind::CommandExec,
                serde_json::json!({"cmd": cmd_str}),
            );
        }
        let start = if child_ctx.is_some() {
            Some(std::time::Instant::now())
        } else {
            None
        };
        {
            let _ = std::io::stderr().write_all(b"> ");
            let _ = std::io::stderr().write_all(cmd_str.as_bytes());
            let _ = std::io::stderr().write_all(b"\n");
            let _ = std::io::stderr().flush();
        }
        let result = self.executor.run_forwarded(&cmd_str)?;
        let duration_ms = start.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0);
        if let Some(ref ctx) = child_ctx {
            telemetry::emit(
                ctx.clone(),
                telemetry::event::EventKind::CommandExec,
                serde_json::json!({
                    "cmd": cmd_str,
                    "duration_ms": duration_ms,
                    "exit_code": result.exit_code,
                    "stdout_len": result.stdout.len(),
                    "stderr_len": result.stderr.len(),
                }),
            );
        }
        {
            let mut scope = lock_guard(&self.current_scope);
            scope.set_local("?", Value::Int(result.exit_code as i64));
            scope.set_local("stdout", Value::String(result.stdout.clone()));
            scope.set_local("stderr", Value::String(result.stderr.clone()));
        }
        Ok(Value::String(result.stdout))
    }

    fn eval_env(&mut self, n: &Env) -> Result<Value, EvalError> {
        let val = std::env::var(&n.key).unwrap_or_default();
        if !val.is_empty() {
            lock_guard(&self.current_scope)
                .set_local(&n.key, Value::String(val.clone()));
        }
        Ok(Value::String(val))
    }

    // --- Blocks ---

    fn eval_block(&mut self, n: &Block) -> Result<Value, EvalError> {
        let mut last_val = Value::Nil;
        for stmt in &n.statements {
            last_val = self.eval_statement(stmt)?;
            if self.signal.is_some() {
                break;
            }
        }
        Ok(last_val)
    }

    // --- Functions ---

    fn eval_fn_decl(&mut self, n: &FnDecl) -> Result<Value, EvalError> {
        self.current_scope
            .lock()
            .unwrap()
            .functions
            .insert(n.name.clone(), n.clone());
        Ok(Value::Nil)
    }

    // --- Dir block ---

    fn eval_dir_block(&mut self, n: &DirBlock) -> Result<Value, EvalError> {
        let dir_val = self.eval_expr(&n.dir)?;
        let dir_str = match &dir_val {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };

        let md = std::fs::metadata(&dir_str)
            .map_err(|e| EvalError::Msg(format!("directory does not exist: {}: {}", dir_str, e)))?;
        if !md.is_dir() {
            return Err(EvalError::Msg(format!("not a directory: {}", dir_str)));
        }

        let saved = std::env::current_dir()
            .map_err(|e| EvalError::Msg(format!("failed to get current dir: {}", e)))?;
        std::env::set_current_dir(&dir_str)
            .map_err(|e| EvalError::Msg(format!("failed to change dir: {}", e)))?;

        let result = self.eval_statement(&n.body);

        let _ = std::env::set_current_dir(&saved);
        result
    }

    // --- Compact statement ---

    fn eval_compact_stmt(&mut self, n: &CompactStmt) -> Result<Value, EvalError> {
        let arg_val = self.eval_expr(&n.arg)?;
        let arg_str = format!("{}", arg_val);

        if let Ok(d) = crate::runtime::compact::Directive::parse(&arg_str) {
            d.apply(&mut self.compact_config);
        }

        Ok(Value::Int(0))
    }

    fn eval_session_block(&mut self, n: &SessionBlock) -> Result<Value, EvalError> {
        if self.session_depth > 0 {
            return Err(EvalError::Msg(
                "nested session blocks are not allowed".to_string(),
            ));
        }

        self.session_depth += 1;
        let result = self.eval_statement(&n.body);
        self.session_depth -= 1;

        result
    }

    fn eval_session_toggle(&mut self, n: &SessionToggle) -> Result<Value, EvalError> {
        if n.active {
            if self.session_depth > 0 {
                return Err(EvalError::Msg(
                    "nested session blocks are not allowed".to_string(),
                ));
            }
            self.session_depth += 1;
        } else {
            if self.session_depth == 0 {
                return Err(EvalError::Msg(
                    "session end without matching begin".to_string(),
                ));
            }
            self.session_depth -= 1;
        }
        Ok(Value::Nil)
    }

    fn eval_within_toggle(&mut self, n: &WithinToggle) -> Result<Value, EvalError> {
        if n.active {
            let dir_val = self.eval_expr(n.path.as_ref().unwrap())?;
            let dir_str = match &dir_val {
                Value::String(s) => s.clone(),
                other => format!("{}", other),
            };

            let md = std::fs::metadata(&dir_str)
                .map_err(|e| EvalError::Msg(format!("directory does not exist: {}: {}", dir_str, e)))?;
            if !md.is_dir() {
                return Err(EvalError::Msg(format!("not a directory: {}", dir_str)));
            }

            let saved = std::env::current_dir()
                .map_err(|e| EvalError::Msg(format!("failed to get current dir: {}", e)))?;
            self.within_stack.push(saved);

            std::env::set_current_dir(&dir_str)
                .map_err(|e| EvalError::Msg(format!("failed to change dir: {}", e)))?;
        } else {
            match self.within_stack.pop() {
                Some(prev) => {
                    let _ = std::env::set_current_dir(&prev);
                }
                None => {
                    return Err(EvalError::Msg(
                        "within end without matching begin".to_string(),
                    ));
                }
            }
        }
        Ok(Value::Nil)
    }

    fn eval_use_agent(&mut self, n: &UseAgent) -> Result<Value, EvalError> {
        self.set_default_agent(&n.agent);
        self.set_exit_code(0);
        Ok(Value::Nil)
    }

    pub fn set_source_path(&mut self, path: Option<PathBuf>) {
        self.source_path = path;
    }

    fn resolve_include_path(&self, path: &str) -> PathBuf {
        let p = std::path::Path::new(path);
        if p.is_absolute() {
            return p.to_path_buf();
        }
        if let Some(ref sp) = self.source_path {
            return sp.parent().unwrap().join(path);
        }
        p.to_path_buf()
    }

    fn resolve_exec_paths(&self, cmd: &str) -> String {
        let mut out = String::with_capacity(cmd.len());
        let mut chars = cmd.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\'' => {
                    out.push(ch);
                    while let Some(&c) = chars.peek() {
                        if c == '\'' {
                            out.push(chars.next().unwrap());
                            break;
                        }
                        out.push(chars.next().unwrap());
                    }
                }
                '"' => {
                    out.push(ch);
                    while let Some(&c) = chars.peek() {
                        if c == '"' {
                            out.push(chars.next().unwrap());
                            break;
                        }
                        if c == '@' {
                            chars.next();
                            let mut path = String::new();
                            while let Some(&c2) = chars.peek() {
                                if c2 == '"' || c2 == '\'' || c2 == '`' || c2.is_whitespace() {
                                    break;
                                }
                                path.push(chars.next().unwrap());
                            }
                            if path.is_empty() {
                                out.push('@');
                            } else {
                                let resolved = self.resolve_include_path(&path);
                                out.push_str(&resolved.to_string_lossy());
                            }
                        } else {
                            out.push(chars.next().unwrap());
                        }
                    }
                }
                '`' => {
                    out.push(ch);
                    while let Some(&c) = chars.peek() {
                        if c == '`' {
                            out.push(chars.next().unwrap());
                            break;
                        }
                        out.push(chars.next().unwrap());
                    }
                }
                '@' => {
                    let mut path = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '\'' || c == '"' || c == '`' || c.is_whitespace() {
                            break;
                        }
                        path.push(chars.next().unwrap());
                    }
                    if path.is_empty() {
                        out.push('@');
                    } else {
                        let resolved = self.resolve_include_path(&path);
                        out.push_str(&resolved.to_string_lossy());
                    }
                }
                _ => out.push(ch),
            }
        }
        out
    }
}

impl TaskExecutor for Evaluator {
    fn fork(&self) -> Box<dyn TaskExecutor + Send> {
        Box::new(Evaluator::fork(self))
    }

    fn eval_script(&mut self, script: &Script) -> Result<(), ExecError> {
        Evaluator::eval_script(self, script).map_err(|e| match e {
            EvalError::Exit(ex) => ExecError::Exit(ex.code),
            EvalError::Msg(s) => ExecError::Msg(s),
        })
    }

    fn set_default_agent(&mut self, name: &str) {
        Evaluator::set_default_agent(self, name);
    }

    fn set_default_model(&mut self, name: &str) {
        Evaluator::set_default_model(self, name);
    }

    fn set_source_path(&mut self, path: Option<PathBuf>) {
        Evaluator::set_source_path(self, path);
    }

    fn current_scope(&self) -> ScopeRef {
        self.current_scope.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Arc;
    use crate::lang::ast::{Block, BinaryExpr, Env, IntLiteral, Pos, Print, StringLiteral, UseAgent, VarAssign, VarRef};

    fn make_evaluator() -> Evaluator {
        Evaluator::new()
    }

    fn int_lit(v: i64) -> Node {
        Node::IntLiteral(IntLiteral {
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

    fn env_node(key: &str) -> Node {
        Node::Env(Env {
            pos: Pos { line: 1, col: 1 },
            key: key.to_string(),
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

    #[test]
    fn test_print() {
        let mut ev = make_evaluator();
        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        ev.eval_statement(&print_node(string_lit("hello")))
            .unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "hello\n");
    }

    #[test]
    fn test_print_var() {
        let mut ev = make_evaluator();
        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        ev.eval_statement(&var_assign("X", int_lit(42)))
            .unwrap();
        ev.eval_statement(&print_node(var_ref("X")))
            .unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output.trim(), "42");
    }

    #[test]
    fn test_env() {
        let mut ev = make_evaluator();

        std::env::set_var("MASH_TEST_VAR", "mash_val");
        let result = ev.eval_statement(&env_node("MASH_TEST_VAR")).unwrap();
        assert_eq!(result, Value::String("mash_val".to_string()));
    }

    #[test]
    fn test_eval_script_order() {
        let mut ev = make_evaluator();

        let stmts = vec![
            var_assign("A", int_lit(1)),
            var_assign("B", bin_op(var_ref("A"), "+", int_lit(2))),
        ];
        ev.eval_statements(&stmts).unwrap();

        assert_eq!(ev.get_var("A").unwrap(), Value::Int(1));
        assert_eq!(ev.get_var("B").unwrap(), Value::Int(3));
    }

    #[test]
    fn test_block_scope() {
        let mut ev = make_evaluator();

        ev.eval_statement(&var_assign("X", int_lit(1)))
            .unwrap();
        ev.eval_statement(&block(vec![var_assign("X", int_lit(2))]))
            .unwrap();
        let x = ev.get_var("X").unwrap();
        assert_eq!(x, Value::Int(2));
    }

    #[test]
    fn test_empty_script() {
        let mut ev = make_evaluator();

        let result = ev.eval_statements(&[]);
        assert!(result.is_ok());

        let script = Script {
            shebang: None,
            compact: None,
            body: vec![],
        };
        let result = ev.eval_script(&script);
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_agent_sets_default() {
        let mut ev = make_evaluator();
        ev.default_agent = "echo".to_string();

        let stmt = Node::UseAgent(UseAgent {
            pos: Pos { line: 1, col: 1 },
            agent: "opencode".to_string(),
        });
        let result = ev.eval_statement(&stmt).unwrap();
        assert_eq!(result, Value::Nil);
        assert_eq!(ev.default_agent, "opencode");
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(0));
    }

    #[test]
    fn test_resolve_exec_paths_unquoted() {
        let mut ev = make_evaluator();
        ev.source_path = Some(std::path::PathBuf::from("/project/scripts/build.ash"));

        let result = ev.resolve_exec_paths("node @deploy.js");
        assert_eq!(result, "node /project/scripts/deploy.js");
    }

    #[test]
    fn test_resolve_exec_paths_double_quotes() {
        let mut ev = make_evaluator();
        ev.source_path = Some(std::path::PathBuf::from("/project/scripts/build.ash"));

        let result = ev.resolve_exec_paths("cat \"@data/file.txt\"");
        assert_eq!(result, "cat \"/project/scripts/data/file.txt\"");
    }

    #[test]
    fn test_resolve_exec_paths_single_quotes_preserved() {
        let mut ev = make_evaluator();
        ev.source_path = Some(std::path::PathBuf::from("/project/scripts/build.ash"));

        let result = ev.resolve_exec_paths("echo '@literal @path'");
        assert_eq!(result, "echo '@literal @path'");
    }

    #[test]
    fn test_resolve_exec_paths_absolute_passthrough() {
        let mut ev = make_evaluator();
        ev.source_path = Some(std::path::PathBuf::from("/project/scripts/build.ash"));

        let result = ev.resolve_exec_paths("cat @/etc/hostname");
        assert_eq!(result, "cat /etc/hostname");
    }

    #[test]
    fn test_resolve_exec_paths_no_at() {
        let mut ev = make_evaluator();
        ev.source_path = Some(std::path::PathBuf::from("/project/scripts/build.ash"));

        let result = ev.resolve_exec_paths("mkdir -p tmp && cp src/file.txt dest/");
        assert_eq!(result, "mkdir -p tmp && cp src/file.txt dest/");
    }

    #[test]
    fn test_resolve_exec_paths_backticks_preserved() {
        let mut ev = make_evaluator();
        ev.source_path = Some(std::path::PathBuf::from("/project/scripts/build.ash"));

        let result = ev.resolve_exec_paths("echo `cat @file.txt`");
        assert_eq!(result, "echo `cat @file.txt`");
    }

    #[test]
    fn test_resolve_exec_paths_multiple_at() {
        let mut ev = make_evaluator();
        ev.source_path = Some(std::path::PathBuf::from("/project/scripts/build.ash"));

        let result = ev.resolve_exec_paths("node @tool.js --input @data/in.json");
        assert_eq!(result, "node /project/scripts/tool.js --input /project/scripts/data/in.json");
    }

    #[test]
    fn test_resolve_exec_paths_no_source_path_strips_at() {
        let ev = make_evaluator();
        assert!(ev.source_path.is_none());

        let result = ev.resolve_exec_paths("node @deploy.js");
        assert_eq!(result, "node deploy.js");
    }
}

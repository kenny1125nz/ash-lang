pub mod agent;
pub mod conc;
pub mod expr;

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::ast::*;
use crate::compact::Config as CompactConfig;
use crate::executor::Executor;
use crate::scope::{Scope, ScopeRef};
use crate::value::Value;

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
    pub bg_handles: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
    pub default_agent: String,
    pub default_model: String,
    pub session_depth: usize,
    pub within_stack: Vec<PathBuf>,
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
        }
    }

    pub fn set_default_agent(&mut self, name: &str) {
        self.default_agent = name.to_string();
    }

    pub fn set_default_model(&mut self, name: &str) {
        self.default_model = name.to_string();
    }

    // --- Public API ---

    pub fn eval_script(&mut self, script: &Script) -> Result<(), EvalError> {
        self.eval_statements(&script.body)
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
            Node::Break(_) => {
                self.signal = Some(FlowSignal {
                    kind: SignalKind::Break,
                    value: None,
                });
                Ok(Value::Nil)
            }
            Node::Continue(_) => {
                self.signal = Some(FlowSignal {
                    kind: SignalKind::Continue,
                    value: None,
                });
                Ok(Value::Nil)
            }
            Node::AgentCall(n) => self.eval_agent_call(n),
            Node::BinaryTry(n) => self.eval_binary_try(n),
            Node::EvalTry(n) => self.eval_eval_try(n),
            Node::WaitBlock(n) => self.eval_wait(n),
            Node::Background(n) => self.eval_background(n),
            Node::Include(n) => self.eval_include(n),
            Node::DirBlock(n) => self.eval_dir_block(n),
            Node::CompactStmt(n) => self.eval_compact_stmt(n),
            Node::SessionBlock(n) => self.eval_session_block(n),
            Node::SessionToggle(n) => self.eval_session_toggle(n),
            Node::WithinToggle(n) => self.eval_within_toggle(n),
            _ => self.eval_expr(node),
        }
    }

    // --- Statements ---

    fn eval_print(&mut self, n: &Print) -> Result<Value, EvalError> {
        let val = self.eval_expr(&n.message)?;
        writeln!(self.stdout.lock().unwrap(), "{}", val)
            .map_err(|e| EvalError::Msg(e.to_string()))?;
        self.set_exit_code(0);
        Ok(val)
    }

    fn eval_exit(&mut self, n: &Exit) -> Result<Value, EvalError> {
        let code_val = self.eval_expr(&n.code)?;
        let code = match &code_val {
            Value::Int(i) => *i as i32,
            Value::Float(f) => *f as i32,
            _ => 0,
        };
        self.set_exit_code(code);
        Err(EvalError::Exit(ExitError { code }))
    }

    fn eval_exec(&mut self, n: &Exec) -> Result<Value, EvalError> {
        let cmd_val = self.eval_expr(&n.cmd)?;
        let cmd_str = match &cmd_val {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };
        let result = self.executor.run(&cmd_str)?;
        {
            let mut scope = self.current_scope.lock().unwrap();
            scope.set_local("?", Value::Int(result.exit_code as i64));
            scope.set_local("stdout", Value::String(result.stdout.clone()));
            scope.set_local("stderr", Value::String(result.stderr.clone()));
        }
        Ok(Value::String(result.stdout))
    }

    fn eval_env(&mut self, n: &Env) -> Result<Value, EvalError> {
        let val = std::env::var(&n.key).unwrap_or_default();
        if !val.is_empty() {
            self.current_scope
                .lock()
                .unwrap()
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

    // --- Control flow ---

    fn eval_if(&mut self, n: &IfStmt) -> Result<Value, EvalError> {
        let cond = self.eval_expr(&n.cond)?;
        if cond.is_truthy() {
            return self.eval_statement(&n.body);
        }
        for else_if in &n.else_ifs {
            let ec = self.eval_expr(&else_if.cond)?;
            if ec.is_truthy() {
                return self.eval_statement(&else_if.body);
            }
        }
        if let Some(else_body) = &n.else_body {
            self.eval_statement(else_body)
        } else {
            Ok(Value::Nil)
        }
    }

    fn eval_for(&mut self, n: &ForStmt) -> Result<Value, EvalError> {
        let list_val = self.eval_expr(&n.list)?;
        let items: Vec<Value> = match &list_val {
            Value::Array(arr) => arr.clone(),
            other => {
                let s = format!("{}", other);
                s.split('\n').map(|s| Value::String(s.to_string())).collect()
            }
        };
        let mut last_val = Value::Nil;
        for item in items {
            self.set_var(&n.var, item)?;
            last_val = self.eval_statement(&n.body)?;
            if let Some(signal) = self.signal.take() {
                match signal.kind {
                    SignalKind::Break => break,
                    SignalKind::Continue => {}
                    SignalKind::Return => {
                        self.signal = Some(signal);
                        break;
                    }
                }
            }
        }
        Ok(last_val)
    }

    fn eval_while(&mut self, n: &WhileStmt) -> Result<Value, EvalError> {
        let mut last_val = Value::Nil;
        loop {
            let cond = self.eval_expr(&n.cond)?;
            if !cond.is_truthy() {
                break;
            }
            last_val = self.eval_statement(&n.body)?;
            if let Some(signal) = self.signal.take() {
                match signal.kind {
                    SignalKind::Break => break,
                    SignalKind::Continue => {}
                    SignalKind::Return => {
                        self.signal = Some(signal);
                        break;
                    }
                }
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

    // --- Return / flow ---

    fn eval_return(&mut self, n: &Return) -> Result<Value, EvalError> {
        let value = if let Some(ref expr) = n.value {
            self.eval_expr(expr)?
        } else {
            Value::Nil
        };
        self.set_exit_code(0);
        self.signal = Some(FlowSignal {
            kind: SignalKind::Return,
            value: Some(value.clone()),
        });
        Ok(value)
    }

    // --- Scope helpers ---

    pub fn push_scope(&mut self) {
        let new_scope = Scope::with_parent(self.current_scope.clone());
        self.current_scope = new_scope;
    }

    pub fn pop_scope(&mut self) {
        let parent = self.current_scope.lock().unwrap().parent.clone();
        if let Some(p) = parent {
            self.current_scope = p;
        }
    }

    pub fn get_var(&self, name: &str) -> Result<Value, String> {
        self.current_scope
            .lock()
            .unwrap()
            .get(name)
            .ok_or_else(|| format!("undefined variable: {}", name))
    }

    pub fn set_var(&mut self, name: &str, value: Value) -> Result<(), String> {
        self.current_scope.lock().unwrap().set(name, value);
        Ok(())
    }

    pub fn set_exit_code(&mut self, code: i32) {
        self.current_scope
            .lock()
            .unwrap()
            .set_local("?", Value::Int(code as i64));
    }

    // --- Include ---

    fn eval_include(&mut self, n: &Include) -> Result<Value, EvalError> {
        let path_val = self.eval_expr(&n.path)?;
        let path = match &path_val {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };

        let src = fs::read_to_string(&path)
            .map_err(|e| EvalError::Msg(format!("include: failed to read '{}': {}", path, e)))?;

        let script = crate::parser::parse_str(&src)
            .map_err(|e| EvalError::Msg(format!("include: parse error: {}", e)))?;

        for stmt in script.body {
            self.eval_statement(&stmt)?;
        }

        self.set_exit_code(0);
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

        if let Ok(d) = crate::compact::Directive::parse(&arg_str) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;

    use crate::ast::{
        ArrayLiteral, Background, BinaryExpr, BinaryTry, BoolLiteral, Break, CompactStmt, Continue,
        DirBlock, ElseIf, Env, EvalTry, Exit, FnCall, FnDecl, ForStmt, IfStmt, Include,
        IntLiteral, Pos, Print, Return, StringLiteral, UnaryExpr, VarAssign, VarRef, WaitBlock,
        WhileStmt,
    };

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

    fn env_node(key: &str) -> Node {
        Node::Env(Env {
            pos: Pos { line: 1, col: 1 },
            key: key.to_string(),
        })
    }

    fn if_node(cond: Node, body: Node, else_ifs: Vec<ElseIf>, else_body: Option<Node>) -> Node {
        Node::IfStmt(IfStmt {
            pos: Pos { line: 1, col: 1 },
            cond: Box::new(cond),
            body: Box::new(body),
            else_ifs,
            else_body: else_body.map(Box::new),
        })
    }

    fn else_if_node(cond: Node, body: Node) -> ElseIf {
        ElseIf {
            pos: Pos { line: 1, col: 1 },
            cond: Box::new(cond),
            body: Box::new(body),
        }
    }

    fn for_node(var: &str, list: Node, body: Node) -> Node {
        Node::ForStmt(ForStmt {
            pos: Pos { line: 1, col: 1 },
            var: var.to_string(),
            list: Box::new(list),
            body: Box::new(body),
        })
    }

    fn while_node(cond: Node, body: Node) -> Node {
        Node::WhileStmt(WhileStmt {
            pos: Pos { line: 1, col: 1 },
            cond: Box::new(cond),
            body: Box::new(body),
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

    fn fn_call_node(name: &str, args: Vec<Node>) -> Node {
        Node::FnCall(FnCall {
            pos: Pos { line: 1, col: 1 },
            name: name.to_string(),
            args,
        })
    }

    fn return_node(val: Option<Node>) -> Node {
        Node::Return(Return {
            pos: Pos { line: 1, col: 1 },
            value: val.map(Box::new),
        })
    }

    fn break_node() -> Node {
        Node::Break(Break {
            pos: Pos { line: 1, col: 1 },
        })
    }

    fn continue_node() -> Node {
        Node::Continue(Continue {
            pos: Pos { line: 1, col: 1 },
        })
    }

    // ===== Existing expression tests =====

    #[test]
    fn test_expr_arithmetic() {
        let mut ev = make_evaluator();

        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(1), "+", int_lit(2)))
                .unwrap(),
            Value::Int(3)
        );

        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(7), "/", int_lit(2)))
                .unwrap(),
            Value::Float(3.5)
        );

        assert_eq!(
            ev.eval_expr(&Node::UnaryExpr(UnaryExpr {
                pos: Pos { line: 1, col: 1 },
                op: "-".to_string(),
                right: Box::new(int_lit(5)),
            }))
            .unwrap(),
            Value::Int(-5)
        );

        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(1), "+", bin_op(int_lit(2), "*", int_lit(3))))
                .unwrap(),
            Value::Int(7)
        );

        let grouped = Node::GroupExpr(GroupExpr {
            pos: Pos { line: 1, col: 1 },
            inner: Box::new(bin_op(int_lit(1), "+", int_lit(2))),
        });
        assert_eq!(
            ev.eval_expr(&bin_op(grouped, "*", int_lit(3)))
                .unwrap(),
            Value::Int(9)
        );

        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(10), "%", int_lit(3)))
                .unwrap(),
            Value::Int(1)
        );
    }

    #[test]
    fn test_expr_comparison() {
        let mut ev = make_evaluator();

        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(5), "==", int_lit(5)))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(3), "!=", int_lit(4)))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(5), ">", int_lit(3)))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(3), "<", int_lit(5)))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(5), ">=", int_lit(5)))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ev.eval_expr(&bin_op(int_lit(3), "<=", int_lit(4)))
                .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_expr_boolean() {
        let mut ev = make_evaluator();

        assert_eq!(
            ev.eval_expr(&bin_op(bool_lit(true), "and", bool_lit(true)))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ev.eval_expr(&bin_op(bool_lit(true), "or", bool_lit(false)))
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            ev.eval_expr(&Node::UnaryExpr(UnaryExpr {
                pos: Pos { line: 1, col: 1 },
                op: "not".to_string(),
                right: Box::new(bool_lit(false)),
            }))
            .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_expr_full() {
        let mut ev = make_evaluator();

        let ten_plus_twenty = bin_op(int_lit(10), "+", int_lit(20));
        let grouped = Node::GroupExpr(GroupExpr {
            pos: Pos { line: 1, col: 1 },
            inner: Box::new(ten_plus_twenty),
        });
        let gt = bin_op(grouped, ">", int_lit(5));
        let expr = bin_op(gt, "and", bool_lit(true));

        assert_eq!(ev.eval_expr(&expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_string_concat() {
        let mut ev = make_evaluator();

        assert_eq!(
            ev.eval_expr(&bin_op(string_lit("hello "), "+", string_lit("world")))
                .unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_var_assign_and_ref() {
        let mut ev = make_evaluator();

        assert_eq!(
            ev.eval_expr(&var_assign("X", int_lit(42)))
                .unwrap(),
            Value::Int(42)
        );

        assert_eq!(
            ev.eval_expr(&var_assign("Y", var_ref("X")))
                .unwrap(),
            Value::Int(42)
        );

        assert_eq!(ev.eval_expr(&var_ref("Y")).unwrap(), Value::Int(42));
    }

    #[test]
    fn test_var_ref_undefined() {
        let mut ev = make_evaluator();
        let result = ev.eval_expr(&var_ref("UNDEFINED_VAR"));
        assert!(result.is_err());
    }

    #[test]
    fn test_string_interpolation() {
        let mut ev = make_evaluator();

        ev.eval_expr(&var_assign("VAR", string_lit("hello")))
            .unwrap();

        let interp = Node::StringLiteral(StringLiteral {
            pos: Pos { line: 1, col: 1 },
            value: String::new(),
            interps: vec![InterpSpan {
                pos: Pos { line: 1, col: 1 },
                typ: InterpType::Var("VAR".to_string()),
            }],
        });
        let result = ev.eval_expr(&interp).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_string_interpolation_escape() {
        let mut ev = make_evaluator();

        let s = string_lit("$VAR");
        let result = ev.eval_expr(&s).unwrap();
        assert_eq!(result, Value::String("$VAR".to_string()));

        let escaped = Node::StringLiteral(StringLiteral {
            pos: Pos { line: 1, col: 1 },
            value: "\\$".to_string(),
            interps: vec![],
        });
        let result = ev.eval_expr(&escaped).unwrap();
        assert_eq!(result, Value::String("$".to_string()));
    }

    #[test]
    fn test_string_interpolation_with_expr() {
        let mut ev = make_evaluator();

        let arr = Node::ArrayLiteral(ArrayLiteral {
            pos: Pos { line: 1, col: 1 },
            elements: vec![string_lit("a"), string_lit("b")],
        });

        ev.eval_expr(&var_assign("ITEMS", arr)).unwrap();

        let s = string_lit("count: ${len(ITEMS)}");
        let result = ev.eval_expr(&s).unwrap();
        assert_eq!(result, Value::String("count: 2".to_string()));
    }

    #[test]
    fn test_scope_set_get() {
        let mut ev = make_evaluator();

        ev.set_var("X", Value::Int(10)).unwrap();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(10));
    }

    #[test]
    fn test_scope_shadow() {
        let mut ev = make_evaluator();

        ev.set_var("X", Value::Int(1)).unwrap();
        ev.push_scope();
        ev.current_scope
            .lock()
            .unwrap()
            .set_local("X", Value::Int(2));
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(2));
        ev.pop_scope();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_reassign_nearest() {
        let mut ev = make_evaluator();

        ev.set_var("X", Value::Int(1)).unwrap();
        ev.push_scope();
        ev.set_var("X", Value::Int(2)).unwrap();
        ev.pop_scope();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_reassign_local() {
        let mut ev = make_evaluator();

        ev.set_var("X", Value::Int(1)).unwrap();
        ev.push_scope();
        ev.current_scope
            .lock()
            .unwrap()
            .set_local("X", Value::Int(10));
        ev.set_var("X", Value::Int(20)).unwrap();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(20));
        ev.pop_scope();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_array_literal_and_index() {
        let mut ev = make_evaluator();

        let arr = Node::ArrayLiteral(ArrayLiteral {
            pos: Pos { line: 1, col: 1 },
            elements: vec![int_lit(10), int_lit(20), int_lit(30)],
        });
        let result = ev.eval_expr(&arr).unwrap();
        assert_eq!(
            result,
            Value::Array(vec![Value::Int(10), Value::Int(20), Value::Int(30)])
        );

        let idx = Node::IndexExpr(IndexExpr {
            pos: Pos { line: 1, col: 1 },
            object: Box::new(arr),
            index: Box::new(int_lit(1)),
        });
        assert_eq!(ev.eval_expr(&idx).unwrap(), Value::Int(20));
    }

    #[test]
    fn test_len_builtin() {
        let mut ev = make_evaluator();

        let call = Node::FnCall(FnCall {
            pos: Pos { line: 1, col: 1 },
            name: "len".to_string(),
            args: vec![string_lit("hello")],
        });
        assert_eq!(ev.eval_expr(&call).unwrap(), Value::Int(5));

        let a = Node::ArrayLiteral(ArrayLiteral {
            pos: Pos { line: 1, col: 1 },
            elements: vec![int_lit(1), int_lit(2), int_lit(3)],
        });
        let call2 = Node::FnCall(FnCall {
            pos: Pos { line: 1, col: 1 },
            name: "len".to_string(),
            args: vec![a],
        });
        assert_eq!(ev.eval_expr(&call2).unwrap(), Value::Int(3));
    }

    #[test]
    fn test_exit_code() {
        let mut ev = make_evaluator();

        ev.set_exit_code(42);
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(42));
    }

    // ===== Statement tests =====

    fn shared_writer(buf: Arc<Mutex<Vec<u8>>>) -> SharedWriter {
        Arc::new(Mutex::new(Box::new(StdoutWriter(buf))))
    }

    struct StdoutWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for StdoutWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }

    #[test]
    fn test_print() {
        let mut ev = make_evaluator();
        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        ev.eval_statement(&print_node(string_lit("hello")))
            .unwrap();

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
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

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output.trim(), "42");
    }

    #[test]
    fn test_exit() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(Mutex::new(Box::new(std::io::sink())));

        let result = ev.eval_statement(&exit_node(int_lit(42)));
        match result {
            Err(EvalError::Exit(ex)) => assert_eq!(ex.code, 42),
            other => panic!("expected Exit error, got {:?}", other),
        }
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(42));
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

    // --- If/for/while ---

    #[test]
    fn test_if_true() {
        let mut ev = make_evaluator();

        let stmt = if_node(
            bool_lit(true),
            var_assign("X", int_lit(1)),
            vec![],
            None,
        );
        ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_if_false() {
        let mut ev = make_evaluator();

        let stmt = if_node(
            bool_lit(false),
            var_assign("X", int_lit(1)),
            vec![],
            None,
        );
        ev.eval_statement(&stmt).unwrap();
        assert!(ev.get_var("X").is_err());
    }

    #[test]
    fn test_if_else() {
        let mut ev = make_evaluator();

        let stmt = if_node(
            bool_lit(false),
            var_assign("X", int_lit(1)),
            vec![],
            Some(var_assign("X", int_lit(2))),
        );
        ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_if_else_if() {
        let mut ev = make_evaluator();

        let stmt = if_node(
            bool_lit(false),
            var_assign("X", int_lit(1)),
            vec![else_if_node(bool_lit(true), var_assign("X", int_lit(2)))],
            Some(var_assign("X", int_lit(3))),
        );
        ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_for_loop() {
        let mut ev = make_evaluator();

        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = for_node(
            "X",
            string_lit("a\nb\nc"),
            print_node(var_ref("X")),
        );
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "a\nb\nc\n");
    }

    #[test]
    fn test_while_loop() {
        let mut ev = make_evaluator();

        ev.set_var("COUNT", Value::Int(0)).unwrap();
        let body = block(vec![
            var_assign(
                "COUNT",
                bin_op(var_ref("COUNT"), "+", int_lit(1)),
            ),
        ]);
        let stmt = while_node(
            bin_op(var_ref("COUNT"), "<", int_lit(3)),
            body,
        );
        ev.eval_statement(&stmt).unwrap();
        assert_eq!(ev.get_var("COUNT").unwrap(), Value::Int(3));
    }

    // --- Break/continue ---

    #[test]
    fn test_break() {
        let mut ev = make_evaluator();
        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let body = block(vec![
            if_node(
                bin_op(var_ref("X"), "==", string_lit("b")),
                break_node(),
                vec![],
                None,
            ),
            print_node(var_ref("X")),
        ]);
        let stmt = for_node("X", string_lit("a\nb\nc"), body);
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output.trim(), "a");
    }

    #[test]
    fn test_continue() {
        let mut ev = make_evaluator();
        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let body = block(vec![
            if_node(
                bin_op(var_ref("X"), "==", string_lit("b")),
                continue_node(),
                vec![],
                None,
            ),
            print_node(var_ref("X")),
        ]);
        let stmt = for_node("X", string_lit("a\nb\nc"), body);
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "a\nc\n");
    }

    // --- Functions ---

    #[test]
    fn test_fn_decl_and_call() {
        let mut ev = make_evaluator();

        let body = bin_op(var_ref("A"), "+", var_ref("B"));
        ev.eval_statement(&fn_decl_node("add", vec!["A".to_string(), "B".to_string()], body))
            .unwrap();

        let call = fn_call_node("add", vec![int_lit(3), int_lit(4)]);
        let result = ev.eval_statement(&call).unwrap();
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn test_fn_return() {
        let mut ev = make_evaluator();

        let body = return_node(Some(bin_op(var_ref("A"), "+", var_ref("B"))));
        ev.eval_statement(&fn_decl_node("add", vec!["A".to_string(), "B".to_string()], body))
            .unwrap();

        let call = fn_call_node("add", vec![int_lit(10), int_lit(20)]);
        let result = ev.eval_statement(&call).unwrap();
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_scope_in_fn() {
        let mut ev = make_evaluator();

        ev.set_var("OUTER", Value::Int(99)).unwrap();

        let body = block(vec![
            var_assign("X", bin_op(var_ref("OUTER"), "+", var_ref("A"))),
            return_node(Some(var_ref("X"))),
        ]);
        ev.eval_statement(&fn_decl_node("f", vec!["A".to_string()], body))
            .unwrap();

        let call = fn_call_node("f", vec![int_lit(1)]);
        let result = ev.eval_statement(&call).unwrap();
        assert_eq!(result, Value::Int(100));
    }

    // --- Block/scope ---

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

    // --- Additional edge cases ---

    #[test]
    fn test_nested_if() {
        let mut ev = make_evaluator();

        let inner = if_node(
            bin_op(int_lit(1), "==", int_lit(1)),
            var_assign("Y", int_lit(10)),
            vec![],
            None,
        );
        let outer = if_node(
            bool_lit(true),
            block(vec![inner]),
            vec![],
            None,
        );
        ev.eval_statement(&outer).unwrap();
        assert_eq!(ev.get_var("Y").unwrap(), Value::Int(10));
    }

    #[test]
    fn test_for_with_array() {
        let mut ev = make_evaluator();
        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let arr = Node::ArrayLiteral(ArrayLiteral {
            pos: Pos { line: 1, col: 1 },
            elements: vec![string_lit("x"), string_lit("y")],
        });
        let stmt = for_node("ITEM", arr, print_node(var_ref("ITEM")));
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "x\ny\n");
    }

    #[test]
    fn test_fn_unknown() {
        let mut ev = make_evaluator();

        let call = fn_call_node("nonexistent", vec![]);
        let result = ev.eval_statement(&call);
        assert!(result.is_err());
        match result {
            Err(EvalError::Msg(s)) => assert!(s.contains("unknown function")),
            _ => panic!("expected Msg error"),
        }
    }

    // ===== Advanced feature tests =====

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

    fn include_node(path: Node) -> Node {
        Node::Include(Include {
            pos: Pos { line: 1, col: 1 },
            path: Box::new(path),
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

        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "bg task\n");
    }

    #[test]
    fn test_include_file() {
        let mut ev = make_evaluator();
        let dir = std::env::temp_dir();
        let path = dir.join("test_include_expr.ash");
        let content = "X = 42\nprint X\n";
        std::fs::write(&path, content).unwrap();

        let buf = Arc::new(Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit(path.to_str().unwrap()));
        ev.eval_statement(&stmt).unwrap();

        assert_eq!(ev.get_var("X").unwrap(), Value::Int(42));
        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        assert_eq!(output, "42\n");

        let _ = std::fs::remove_file(&path);
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
}

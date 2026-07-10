use std::fs;

use log::debug;

use crate::lang::ast::*;
use crate::runtime::value::Value;

use super::{EvalError, Evaluator, ExitError, FlowSignal, SignalKind};

impl Evaluator {
    pub(super) fn eval_if(&mut self, n: &IfStmt) -> Result<Value, EvalError> {
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

    pub(super) fn eval_for(&mut self, n: &ForStmt) -> Result<Value, EvalError> {
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

    pub(super) fn eval_while(&mut self, n: &WhileStmt) -> Result<Value, EvalError> {
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

    pub(super) fn eval_return(&mut self, n: &Return) -> Result<Value, EvalError> {
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

    pub(super) fn eval_break(&mut self) -> Result<Value, EvalError> {
        self.signal = Some(FlowSignal {
            kind: SignalKind::Break,
            value: None,
        });
        Ok(Value::Nil)
    }

    pub(super) fn eval_continue(&mut self) -> Result<Value, EvalError> {
        self.signal = Some(FlowSignal {
            kind: SignalKind::Continue,
            value: None,
        });
        Ok(Value::Nil)
    }

    pub(super) fn eval_exit(&mut self, n: &Exit) -> Result<Value, EvalError> {
        let code_val = self.eval_expr(&n.code)?;
        let code = match &code_val {
            Value::Int(i) => *i as i32,
            Value::Float(f) => *f as i32,
            _ => 0,
        };
        self.set_exit_code(code);
        Err(EvalError::Exit(ExitError { code }))
    }

    pub(super) fn eval_include(&mut self, n: &Include) -> Result<Value, EvalError> {
        let path_val = self.eval_expr(&n.path)?;
        let path = match &path_val {
            Value::String(s) => s.clone(),
            other => format!("{}", other),
        };

        let resolved = self.resolve_include_path(&path);
        debug!("eval — include file {}", resolved.display());
        let src = fs::read_to_string(&resolved)
            .map_err(|e| EvalError::Msg(format!("include: failed to read '{}': {}", resolved.display(), e)))?;

        let script = crate::lang::parser::parse_str(&src)
            .map_err(|e| EvalError::Msg(format!("include: parse error: {}", e)))?;

        let saved_source = self.source_path.clone();
        self.source_path = Some(resolved);

        for stmt in script.body {
            self.eval_statement(&stmt)?;
        }

        self.source_path = saved_source;

        self.set_exit_code(0);
        Ok(Value::Nil)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    use super::super::*;
    use crate::lang::ast::*;
    use crate::util::lock_guard;

    fn make_evaluator() -> Evaluator {
        Evaluator::new()
    }

    fn int_lit(v: i64) -> Node { Node::IntLiteral(IntLiteral { pos: Pos { line: 1, col: 1 }, value: v }) }
    fn bool_lit(v: bool) -> Node { Node::BoolLiteral(BoolLiteral { pos: Pos { line: 1, col: 1 }, value: v }) }
    fn string_lit(s: &str) -> Node { Node::StringLiteral(StringLiteral { pos: Pos { line: 1, col: 1 }, value: s.to_string(), interps: vec![] }) }
    fn var_ref(name: &str) -> Node { Node::VarRef(VarRef { pos: Pos { line: 1, col: 1 }, name: name.to_string() }) }
    fn var_assign(name: &str, val: Node) -> Node { Node::VarAssign(VarAssign { pos: Pos { line: 1, col: 1 }, name: name.to_string(), value: Box::new(val) }) }
    fn bin_op(left: Node, op: &str, right: Node) -> Node { Node::BinaryExpr(BinaryExpr { pos: Pos { line: 1, col: 1 }, left: Box::new(left), op: op.to_string(), right: Box::new(right) }) }
    fn block(stmts: Vec<Node>) -> Node { Node::Block(Block { pos: Pos { line: 1, col: 1 }, statements: stmts }) }
    fn print_node(msg: Node) -> Node { Node::Print(Print { pos: Pos { line: 1, col: 1 }, message: Box::new(msg) }) }
    fn exit_node(code: Node) -> Node { Node::Exit(Exit { pos: Pos { line: 1, col: 1 }, code: Box::new(code) }) }
    fn if_node(cond: Node, body: Node, else_ifs: Vec<ElseIf>, else_body: Option<Node>) -> Node { Node::IfStmt(IfStmt { pos: Pos { line: 1, col: 1 }, cond: Box::new(cond), body: Box::new(body), else_ifs, else_body: else_body.map(Box::new) }) }
    fn else_if_node(cond: Node, body: Node) -> ElseIf { ElseIf { pos: Pos { line: 1, col: 1 }, cond: Box::new(cond), body: Box::new(body) } }
    fn for_node(var: &str, list: Node, body: Node) -> Node { Node::ForStmt(ForStmt { pos: Pos { line: 1, col: 1 }, var: var.to_string(), list: Box::new(list), body: Box::new(body) }) }
    fn while_node(cond: Node, body: Node) -> Node { Node::WhileStmt(WhileStmt { pos: Pos { line: 1, col: 1 }, cond: Box::new(cond), body: Box::new(body) }) }
    fn fn_decl_node(name: &str, params: Vec<String>, body: Node) -> Node { Node::FnDecl(FnDecl { pos: Pos { line: 1, col: 1 }, name: name.to_string(), params, body: Box::new(body) }) }
    fn fn_call_node(name: &str, args: Vec<Node>) -> Node { Node::FnCall(FnCall { pos: Pos { line: 1, col: 1 }, name: name.to_string(), args }) }
    fn return_node(val: Option<Node>) -> Node { Node::Return(Return { pos: Pos { line: 1, col: 1 }, value: val.map(Box::new) }) }
    fn break_node() -> Node { Node::Break(Break { pos: Pos { line: 1, col: 1 } }) }
    fn continue_node() -> Node { Node::Continue(Continue { pos: Pos { line: 1, col: 1 } }) }
    fn include_node(path: Node) -> Node { Node::Include(Include { pos: Pos { line: 1, col: 1 }, path: Box::new(path) }) }

    fn shared_writer(buf: Arc<std::sync::Mutex<Vec<u8>>>) -> SharedWriter {
        Arc::new(std::sync::Mutex::new(Box::new(StdoutWriter(buf))))
    }
    struct StdoutWriter(Arc<std::sync::Mutex<Vec<u8>>>);
    impl Write for StdoutWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { lock_guard(&self.0).write(buf) }
        fn flush(&mut self) -> std::io::Result<()> { lock_guard(&self.0).flush() }
    }

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

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = for_node(
            "X",
            string_lit("a\nb\nc"),
            print_node(var_ref("X")),
        );
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
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

    #[test]
    fn test_break() {
        let mut ev = make_evaluator();
        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
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

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output.trim(), "a");
    }

    #[test]
    fn test_continue() {
        let mut ev = make_evaluator();
        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
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

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "a\nc\n");
    }

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
        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let arr = Node::ArrayLiteral(ArrayLiteral {
            pos: Pos { line: 1, col: 1 },
            elements: vec![string_lit("x"), string_lit("y")],
        });
        let stmt = for_node("ITEM", arr, print_node(var_ref("ITEM")));
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "x\ny\n");
    }

    #[test]
    fn test_exit() {
        let mut ev = make_evaluator();
        ev.stdout = Arc::new(std::sync::Mutex::new(Box::new(std::io::sink())));

        let result = ev.eval_statement(&exit_node(int_lit(42)));
        match result {
            Err(EvalError::Exit(ex)) => assert_eq!(ex.code, 42),
            other => panic!("expected Exit error, got {:?}", other),
        }
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(42));
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
    fn test_include_file() {
        let mut ev = make_evaluator();
        let dir = std::env::temp_dir();
        let path = dir.join("test_include_expr.ash");
        let content = "X = 42\nprint X\n";
        std::fs::write(&path, content).unwrap();

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit(path.to_str().unwrap()));
        ev.eval_statement(&stmt).unwrap();

        assert_eq!(ev.get_var("X").unwrap(), Value::Int(42));
        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "42\n");

        let _ = std::fs::remove_file(&path);
    }

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_temp_dir() -> std::path::PathBuf {
        let id = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("ash-test-{}-{}", std::process::id(), id));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_include_sibling_resolves_via_source_path() {
        let mut ev = make_evaluator();
        let dir = unique_temp_dir();
        let main_path = dir.join("main.ash");
        let sibling_path = dir.join("sibling.ash");

        std::fs::write(&main_path, "include \"sibling.ash\"").unwrap();
        std::fs::write(&sibling_path, "print \"hello from sibling\"").unwrap();

        ev.source_path = Some(main_path.clone());

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit("sibling.ash"));
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "hello from sibling\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_include_absolute_passthrough() {
        let mut ev = make_evaluator();
        let dir = unique_temp_dir();
        let script_path = dir.join("script.ash");

        std::fs::write(&script_path, "print \"direct\"").unwrap();

        ev.source_path = Some(dir.join("other.ash"));

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit(script_path.to_str().unwrap()));
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "direct\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_include_nested_resolves_relative_to_included() {
        let mut ev = make_evaluator();
        let dir = unique_temp_dir();
        let subdir = dir.join("sub");
        std::fs::create_dir_all(&subdir).unwrap();

        let main_path = dir.join("main.ash");
        let included_path = subdir.join("included.ash");
        let sibling_path = subdir.join("sibling.ash");

        std::fs::write(&main_path, "include \"sub/included.ash\"").unwrap();
        std::fs::write(&included_path, "include \"sibling.ash\"").unwrap();
        std::fs::write(&sibling_path, "print \"nested ok\"").unwrap();

        ev.source_path = Some(main_path);

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit("sub/included.ash"));
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "nested ok\n");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_include_source_path_restored_after_nested() {
        let mut ev = make_evaluator();
        let dir = unique_temp_dir();
        let subdir = dir.join("sub");
        std::fs::create_dir_all(&subdir).unwrap();

        let main_path = dir.join("main.ash");
        let included_path = subdir.join("included.ash");

        std::fs::write(&main_path, "include \"sub/included.ash\"").unwrap();
        std::fs::write(&included_path, "print \"included\"").unwrap();

        ev.source_path = Some(main_path.clone());

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit("sub/included.ash"));
        ev.eval_statement(&stmt).unwrap();

        assert_eq!(ev.source_path, Some(main_path));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_include_no_source_path_uses_cwd() {
        let mut ev = make_evaluator();
        let dir = unique_temp_dir();
        let sibling_path = dir.join("sibling.ash");

        std::fs::write(&sibling_path, "print \"cwd sibling\"").unwrap();

        // source_path is None — resolves relative to CWD
        assert!(ev.source_path.is_none());

        // Write sibling to CWD (temp_dir) as well
        let cwd_sibling = std::env::temp_dir().join("sibling.ash");
        std::fs::write(&cwd_sibling, "print \"cwd ok\"").unwrap();

        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit("sibling.ash"));
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "cwd sibling\n");

        std::env::set_current_dir(&orig_cwd).unwrap();
        let _ = std::fs::remove_file(&cwd_sibling);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_include_deeply_nested_three_levels() {
        let mut ev = make_evaluator();
        let dir = unique_temp_dir();
        let sub = dir.join("sub");
        let subsub = sub.join("subsub");
        std::fs::create_dir_all(&subsub).unwrap();

        let main_path = dir.join("main.ash");
        let a_path = sub.join("a.ash");
        let b_path = subsub.join("b.ash");
        let c_path = subsub.join("sibling.ash");

        std::fs::write(&main_path, "include \"sub/a.ash\"").unwrap();
        std::fs::write(&a_path, "include \"subsub/b.ash\"").unwrap();
        std::fs::write(&b_path, "include \"sibling.ash\"").unwrap();
        std::fs::write(&c_path, "print \"deep ok\"").unwrap();

        ev.source_path = Some(main_path);

        let buf = Arc::new(std::sync::Mutex::new(Vec::new()));
        ev.stdout = shared_writer(buf.clone());

        let stmt = include_node(string_lit("sub/a.ash"));
        ev.eval_statement(&stmt).unwrap();

        let output = String::from_utf8(lock_guard(&buf).clone()).unwrap();
        assert_eq!(output, "deep ok\n");

        let _ = std::fs::remove_dir_all(&dir);
    }
}

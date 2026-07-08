use std::fs;

use log::debug;
use regex::Regex;

use crate::lang::ast::*;
use crate::runtime::interpolation::Interpolation;
use crate::runtime::value::Value;

use super::{EvalError, Evaluator, SignalKind};

impl Evaluator {
    pub(super) fn eval_expr(&mut self, node: &Node) -> Result<Value, EvalError> {
        match node {
            Node::VarAssign(n) => {
                let val = self.eval_expr(&n.value)?;
                self.current_scope
                    .lock()
                    .unwrap()
                    .set(&n.name, val.clone());
                Ok(val)
            }
            Node::BinaryExpr(n) => self.eval_binary_expr(n),
            Node::UnaryExpr(n) => self.eval_unary_expr(n),
            Node::VarRef(n) => Ok(self.get_var(&n.name)?),
            Node::StringLiteral(n) => self.eval_string(n),
            Node::TextBlock(n) => self.eval_text_block(n),
            Node::IntLiteral(n) => Ok(Value::Int(n.value)),
            Node::FloatLiteral(n) => Ok(Value::Float(n.value)),
            Node::BoolLiteral(n) => Ok(Value::Bool(n.value)),
            Node::CommandSubst(n) => self.eval_command_subst(n),
            Node::FnCall(n) => self.eval_fn_call(n),
            Node::ArrayLiteral(n) => self.eval_array_literal(n),
            Node::IndexExpr(n) => self.eval_index_expr(n),
            Node::GroupExpr(n) => self.eval_expr(&n.inner),
            Node::FilePath(n) => self.eval_fp(n),
            _ => Err(EvalError::Msg(format!(
                "cannot evaluate {:?} as expression",
                node
            ))),
        }
    }

    fn eval_binary_expr(&mut self, n: &BinaryExpr) -> Result<Value, EvalError> {
        match n.op.as_str() {
            "and" | "or" => {
                let left = self.eval_expr(&n.left)?;
                let left_bool = left.is_truthy();

                match n.op.as_str() {
                    "and" => {
                        if !left_bool {
                            return Ok(Value::Bool(false));
                        }
                    }
                    "or" => {
                        if left_bool {
                            return Ok(Value::Bool(true));
                        }
                    }
                    _ => unreachable!(),
                }

                let right = self.eval_expr(&n.right)?;
                Ok(Value::Bool(right.is_truthy()))
            }
            "==" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.eq(&right).map_err(Into::into)
            }
            "!=" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.ne(&right).map_err(Into::into)
            }
            ">" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.gt(&right).map_err(Into::into)
            }
            "<" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.lt(&right).map_err(Into::into)
            }
            ">=" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.ge(&right).map_err(Into::into)
            }
            "<=" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.le(&right).map_err(Into::into)
            }
            "+" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.add(&right).map_err(Into::into)
            }
            "-" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.sub(&right).map_err(Into::into)
            }
            "*" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.mul(&right).map_err(Into::into)
            }
            "/" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.div(&right).map_err(Into::into)
            }
            "%" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.rem(&right).map_err(Into::into)
            }
            _ => Err(EvalError::Msg(format!(
                "unknown binary operator: {}",
                n.op
            ))),
        }
    }

    fn eval_unary_expr(&mut self, n: &UnaryExpr) -> Result<Value, EvalError> {
        let val = self.eval_expr(&n.right)?;
        match n.op.as_str() {
            "not" => Ok(Value::Bool(!val.is_truthy())),
            "-" => val.neg().map_err(Into::into),
            "+" => Ok(val),
            _ => Err(EvalError::Msg(format!(
                "unknown unary operator: {}",
                n.op
            ))),
        }
    }

    fn eval_string(&mut self, n: &StringLiteral) -> Result<Value, EvalError> {
        self.resolve_interpolations(&n.value, &n.interps)
            .map(Value::String)
    }

    fn eval_text_block(&mut self, n: &TextBlock) -> Result<Value, EvalError> {
        self.resolve_interpolations(&n.value, &n.interps)
            .map(Value::String)
    }

    fn resolve_interpolations(
        &mut self,
        value: &str,
        interps: &[InterpSpan],
    ) -> Result<String, EvalError> {
        if !interps.is_empty() {
            let scope = self.current_scope.clone();
            let executor = &mut self.executor;
            let get_var = |name: &str| -> Option<Value> { crate::util::lock_guard(&scope).get(name) };
            let exec_cmd = |cmd: &str| -> Result<String, crate::AshError> {
                let result = executor.run(cmd)?;
                Ok(result.stdout)
            };
            return Ok(Interpolation::resolve_spans(interps, &get_var, &exec_cmd)?);
        }
        let scope = self.current_scope.clone();
        let get_var = move |name: &str| -> Option<String> {
            scope
                .lock()
                .unwrap()
                .get(name)
                .map(|v| format!("{}", v))
        };
        let exec_cmd = |cmd: &str| -> Result<String, crate::AshError> {
            let result = self.executor.run(cmd)?;
            Ok(result.stdout)
        };
        let result = Interpolation::resolve(value, get_var, exec_cmd)?;
        self.resolve_remaining_exprs(&result)
    }

    fn resolve_remaining_exprs(&mut self, value: &str) -> Result<String, EvalError> {
        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        if !re.is_match(value) {
            return Ok(value.to_string());
        }
        let captures: Vec<String> = re.captures_iter(value)
            .map(|c| c[1].to_string())
            .collect();
        let mut result = value.to_string();
        for expr_str in captures {
            let placeholder = format!("${{{}}}", expr_str);
            if !result.contains(&placeholder) {
                continue;
            }
            if let Ok(expr) = crate::lang::parser::parse_expr_str(&expr_str) {
                if let Ok(val) = self.eval_expr(&expr) {
                    result = result.replace(&placeholder, &format!("{}", val));
                }
            }
        }
        Ok(result)
    }

    fn eval_command_subst(&mut self, n: &CommandSubst) -> Result<Value, EvalError> {
        let result = self.executor.run(&n.cmd)?;
        self.set_exit_code(result.exit_code);
        Ok(Value::String(result.stdout.trim().to_string()))
    }

    fn eval_array_literal(&mut self, n: &ArrayLiteral) -> Result<Value, EvalError> {
        let mut elements = Vec::with_capacity(n.elements.len());
        for elem in &n.elements {
            elements.push(self.eval_expr(elem)?);
        }
        Ok(Value::Array(elements))
    }

    fn eval_index_expr(&mut self, n: &IndexExpr) -> Result<Value, EvalError> {
        let object = self.eval_expr(&n.object)?;
        let index = self.eval_expr(&n.index)?;

        let idx = match &index {
            Value::Int(i) => *i,
            _ => {
                return Err(EvalError::Msg(format!(
                    "index must be an integer, got {}",
                    index.type_name()
                )))
            }
        };

        match &object {
            Value::String(s) => {
                let chars: Vec<char> = s.chars().collect();
                if idx < 0 || idx as usize >= chars.len() {
                    return Err(EvalError::Msg(format!(
                        "index out of bounds: {} (len={})",
                        idx,
                        chars.len()
                    )));
                }
                Ok(Value::String(chars[idx as usize].to_string()))
            }
            Value::Array(a) => {
                if idx < 0 || idx as usize >= a.len() {
                    return Err(EvalError::Msg(format!(
                        "index out of bounds: {} (len={})",
                        idx,
                        a.len()
                    )));
                }
                Ok(a[idx as usize].clone())
            }
            _ => Err(EvalError::Msg(format!(
                "cannot index into {}",
                object.type_name()
            ))),
        }
    }

    fn eval_fp(&mut self, n: &FilePath) -> Result<Value, EvalError> {
        let path_val = self.eval_expr(&n.path)?;
        let path = match &path_val {
            Value::String(s) => s.clone(),
            _ => {
                return Err(EvalError::Msg(format!(
                    "file path must be a string, got {}",
                    path_val.type_name()
                )))
            }
        };
        debug!("eval — reading file {}", path);
        let content = fs::read_to_string(&path)
            .map_err(|e| EvalError::Msg(format!("failed to read file '{}': {}", path, e)))?;
        self.resolve_interpolations(&content, &[]).map(Value::String)
    }

    pub(super) fn eval_fn_call(&mut self, n: &FnCall) -> Result<Value, EvalError> {
        match n.name.as_str() {
            "len" => {
                if n.args.len() != 1 {
                    return Err(EvalError::Msg(format!(
                        "len() takes 1 argument, got {}",
                        n.args.len()
                    )));
                }
                let arg = self.eval_expr(&n.args[0])?;
                arg.len().map_err(Into::into)
            }
            "range" => {
                let args: Result<Vec<Value>, _> = n.args.iter().map(|a| self.eval_expr(a)).collect();
                let args = args?;
                match args.len() {
                    1 => {
                        let end = match &args[0] {
                            Value::Int(i) => *i,
                            _ => return Err(EvalError::Msg("range() expects integer argument".into())),
                        };
                        Ok(Value::Array((0..end).map(|i| Value::Int(i)).collect()))
                    }
                    2 => {
                        let start = match &args[0] {
                            Value::Int(i) => *i,
                            _ => return Err(EvalError::Msg("range() expects integer argument".into())),
                        };
                        let end = match &args[1] {
                            Value::Int(i) => *i,
                            _ => return Err(EvalError::Msg("range() expects integer argument".into())),
                        };
                        Ok(Value::Array((start..end).map(|i| Value::Int(i)).collect()))
                    }
                    _ => Err(EvalError::Msg(format!(
                        "range() takes 1 or 2 arguments, got {}",
                        args.len()
                    ))),
                }
            }
            _ => {
                let fn_decl = self
                    .current_scope
                    .lock()
                    .unwrap()
                    .get_function(&n.name)
                    .ok_or_else(|| EvalError::Msg(format!("unknown function: {}", n.name)))?;

                if fn_decl.params.len() != n.args.len() {
                    return Err(EvalError::Msg(format!(
                        "function '{}' takes {} arguments, got {}",
                        n.name,
                        fn_decl.params.len(),
                        n.args.len()
                    )));
                }

                let mut args = Vec::new();
                for arg in &n.args {
                    args.push(self.eval_expr(arg)?);
                }

                self.push_scope();
                for (param, arg) in fn_decl.params.iter().zip(args.iter()) {
                    self.current_scope
                        .lock()
                        .unwrap()
                        .set_local(param, arg.clone());
                }

                let body_result = self.eval_statement(&fn_decl.body);

                let return_val = if let Some(signal) = self.signal.take() {
                    if matches!(signal.kind, SignalKind::Return) {
                        signal.value.unwrap_or(Value::Nil)
                    } else {
                        self.signal = Some(signal);
                        self.pop_scope();
                        return body_result;
                    }
                } else {
                    match body_result {
                        Ok(v) => v,
                        Err(e) => {
                            self.pop_scope();
                            return Err(e);
                        }
                    }
                };

                self.pop_scope();
                Ok(return_val)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::lang::ast::*;

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
}

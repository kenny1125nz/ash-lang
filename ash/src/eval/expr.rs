use std::fs;

use log::debug;
use regex::Regex;

use crate::ast::*;
use crate::interpolation::Interpolation;
use crate::value::Value;

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
            Node::VarRef(n) => self.get_var(&n.name).map_err(EvalError::Msg),
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
                left.eq(&right).map_err(EvalError::Msg)
            }
            "!=" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.ne(&right).map_err(EvalError::Msg)
            }
            ">" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.gt(&right).map_err(EvalError::Msg)
            }
            "<" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.lt(&right).map_err(EvalError::Msg)
            }
            ">=" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.ge(&right).map_err(EvalError::Msg)
            }
            "<=" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.le(&right).map_err(EvalError::Msg)
            }
            "+" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.add(&right).map_err(EvalError::Msg)
            }
            "-" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.sub(&right).map_err(EvalError::Msg)
            }
            "*" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.mul(&right).map_err(EvalError::Msg)
            }
            "/" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.div(&right).map_err(EvalError::Msg)
            }
            "%" => {
                let left = self.eval_expr(&n.left)?;
                let right = self.eval_expr(&n.right)?;
                left.rem(&right).map_err(EvalError::Msg)
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
            "-" => val.neg().map_err(EvalError::Msg),
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
            let get_var = |name: &str| -> Option<Value> { scope.lock().unwrap().get(name) };
            let exec_cmd = |cmd: &str| -> Result<String, String> {
                let result = executor.run(cmd)?;
                Ok(result.stdout)
            };
            return Interpolation::resolve_spans(interps, &get_var, &exec_cmd)
                .map_err(EvalError::Msg);
        }
        let scope = self.current_scope.clone();
        let get_var = move |name: &str| -> Option<String> {
            scope
                .lock()
                .unwrap()
                .get(name)
                .map(|v| format!("{}", v))
        };
        let exec_cmd = |cmd: &str| -> Result<String, String> {
            let result = self.executor.run(cmd)?;
            Ok(result.stdout)
        };
        let result = Interpolation::resolve(value, get_var, exec_cmd)
            .map_err(EvalError::Msg)?;
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
            if let Ok(expr) = crate::parser::parse_expr_str(&expr_str) {
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

    fn eval_fn_call(&mut self, n: &FnCall) -> Result<Value, EvalError> {
        match n.name.as_str() {
            "len" => {
                if n.args.len() != 1 {
                    return Err(EvalError::Msg(format!(
                        "len() takes 1 argument, got {}",
                        n.args.len()
                    )));
                }
                let arg = self.eval_expr(&n.args[0])?;
                arg.len().map_err(EvalError::Msg)
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

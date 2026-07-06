use std::sync::Arc;
use std::thread;

use crate::ast::*;
use crate::compact::Config as CompactConfig;
use crate::executor::Executor;

use super::Evaluator;

impl Evaluator {
    pub(super) fn eval_wait(&mut self, n: &WaitBlock) -> Result<super::Value, super::EvalError> {
        if let Some(body) = &n.body {
            if let Node::Block(block) = &**body {
                let mut handles = Vec::new();
                for stmt in &block.statements {
                    let scope = self.current_scope.clone();
                    let stmt_clone = stmt.clone();
                    let stdout = self.stdout.clone();
                    let stderr = self.stderr.clone();
                    let handle = thread::spawn(move || {
                        let mut eval = Evaluator {
                            global_scope: scope.clone(),
                            current_scope: scope,
                            stdout,
                            stderr,
                            executor: Executor::new(),
                            compact_config: CompactConfig::new(),
                            signal: None,
                            bg_handles: Arc::new(std::sync::Mutex::new(Vec::new())),
                            default_agent: super::DEFAULT_AGENT.to_string(),
                            default_model: String::new(),
                            session_depth: 0,
                            within_stack: Vec::new(),
                        };
                        eval.push_scope();
                        let _ = eval.eval_statement(&stmt_clone);
                    });
                    handles.push(handle);
                }
                for h in handles {
                    let _ = h.join();
                }
            }
        }

        let bg_handles = std::mem::replace(&mut *self.bg_handles.lock().unwrap(), Vec::new());
        for h in bg_handles {
            let _ = h.join();
        }

        Ok(super::Value::Int(0))
    }

    pub(super) fn eval_background(&mut self, n: &Background) -> Result<super::Value, super::EvalError> {
        let scope = self.current_scope.clone();
        let stmt = n.stmt.clone();
        let stdout = self.stdout.clone();
        let stderr = self.stderr.clone();
        let handle = thread::spawn(move || {
            let mut eval = Evaluator {
                global_scope: scope.clone(),
                current_scope: scope,
                stdout,
                stderr,
                executor: Executor::new(),
                compact_config: CompactConfig::new(),
                signal: None,
                bg_handles: Arc::new(std::sync::Mutex::new(Vec::new())),
                default_agent: super::DEFAULT_AGENT.to_string(),
                default_model: String::new(),
                session_depth: 0,
                within_stack: Vec::new(),
            };
            eval.push_scope();
            let _ = eval.eval_statement(&stmt);
        });
        self.bg_handles.lock().unwrap().push(handle);
        Ok(super::Value::Int(0))
    }
}

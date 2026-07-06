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

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ast::*;

    fn background_stmt_node(inner: Node) -> Node {
        Node::Background(Background {
            pos: Pos { line: 1, col: 1 },
            stmt: Box::new(inner),
        })
    }

    fn wait_node(body: Option<Node>) -> Node {
        Node::WaitBlock(WaitBlock {
            pos: Pos { line: 1, col: 1 },
            body: body.map(Box::new),
        })
    }

    #[test]
    fn test_background_stores_handle() {
        let mut ev = Evaluator::new();
        ev.stdout = Arc::new(std::sync::Mutex::new(Box::new(std::io::sink())));

        let task = Node::Exec(Exec {
            pos: Pos { line: 1, col: 1 },
            cmd: Box::new(Node::StringLiteral(StringLiteral {
                pos: Pos { line: 1, col: 1 },
                value: "echo bg".into(),
                interps: vec![],
            })),
        });

        assert_eq!(ev.bg_handles.lock().unwrap().len(), 0);
        ev.eval_statement(&background_stmt_node(task)).unwrap();
        assert_eq!(ev.bg_handles.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_wait_drains_background_handles() {
        let mut ev = Evaluator::new();
        ev.stdout = Arc::new(std::sync::Mutex::new(Box::new(std::io::sink())));

        let task = Node::Exec(Exec {
            pos: Pos { line: 1, col: 1 },
            cmd: Box::new(Node::StringLiteral(StringLiteral {
                pos: Pos { line: 1, col: 1 },
                value: "echo wait-test".into(),
                interps: vec![],
            })),
        });

        ev.eval_statement(&background_stmt_node(task.clone())).unwrap();
        assert_eq!(ev.bg_handles.lock().unwrap().len(), 1);

        ev.eval_statement(&wait_node(None)).unwrap();
        assert_eq!(ev.bg_handles.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_wait_runs_body_in_parallel() {
        let mut ev = Evaluator::new();
        ev.stdout = Arc::new(std::sync::Mutex::new(Box::new(std::io::sink())));

        // Use exec to run a lightweight shell command in each parallel task
        let task1 = Node::Exec(Exec {
            pos: Pos { line: 1, col: 1 },
            cmd: Box::new(Node::StringLiteral(StringLiteral {
                pos: Pos { line: 1, col: 1 },
                value: "echo p1".into(),
                interps: vec![],
            })),
        });
        let task2 = Node::Exec(Exec {
            pos: Pos { line: 1, col: 1 },
            cmd: Box::new(Node::StringLiteral(StringLiteral {
                pos: Pos { line: 1, col: 1 },
                value: "echo p2".into(),
                interps: vec![],
            })),
        });

        let body = Node::Block(Block {
            pos: Pos { line: 1, col: 1 },
            statements: vec![task1, task2],
        });

        let result = ev.eval_statement(&wait_node(Some(body))).unwrap();
        assert_eq!(result, Value::Int(0));
    }
}

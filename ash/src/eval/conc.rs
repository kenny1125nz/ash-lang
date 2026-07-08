use std::sync::mpsc;

use crate::lang::ast::*;
use crate::util::{get_pool, lock_guard};

use super::Evaluator;

impl Evaluator {
    pub(super) fn eval_wait(&mut self, n: &WaitBlock) -> Result<super::Value, super::EvalError> {
        if let Some(body) = &n.body {
            if let Node::Block(block) = &**body {
                let mut handles = Vec::new();
                for stmt in &block.statements {
                    let stmt_clone = stmt.clone();
                    let mut eval = self.fork();
                    let (tx, rx) = mpsc::channel();
                    get_pool().execute(move || {
                        eval.push_scope();
                        let _ = eval.eval_statement(&stmt_clone);
                        let _ = tx.send(());
                    });
                    handles.push(rx);
                }
                for rx in handles {
                    let _ = rx.recv();
                }
            }
        }

        let bg_handles = std::mem::replace(&mut *lock_guard(&self.bg_handles), Vec::new());
        for rx in bg_handles {
            let _ = rx.recv();
        }

        Ok(super::Value::Int(0))
    }

    pub(super) fn eval_background(&mut self, n: &Background) -> Result<super::Value, super::EvalError> {
        let stmt = n.stmt.clone();
        let mut eval = self.fork();
        let (tx, rx) = mpsc::channel();
        get_pool().execute(move || {
            eval.push_scope();
            let _ = eval.eval_statement(&stmt);
            let _ = tx.send(());
        });
        lock_guard(&self.bg_handles).push(rx);
        Ok(super::Value::Int(0))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::super::*;
    use crate::lang::ast::*;

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

        assert_eq!(lock_guard(&ev.bg_handles).len(), 0);
        ev.eval_statement(&background_stmt_node(task)).unwrap();
        assert_eq!(lock_guard(&ev.bg_handles).len(), 1);
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
        assert_eq!(lock_guard(&ev.bg_handles).len(), 1);

        ev.eval_statement(&wait_node(None)).unwrap();
        assert_eq!(lock_guard(&ev.bg_handles).len(), 0);
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

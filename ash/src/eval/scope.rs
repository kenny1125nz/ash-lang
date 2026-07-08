use log::trace;

use crate::util::lock_guard;
use crate::AshError;
use crate::runtime::scope::Scope;
use crate::runtime::value::Value;

use super::Evaluator;

impl Evaluator {
    pub fn push_scope(&mut self) {
        trace!("scope — enter");
        let new_scope = Scope::with_parent(self.current_scope.clone());
        self.current_scope = new_scope;
    }

    pub fn pop_scope(&mut self) {
        trace!("scope — exit");
        let parent = lock_guard(&self.current_scope).parent.clone();
        if let Some(p) = parent {
            self.current_scope = p;
        }
    }

    pub fn get_var(&self, name: &str) -> Result<Value, AshError> {
        self.current_scope
            .lock()
            .unwrap()
            .get(name)
            .ok_or_else(|| AshError::Msg(format!("undefined variable: {}", name)))
    }

    pub fn set_var(&mut self, name: &str, value: Value) -> Result<(), AshError> {
        lock_guard(&self.current_scope).set(name, value);
        Ok(())
    }

    pub fn set_exit_code(&mut self, code: i32) {
        lock_guard(&self.current_scope)
            .set_local("?", Value::Int(code as i64));
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_scope_set_get() {
        let mut ev = Evaluator::new();

        ev.set_var("X", Value::Int(10)).unwrap();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(10));
    }

    #[test]
    fn test_scope_shadow() {
        let mut ev = Evaluator::new();

        ev.set_var("X", Value::Int(1)).unwrap();
        ev.push_scope();
        lock_guard(&ev.current_scope)
            .set_local("X", Value::Int(2));
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(2));
        ev.pop_scope();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_reassign_nearest() {
        let mut ev = Evaluator::new();

        ev.set_var("X", Value::Int(1)).unwrap();
        ev.push_scope();
        ev.set_var("X", Value::Int(2)).unwrap();
        ev.pop_scope();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(2));
    }

    #[test]
    fn test_reassign_local() {
        let mut ev = Evaluator::new();

        ev.set_var("X", Value::Int(1)).unwrap();
        ev.push_scope();
        lock_guard(&ev.current_scope)
            .set_local("X", Value::Int(10));
        ev.set_var("X", Value::Int(20)).unwrap();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(20));
        ev.pop_scope();
        assert_eq!(ev.get_var("X").unwrap(), Value::Int(1));
    }

    #[test]
    fn test_exit_code() {
        let mut ev = Evaluator::new();

        ev.set_exit_code(42);
        assert_eq!(ev.get_var("?").unwrap(), Value::Int(42));
    }
}

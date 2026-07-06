use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::ast::FnDecl;
use crate::value::Value;

pub type ScopeRef = Arc<Mutex<Scope>>;

pub struct Scope {
    pub(crate) parent: Option<ScopeRef>,
    variables: HashMap<String, Value>,
    pub(crate) functions: HashMap<String, FnDecl>,
}

impl Scope {
    pub fn new() -> ScopeRef {
        let scope = Scope {
            parent: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
        };
        let ref_ = Arc::new(Mutex::new(scope));
        {
            let mut s = ref_.lock().unwrap();
            s.variables.insert("?".to_string(), Value::Int(0));
            s.variables.insert("stdout".to_string(), Value::String(String::new()));
            s.variables.insert("stderr".to_string(), Value::String(String::new()));
        }
        ref_
    }

    pub fn with_parent(parent: ScopeRef) -> ScopeRef {
        let scope = Scope {
            parent: Some(parent),
            variables: HashMap::new(),
            functions: HashMap::new(),
        };
        Arc::new(Mutex::new(scope))
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.variables.get(name) {
            Some(v) => Some(v.clone()),
            None => {
                if let Some(ref parent) = self.parent {
                    parent.lock().unwrap().get(name)
                } else {
                    None
                }
            }
        }
    }

    pub fn set(&mut self, name: &str, value: Value) {
        if let Some(ref parent) = self.parent {
            if !self.variables.contains_key(name) {
                parent.lock().unwrap().set(name, value);
                return;
            }
        }
        self.variables.insert(name.to_string(), value);
    }

    pub fn set_local(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn has_local(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    pub fn get_all(&self) -> std::collections::HashMap<String, crate::value::Value> {
        self.variables.clone()
    }

    pub fn get_function(&self, name: &str) -> Option<FnDecl> {
        match self.functions.get(name) {
            Some(f) => Some(f.clone()),
            None => {
                if let Some(ref parent) = self.parent {
                    parent.lock().unwrap().get_function(name)
                } else {
                    None
                }
            }
        }
    }
}

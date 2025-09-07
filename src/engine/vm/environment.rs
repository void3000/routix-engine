use std::collections::HashMap;
use crate::engine::lang::ast::Value;

#[derive(Default)]
pub struct Environment {
    pub env: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env: Environment = Environment { env: Vec::new() };
        env.enter_scope();
        env
    }

    pub fn enter_scope(&mut self) {
        self.env.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.env.pop();
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        for scope in self.env.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

    pub fn insert(&mut self, name: impl Into<String>, value: Value) {
        if let Some(scope) = self.env.last_mut() {
            scope.insert(name.into(), value);
        }
    }

    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        let name = name.into();
        for scope in self.env.iter_mut().rev() {
            if scope.contains_key(&name) {
                scope.insert(name.clone(), value);
                return;
            }
        }
        self.insert(name, value);
    }
}

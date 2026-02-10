mod scope;

use crate::errors::RuntimeError;
use crate::runtime::value::JsValue;
use scope::Scope;

/// The environment manages a stack of scopes for variable lookup.
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a new variable in the current (innermost) scope.
    pub fn define(&mut self, name: String, value: JsValue) {
        self.scopes
            .last_mut()
            .expect("environment must have at least one scope")
            .define(name, value);
    }

    /// Look up a variable by walking the scope chain outward.
    pub fn get(&self, name: &str) -> Result<JsValue, RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(RuntimeError::UndefinedVariable {
            name: name.to_owned(),
        })
    }

    /// Set an existing variable by walking the scope chain outward.
    pub fn set(&mut self, name: &str, value: JsValue) -> Result<(), RuntimeError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.set(name, value.clone()) {
                return Ok(());
            }
        }
        Err(RuntimeError::UndefinedVariable {
            name: name.to_owned(),
        })
    }
}

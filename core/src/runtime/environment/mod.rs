mod scope;

use crate::errors::RuntimeError;
use crate::runtime::gc::{Trace, Tracer};
use crate::runtime::value::JsValue;
pub(crate) use scope::Binding;
pub(crate) use scope::{BindingKind, Scope};
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

/// The environment manages a stack of scopes for variable lookup.
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<Rc<RefCell<Scope>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![Rc::new(RefCell::new(Scope::new()))],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Rc::new(RefCell::new(Scope::new())));
    }

    pub fn push_scope_with_this(&mut self, this_binding: Option<JsValue>) {
        self.scopes
            .push(Rc::new(RefCell::new(Scope::new_with_this(this_binding))));
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a new variable in the current (innermost) scope.
    pub fn define(&mut self, name: String, value: JsValue) {
        self.define_with_kind(name, value, BindingKind::Let);
    }

    pub fn define_with_kind(&mut self, name: String, value: JsValue, kind: BindingKind) {
        self.scopes
            .last_mut()
            .expect("environment must have at least one scope")
            .borrow_mut()
            .define_with_kind(name, value, kind);
    }

    /// Look up a variable by walking the scope chain outward.
    pub fn get(&self, name: &str) -> Result<JsValue, RuntimeError> {
        if name == "this" {
            for scope in self.scopes.iter().rev() {
                let borrowed = scope.borrow();
                if let Some(this_value) = &borrowed.this_binding {
                    return Ok(this_value.clone());
                }
            }
            return Ok(JsValue::Undefined);
        }

        for scope in self.scopes.iter().rev() {
            let borrowed = scope.borrow();
            if let Some(value) = borrowed.get(name) {
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
            let mut borrowed = scope.borrow_mut();
            if borrowed.get(name).is_some() {
                if matches!(borrowed.kind_of(name), Some(BindingKind::Const)) {
                    return Err(RuntimeError::ConstReassignment {
                        name: name.to_string(),
                    });
                }
                borrowed.set(name, value.clone());
                return Ok(());
            }
        }
        Err(RuntimeError::UndefinedVariable {
            name: name.to_owned(),
        })
    }

    pub fn capture(&self) -> Vec<Rc<RefCell<Scope>>> {
        self.scopes.clone()
    }

    pub fn replace_scopes(&mut self, scopes: Vec<Rc<RefCell<Scope>>>) -> Vec<Rc<RefCell<Scope>>> {
        mem::replace(&mut self.scopes, scopes)
    }

    pub(crate) fn current_scope_bindings_snapshot(
        &self,
    ) -> std::collections::HashMap<String, Binding> {
        self.scopes
            .last()
            .map(|scope| scope.borrow().bindings.clone())
            .unwrap_or_default()
    }
}

impl Trace for Environment {
    fn trace(&self, tracer: &mut Tracer) {
        for scope in &self.scopes {
            scope.borrow().trace(tracer);
        }
    }
}

mod scope;

use crate::errors::RuntimeError;
use crate::runtime::gc::{Gc, GcCell, Heap, Trace, Tracer};
use crate::runtime::value::JsValue;
pub(crate) use scope::Binding;
pub(crate) use scope::{BindingKind, Scope};
use std::mem;

#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<Gc<GcCell<Scope>>>,
}

impl Environment {
    pub fn new(heap: &mut Heap) -> Self {
        Self {
            scopes: vec![heap.alloc_cell(Scope::new())],
        }
    }

    pub fn push_scope(&mut self, heap: &mut Heap) {
        self.scopes.push(heap.alloc_cell(Scope::new()));
    }

    pub fn push_scope_with_this(&mut self, heap: &mut Heap, this_binding: Option<JsValue>) {
        self.scopes.push(heap.alloc_cell(Scope::new_with_this(Some(
            this_binding.unwrap_or(JsValue::Undefined),
        ))));
    }

    pub fn set_global_this(&mut self, value: JsValue) {
        if let Some(scope) = self.scopes.first() {
            scope.borrow_mut().this_binding = Some(value);
        }
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

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

    pub fn capture(&self) -> Vec<Gc<GcCell<Scope>>> {
        self.scopes.clone()
    }

    pub fn replace_scopes(&mut self, scopes: Vec<Gc<GcCell<Scope>>>) -> Vec<Gc<GcCell<Scope>>> {
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
            tracer.mark(*scope);
        }
    }
}

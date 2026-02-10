use crate::runtime::value::JsValue;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Let,
    Const,
    Var,
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub value: JsValue,
    pub kind: BindingKind,
}

/// A single scope frame in the environment chain.
#[derive(Debug, Clone)]
pub struct Scope {
    pub(crate) bindings: HashMap<String, Binding>,
    pub(crate) this_binding: Option<JsValue>,
}

impl Scope {
    pub fn new() -> Self {
        Self::new_with_this(None)
    }

    pub fn new_with_this(this_binding: Option<JsValue>) -> Self {
        Self {
            bindings: HashMap::new(),
            this_binding,
        }
    }

    pub fn get(&self, name: &str) -> Option<&JsValue> {
        self.bindings.get(name).map(|b| &b.value)
    }

    pub fn kind_of(&self, name: &str) -> Option<BindingKind> {
        self.bindings.get(name).map(|b| b.kind)
    }

    pub fn set(&mut self, name: &str, value: JsValue) -> bool {
        if let Some(binding) = self.bindings.get_mut(name) {
            binding.value = value;
            true
        } else {
            false
        }
    }

    pub fn define(&mut self, name: String, value: JsValue) {
        self.define_with_kind(name, value, BindingKind::Let);
    }

    pub fn define_with_kind(&mut self, name: String, value: JsValue, kind: BindingKind) {
        self.bindings.insert(name, Binding { value, kind });
    }
}

use crate::runtime::value::JsValue;
use std::collections::HashMap;

/// A single scope frame in the environment chain.
#[derive(Debug, Clone)]
pub struct Scope {
    pub(crate) bindings: HashMap<String, JsValue>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&JsValue> {
        self.bindings.get(name)
    }

    pub fn set(&mut self, name: &str, value: JsValue) -> bool {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_owned(), value);
            true
        } else {
            false
        }
    }

    pub fn define(&mut self, name: String, value: JsValue) {
        self.bindings.insert(name, value);
    }
}

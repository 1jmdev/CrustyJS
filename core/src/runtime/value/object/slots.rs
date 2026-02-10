use std::collections::HashMap;

use crate::runtime::value::JsValue;

#[derive(Debug, Default, Clone)]
pub struct InternalSlots {
    entries: HashMap<String, JsValue>,
}

impl InternalSlots {
    pub fn set(&mut self, name: impl Into<String>, value: JsValue) {
        self.entries.insert(name.into(), value);
    }

    pub fn get(&self, name: &str) -> Option<JsValue> {
        self.entries.get(name).cloned()
    }
}

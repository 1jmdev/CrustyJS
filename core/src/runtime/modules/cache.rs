use crate::runtime::value::JsValue;
use std::collections::HashMap;

#[derive(Default)]
pub struct ModuleCache {
    exports: HashMap<String, HashMap<String, JsValue>>,
}

impl ModuleCache {
    pub fn get(&self, key: &str) -> Option<HashMap<String, JsValue>> {
        self.exports.get(key).cloned()
    }

    pub fn insert(&mut self, key: String, exports: HashMap<String, JsValue>) {
        self.exports.insert(key, exports);
    }
}

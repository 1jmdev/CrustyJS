use crate::runtime::gc::{ErasedGc, Trace, Tracer};
use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct JsWeakMap {
    pub entries: Vec<(ErasedGc, JsValue)>,
}

impl JsWeakMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get(&self, key: ErasedGc) -> JsValue {
        for (k, v) in &self.entries {
            if *k == key {
                return v.clone();
            }
        }
        JsValue::Undefined
    }

    pub fn has(&self, key: ErasedGc) -> bool {
        self.entries.iter().any(|(k, _)| *k == key)
    }

    pub fn set(&mut self, key: ErasedGc, value: JsValue) {
        for entry in &mut self.entries {
            if entry.0 == key {
                entry.1 = value;
                return;
            }
        }
        self.entries.push((key, value));
    }

    pub fn delete(&mut self, key: ErasedGc) -> bool {
        let len = self.entries.len();
        self.entries.retain(|(k, _)| *k != key);
        self.entries.len() != len
    }
}

impl Trace for JsWeakMap {
    fn trace(&self, tracer: &mut Tracer) {
        for (_, v) in &self.entries {
            v.trace(tracer);
        }
    }
}

pub fn extract_weak_key(value: &JsValue) -> Option<ErasedGc> {
    match value {
        JsValue::Object(gc) => Some(gc.erase()),
        JsValue::Array(gc) => Some(gc.erase()),
        JsValue::Map(gc) => Some(gc.erase()),
        JsValue::Set(gc) => Some(gc.erase()),
        _ => None,
    }
}

pub fn is_valid_weak_key(value: &JsValue) -> bool {
    extract_weak_key(value).is_some()
}

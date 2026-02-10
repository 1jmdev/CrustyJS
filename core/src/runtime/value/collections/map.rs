use crate::runtime::gc::{Gc, Trace, Tracer};
use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct JsMap {
    pub entries: Vec<(JsValue, JsValue)>,
}

impl JsMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, key: &JsValue) -> JsValue {
        for (k, v) in &self.entries {
            if map_key_eq(k, key) {
                return v.clone();
            }
        }
        JsValue::Undefined
    }

    pub fn has(&self, key: &JsValue) -> bool {
        self.entries.iter().any(|(k, _)| map_key_eq(k, key))
    }

    pub fn set(&mut self, key: JsValue, value: JsValue) {
        for entry in &mut self.entries {
            if map_key_eq(&entry.0, &key) {
                entry.1 = value;
                return;
            }
        }
        self.entries.push((key, value));
    }

    pub fn delete(&mut self, key: &JsValue) -> bool {
        let len = self.entries.len();
        self.entries.retain(|(k, _)| !map_key_eq(k, key));
        self.entries.len() != len
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Trace for JsMap {
    fn trace(&self, tracer: &mut Tracer) {
        for (k, v) in &self.entries {
            k.trace(tracer);
            v.trace(tracer);
        }
    }
}

fn map_key_eq(a: &JsValue, b: &JsValue) -> bool {
    match (a, b) {
        (JsValue::Number(x), JsValue::Number(y)) => {
            if x.is_nan() && y.is_nan() {
                return true;
            }
            if *x == 0.0 && *y == 0.0 {
                return true;
            }
            x == y
        }
        (JsValue::Object(x), JsValue::Object(y)) => Gc::ptr_eq(*x, *y),
        (JsValue::Array(x), JsValue::Array(y)) => Gc::ptr_eq(*x, *y),
        _ => a == b,
    }
}

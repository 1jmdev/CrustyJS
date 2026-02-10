use crate::runtime::gc::{Gc, GcCell, Trace, Tracer};
use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct JsWeakSet {
    pub entries: Vec<JsValue>,
}

impl JsWeakSet {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn has(&self, value: &JsValue) -> bool {
        self.entries.iter().any(|v| weak_val_eq(v, value))
    }

    pub fn add(&mut self, value: JsValue) {
        if !self.has(&value) {
            self.entries.push(value);
        }
    }

    pub fn delete(&mut self, value: &JsValue) -> bool {
        let len = self.entries.len();
        self.entries.retain(|v| !weak_val_eq(v, value));
        self.entries.len() != len
    }
}

impl Trace for JsWeakSet {
    fn trace(&self, tracer: &mut Tracer) {
        self.entries.trace(tracer);
    }
}

fn weak_val_eq(a: &JsValue, b: &JsValue) -> bool {
    match (a, b) {
        (JsValue::Object(x), JsValue::Object(y)) => Gc::ptr_eq(*x, *y),
        (JsValue::Array(x), JsValue::Array(y)) => Gc::ptr_eq(*x, *y),
        (JsValue::Map(x), JsValue::Map(y)) => Gc::ptr_eq(*x, *y),
        (JsValue::Set(x), JsValue::Set(y)) => Gc::ptr_eq(*x, *y),
        _ => false,
    }
}

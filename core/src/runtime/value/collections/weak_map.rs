use std::cell::RefCell;
use std::rc::Rc;

use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct JsWeakMap {
    pub entries: Vec<(JsValue, JsValue)>,
}

impl JsWeakMap {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn wrapped(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn get(&self, key: &JsValue) -> JsValue {
        for (k, v) in &self.entries {
            if weak_key_eq(k, key) {
                return v.clone();
            }
        }
        JsValue::Undefined
    }

    pub fn has(&self, key: &JsValue) -> bool {
        self.entries.iter().any(|(k, _)| weak_key_eq(k, key))
    }

    pub fn set(&mut self, key: JsValue, value: JsValue) {
        for entry in &mut self.entries {
            if weak_key_eq(&entry.0, &key) {
                entry.1 = value;
                return;
            }
        }
        self.entries.push((key, value));
    }

    pub fn delete(&mut self, key: &JsValue) -> bool {
        let len = self.entries.len();
        self.entries.retain(|(k, _)| !weak_key_eq(k, key));
        self.entries.len() != len
    }
}

fn weak_key_eq(a: &JsValue, b: &JsValue) -> bool {
    match (a, b) {
        (JsValue::Object(x), JsValue::Object(y)) => Rc::ptr_eq(x, y),
        (JsValue::Array(x), JsValue::Array(y)) => Rc::ptr_eq(x, y),
        (JsValue::Map(x), JsValue::Map(y)) => Rc::ptr_eq(x, y),
        (JsValue::Set(x), JsValue::Set(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

pub fn is_valid_weak_key(value: &JsValue) -> bool {
    matches!(
        value,
        JsValue::Object(_) | JsValue::Array(_) | JsValue::Map(_) | JsValue::Set(_)
    )
}

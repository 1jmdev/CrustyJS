use std::cell::RefCell;
use std::rc::Rc;

use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct JsSet {
    pub entries: Vec<JsValue>,
}

impl JsSet {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn wrapped(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn has(&self, value: &JsValue) -> bool {
        self.entries.iter().any(|v| set_val_eq(v, value))
    }

    pub fn add(&mut self, value: JsValue) {
        if !self.has(&value) {
            self.entries.push(value);
        }
    }

    pub fn delete(&mut self, value: &JsValue) -> bool {
        let len = self.entries.len();
        self.entries.retain(|v| !set_val_eq(v, value));
        self.entries.len() != len
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

fn set_val_eq(a: &JsValue, b: &JsValue) -> bool {
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
        (JsValue::Object(x), JsValue::Object(y)) => Rc::ptr_eq(x, y),
        (JsValue::Array(x), JsValue::Array(y)) => Rc::ptr_eq(x, y),
        _ => a == b,
    }
}

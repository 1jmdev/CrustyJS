use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::value::JsValue;

use super::JsObject;

pub fn get_property(obj: &Gc<GcCell<JsObject>>, key: &str) -> Option<JsValue> {
    let mut current = Some(*obj);
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(prop) = borrowed.properties.get(key) {
            return Some(prop.value.clone());
        }
        current = borrowed.prototype;
    }
    None
}

pub fn set_property(obj: &Gc<GcCell<JsObject>>, key: &str, value: JsValue) {
    obj.borrow_mut().set(key.to_string(), value);
}

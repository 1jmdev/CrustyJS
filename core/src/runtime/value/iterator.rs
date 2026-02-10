use crate::runtime::value::JsValue;
use crate::runtime::value::object::JsObject;

pub fn iter_result(value: JsValue, done: bool) -> JsValue {
    let mut obj = JsObject::new();
    obj.set("value".to_string(), value);
    obj.set("done".to_string(), JsValue::Boolean(done));
    JsValue::Object(obj.wrapped())
}

pub fn iter_done() -> JsValue {
    iter_result(JsValue::Undefined, true)
}

pub fn get_property_simple(value: &JsValue, key: &str) -> Option<JsValue> {
    match value {
        JsValue::Object(obj) => obj.borrow().get(key),
        _ => None,
    }
}

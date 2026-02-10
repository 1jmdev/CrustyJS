use crate::runtime::gc::Heap;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

pub fn iter_result(value: JsValue, done: bool, heap: &mut Heap) -> JsValue {
    let mut obj = JsObject::new();
    obj.set("value".to_string(), value);
    obj.set("done".to_string(), JsValue::Boolean(done));
    JsValue::Object(heap.alloc_cell(obj))
}

pub fn iter_done(heap: &mut Heap) -> JsValue {
    iter_result(JsValue::Undefined, true, heap)
}

pub fn get_property_simple(value: &JsValue, key: &str) -> Option<JsValue> {
    match value {
        JsValue::Object(obj) => obj.borrow().get(key),
        _ => None,
    }
}

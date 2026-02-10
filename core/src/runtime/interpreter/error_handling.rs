use crate::errors::RuntimeError;
use crate::runtime::gc::Heap;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct JsException {
    pub value: JsValue,
}

impl JsException {
    pub fn new(value: JsValue) -> Self {
        Self { value }
    }

    pub fn into_runtime_error(self) -> RuntimeError {
        RuntimeError::Thrown { value: self.value }
    }
}

pub fn create_error_object(message: JsValue, heap: &mut Heap) -> JsValue {
    let mut obj = JsObject::new();
    obj.set("name".to_string(), JsValue::String("Error".to_string()));
    obj.set(
        "message".to_string(),
        JsValue::String(message.to_js_string()),
    );
    JsValue::Object(heap.alloc_cell(obj))
}

use crate::errors::RuntimeError;
use crate::runtime::gc::Heap;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

use super::Interpreter;

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
    obj.set("constructor".to_string(), JsValue::Undefined);
    JsValue::Object(heap.alloc_cell(obj))
}

impl Interpreter {
    /// Create a typed error object (TypeError, ReferenceError, etc.)
    /// suitable for use as a Thrown value that can be caught by try/catch.
    pub(crate) fn create_typed_error_object(&mut self, error_type: &str, message: &str) -> JsValue {
        let mut obj = JsObject::new();
        obj.set("name".to_string(), JsValue::String(error_type.to_string()));
        obj.set("message".to_string(), JsValue::String(message.to_string()));
        let constructor = self.env.get(error_type).unwrap_or(JsValue::Undefined);
        obj.set("constructor".to_string(), constructor);
        // Set the constructor name so instanceof checks can work
        obj.set(
            "[[ErrorType]]".to_string(),
            JsValue::String(error_type.to_string()),
        );
        JsValue::Object(self.heap.alloc_cell(obj))
    }

    /// Throw a catchable TypeError
    pub(crate) fn throw_type_error(&mut self, message: &str) -> RuntimeError {
        let err_obj = self.create_typed_error_object("TypeError", message);
        RuntimeError::Thrown { value: err_obj }
    }
}

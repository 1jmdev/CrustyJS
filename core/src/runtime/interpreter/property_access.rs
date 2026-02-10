use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::value::object::prototype;
use crate::runtime::value::string_methods;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn get_property(
        &self,
        obj_val: &JsValue,
        key: &str,
    ) -> Result<JsValue, RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                Ok(prototype::get_property(obj, key).unwrap_or(JsValue::Undefined))
            }
            JsValue::Array(arr) => {
                let borrowed = arr.borrow();
                if key == "length" {
                    return Ok(JsValue::Number(borrowed.len() as f64));
                }
                if let Ok(idx) = key.parse::<usize>() {
                    return Ok(borrowed.get(idx));
                }
                Ok(JsValue::Undefined)
            }
            JsValue::String(s) => string_methods::resolve_string_property(s, key),
            _ => Err(RuntimeError::TypeError {
                message: format!("cannot access property '{key}' on {obj_val}"),
            }),
        }
    }

    pub(crate) fn set_property(
        &self,
        obj_val: &JsValue,
        key: &str,
        value: JsValue,
    ) -> Result<(), RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                prototype::set_property(obj, key, value);
                Ok(())
            }
            JsValue::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    arr.borrow_mut().set(idx, value);
                    Ok(())
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("cannot set property '{key}' on array"),
                    })
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("cannot set property '{key}' on {obj_val}"),
            }),
        }
    }
}

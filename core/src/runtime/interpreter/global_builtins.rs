use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{Param, Pattern};
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn init_builtins(&mut self) {
        let error_ctor = JsValue::Function {
            name: "Error".to_string(),
            params: vec![Param {
                pattern: Pattern::Identifier("message".to_string()),
                default: None,
            }],
            body: Vec::new(),
            closure_env: self.env.capture(),
        };
        self.env.define("Error".to_string(), error_ctor);
    }

    pub(crate) fn builtin_console_log_values(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let values: Vec<String> = args.iter().map(|v| v.to_string()).collect();
        let line = values.join(" ");
        println!("{line}");
        self.output.push(line);
        Ok(JsValue::Undefined)
    }

    pub(crate) fn builtin_object_create_values(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let proto = args.first().cloned().unwrap_or(JsValue::Null);
        let mut obj = JsObject::new();
        obj.prototype = match proto {
            JsValue::Object(parent) => Some(parent),
            JsValue::Null | JsValue::Undefined => None,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Object.create prototype must be object or null".to_string(),
                });
            }
        };
        Ok(JsValue::Object(obj.wrapped()))
    }
}

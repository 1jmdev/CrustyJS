use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::JsValue;

use crate::runtime::value::object::JsObject;

impl Interpreter {
    pub(crate) fn builtin_reflect_call(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "get" => self.reflect_get(args),
            "set" => self.reflect_set(args),
            "has" => self.reflect_has(args),
            "deleteProperty" => self.reflect_delete_property(args),
            "ownKeys" => self.reflect_own_keys(args),
            "apply" => self.reflect_apply(args),
            "construct" => self.reflect_construct(args),
            "getPrototypeOf" => self.reflect_get_prototype_of(args),
            _ => Err(RuntimeError::TypeError {
                message: format!("Reflect has no method '{method}'"),
            }),
        }
    }

    fn reflect_get(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let prop = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();
        self.get_property(&target, &prop)
    }

    fn reflect_set(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let prop = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();
        let value = args.get(2).cloned().unwrap_or(JsValue::Undefined);
        self.set_property(&target, &prop, value)?;
        Ok(JsValue::Boolean(true))
    }

    fn reflect_has(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let prop = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();
        self.eval_in_value(&prop, &target)
    }

    fn reflect_delete_property(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let prop = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();
        self.delete_property(&target, &prop)
    }

    fn reflect_own_keys(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let keys = self.object_keys(target)?;
        Ok(JsValue::Array(
            JsArray::new(keys.into_iter().map(JsValue::String).collect()).wrapped(),
        ))
    }

    fn reflect_apply(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let this_arg = args.get(1).cloned().unwrap_or(JsValue::Undefined);
        let arg_list = args.get(2).cloned().unwrap_or(JsValue::Undefined);
        let call_args = match arg_list {
            JsValue::Array(arr) => arr.borrow().elements.clone(),
            _ => Vec::new(),
        };
        self.call_function_with_this(&target, &call_args, Some(this_arg))
    }

    fn reflect_construct(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let arg_list = args.get(1).cloned().unwrap_or(JsValue::Undefined);
        let call_args = match arg_list {
            JsValue::Array(arr) => arr.borrow().elements.clone(),
            _ => Vec::new(),
        };
        // If target is a class constructor, do proper instantiation
        if let JsValue::Function { name, .. } = &target {
            if let Some(class_name) = name.strip_suffix("::constructor") {
                if let Some(class) = self.classes.get(class_name).cloned() {
                    let mut instance = JsObject::new();
                    instance.prototype = Some(class.prototype.clone());
                    let instance_value = JsValue::Object(instance.wrapped());
                    self.call_function_with_this(
                        &class.constructor,
                        &call_args,
                        Some(instance_value.clone()),
                    )?;
                    return Ok(instance_value);
                }
            }
        }
        self.call_function(&target, &call_args)
    }

    fn reflect_get_prototype_of(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        self.object_get_prototype_of(&target)
    }
}

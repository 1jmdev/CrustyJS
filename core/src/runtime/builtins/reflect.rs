use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn builtin_reflect(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = || args.first().cloned().unwrap_or(JsValue::Undefined);
        let prop = || {
            args.get(1)
                .cloned()
                .unwrap_or(JsValue::Undefined)
                .to_js_string()
        };

        match method {
            "get" => self.get_property(&target(), &prop()),
            "set" => {
                let val = args.get(2).cloned().unwrap_or(JsValue::Undefined);
                self.set_property(&target(), &prop(), val)?;
                Ok(JsValue::Boolean(true))
            }
            "has" => self.eval_in_value(&prop(), &target()),
            "deleteProperty" => self.delete_property(&target(), &prop()),
            "ownKeys" => {
                let keys = self.object_own_keys(target())?;
                let arr = JsArray::new(keys.into_iter().map(JsValue::String).collect());
                Ok(JsValue::Array(self.heap.alloc_cell(arr)))
            }
            "apply" => {
                let this_arg = args.get(1).cloned().unwrap_or(JsValue::Undefined);
                let call_args = match args.get(2).cloned().unwrap_or(JsValue::Undefined) {
                    JsValue::Array(arr) => arr.borrow().elements.clone(),
                    _ => Vec::new(),
                };
                self.call_function_with_this(&target(), &call_args, Some(this_arg))
            }
            "construct" => {
                let call_args = match args.get(1).cloned().unwrap_or(JsValue::Undefined) {
                    JsValue::Array(arr) => arr.borrow().elements.clone(),
                    _ => Vec::new(),
                };
                let t = target();
                if let JsValue::Function { name, .. } = &t {
                    if let Some(class_name) = name.strip_suffix("::constructor") {
                        if let Some(class) = self.classes.get(class_name).cloned() {
                            let mut instance = JsObject::new();
                            instance.prototype = Some(class.prototype);
                            let inst = JsValue::Object(self.heap.alloc_cell(instance));
                            self.call_function_with_this(
                                &class.constructor,
                                &call_args,
                                Some(inst.clone()),
                            )?;
                            return Ok(inst);
                        }
                    }
                }
                self.call_function(&t, &call_args)
            }
            "getPrototypeOf" => self.object_get_prototype_of(&target()),
            _ => Err(RuntimeError::TypeError {
                message: format!("Reflect.{method} is not a function"),
            }),
        }
    }
}

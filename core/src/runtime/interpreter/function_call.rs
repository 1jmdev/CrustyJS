use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_array_callback_method(
        &mut self,
        arr: &std::rc::Rc<std::cell::RefCell<JsArray>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let callback = args.first().ok_or_else(|| RuntimeError::TypeError {
            message: format!("{method} requires a callback argument"),
        })?;
        let elements = arr.borrow().elements.clone();

        match method {
            "map" => {
                let mut result = Vec::new();
                for elem in &elements {
                    let val = self.call_function(callback, std::slice::from_ref(elem))?;
                    result.push(val);
                }
                Ok(JsValue::Array(JsArray::new(result).wrapped()))
            }
            "filter" => {
                let mut result = Vec::new();
                for elem in &elements {
                    let val = self.call_function(callback, std::slice::from_ref(elem))?;
                    if val.to_boolean() {
                        result.push(elem.clone());
                    }
                }
                Ok(JsValue::Array(JsArray::new(result).wrapped()))
            }
            "forEach" => {
                for elem in &elements {
                    self.call_function(callback, std::slice::from_ref(elem))?;
                }
                Ok(JsValue::Undefined)
            }
            "reduce" => {
                let init = args.get(1).cloned().unwrap_or(JsValue::Undefined);
                let mut acc = init;
                for elem in &elements {
                    acc = self.call_function(callback, &[acc, elem.clone()])?;
                }
                Ok(acc)
            }
            "sort" => {
                let mut sorted = arr.borrow().elements.clone();
                if matches!(callback, JsValue::Undefined) {
                    sorted.sort_by(|a, b| a.to_js_string().cmp(&b.to_js_string()));
                } else {
                    sorted.sort_by(|a, b| {
                        let res = self
                            .call_function(callback, &[a.clone(), b.clone()])
                            .ok()
                            .map(|v| v.to_number())
                            .unwrap_or(0.0);
                        if res < 0.0 {
                            std::cmp::Ordering::Less
                        } else if res > 0.0 {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });
                }
                arr.borrow_mut().elements = sorted.clone();
                Ok(JsValue::Array(JsArray::new(sorted).wrapped()))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("array has no method '{method}'"),
            }),
        }
    }

    pub(crate) fn call_function(
        &mut self,
        func: &JsValue,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        self.call_function_with_this(func, args, None)
    }

    pub(crate) fn call_function_with_this(
        &mut self,
        func: &JsValue,
        args: &[JsValue],
        this_binding: Option<JsValue>,
    ) -> Result<JsValue, RuntimeError> {
        match func {
            JsValue::Function {
                params,
                body,
                closure_env,
                is_async,
                ..
            } => {
                if *is_async {
                    self.execute_async_function_body(params, body, closure_env, this_binding, args)
                } else {
                    self.execute_function_body(params, body, closure_env, this_binding, args)
                }
            }
            JsValue::NativeFunction { handler, .. } => {
                self.call_native_function(handler, args, this_binding)
            }
            other => Err(RuntimeError::NotAFunction {
                name: format!("{other}"),
            }),
        }
    }
}

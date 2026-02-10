use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::array::methods::call_array_method;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::prototype;
use crate::runtime::value::string_methods;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_member_call(
        &mut self,
        object: &Expr,
        property: &str,
        args: &[Expr],
        is_call: bool,
    ) -> Result<JsValue, RuntimeError> {
        if let Expr::Identifier(name) = object {
            if name == "console" && property == "log" {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_console_log_values(&arg_values);
            }
            if name == "Object" && property == "create" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_object_create_values(&arg_values);
            }
        }

        let obj_val = self.eval_expr(object)?;

        if let JsValue::String(ref s) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return string_methods::call_string_method(s, property, &arg_values);
            }
            return string_methods::resolve_string_property(s, property);
        }

        if let JsValue::Array(ref arr) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                if let Some(result) = call_array_method(arr, property, &arg_values)? {
                    return Ok(result);
                }
                return self.eval_array_callback_method(arr, property, &arg_values);
            }
            return self.get_property(&obj_val, property);
        }

        if !is_call {
            return self.get_property(&obj_val, property);
        }

        let arg_values = self.eval_call_args(args)?;
        let method = self.get_property(&obj_val, property)?;
        return self.call_function_with_this(&method, &arg_values, Some(obj_val));
    }

    fn eval_array_callback_method(
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
                ..
            } => {
                let params = params.clone();
                let body = body.clone();
                let captured = closure_env.clone();
                let saved_scopes = self.env.replace_scopes(captured);

                self.env.push_scope_with_this(this_binding);
                for (idx, param) in params.iter().enumerate() {
                    let mut value = args.get(idx).cloned().unwrap_or(JsValue::Undefined);
                    if matches!(value, JsValue::Undefined) {
                        if let Some(default_expr) = &param.default {
                            value = self.eval_expr(default_expr)?;
                        }
                    }
                    self.eval_pattern_binding(&param.pattern, value)?;
                }

                let mut result = JsValue::Undefined;
                let call_result = (|| -> Result<(), RuntimeError> {
                    for stmt in &body {
                        match self.eval_stmt(stmt)? {
                            super::ControlFlow::Return(val) => {
                                result = val;
                                break;
                            }
                            super::ControlFlow::None => {}
                        }
                    }
                    Ok(())
                })();

                self.env.pop_scope();
                self.env.replace_scopes(saved_scopes);
                call_result?;
                Ok(result)
            }
            other => Err(RuntimeError::NotAFunction {
                name: format!("{other}"),
            }),
        }
    }

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

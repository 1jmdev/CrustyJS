use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::array::methods::call_array_method;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::string_methods;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn init_builtins(&mut self) {
        let error_ctor = JsValue::Function {
            name: "Error".to_string(),
            params: vec!["message".to_string()],
            body: Vec::new(),
            closure_env: self.env.capture(),
        };
        self.env.define("Error".to_string(), error_ctor);
    }

    pub(crate) fn eval_member_call(
        &mut self,
        object: &Expr,
        property: &str,
        args: &[Expr],
        is_call: bool,
    ) -> Result<JsValue, RuntimeError> {
        if let Expr::Identifier(name) = object {
            if name == "console" && property == "log" {
                return self.builtin_console_log(args);
            }
        }

        let obj_val = self.eval_expr(object)?;

        if let JsValue::String(ref s) = obj_val {
            if is_call {
                let arg_values: Vec<JsValue> = args
                    .iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;
                return string_methods::call_string_method(s, property, &arg_values);
            }
            return string_methods::resolve_string_property(s, property);
        }

        if let JsValue::Array(ref arr) = obj_val {
            if is_call {
                let arg_values: Vec<JsValue> = args
                    .iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;
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

        Err(RuntimeError::TypeError {
            message: format!("cannot access property '{property}' on this value"),
        })
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
        match func {
            JsValue::Function {
                params,
                body,
                closure_env,
                ..
            } => {
                if params.len() != args.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: args.len(),
                    });
                }

                let params = params.clone();
                let body = body.clone();
                let captured = closure_env.clone();
                let saved_scopes = self.env.replace_scopes(captured);

                self.env.push_scope();
                for (param, value) in params.iter().zip(args) {
                    self.env.define(param.clone(), value.clone());
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
                let borrowed = obj.borrow();
                Ok(borrowed.get(key).unwrap_or(JsValue::Undefined))
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
                obj.borrow_mut().set(key.to_string(), value);
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

    fn builtin_console_log(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let values: Vec<String> = args
            .iter()
            .map(|a| self.eval_expr(a).map(|v| v.to_string()))
            .collect::<Result<_, _>>()?;

        let line = values.join(" ");
        println!("{line}");
        self.output.push(line);
        Ok(JsValue::Undefined)
    }
}

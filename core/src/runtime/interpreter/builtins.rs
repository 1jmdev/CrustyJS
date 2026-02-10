use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::array::methods::call_array_method;
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
            if name == "console" && matches!(property, "log" | "info" | "warn" | "error" | "debug")
            {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_console_log_values(&arg_values);
            }
            if name == "Object" && property == "create" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_object_create_values(&arg_values);
            }
            if name == "Object" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_object_static_call(property, &arg_values);
            }
            if name == "JSON" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_json_call(property, &arg_values);
            }
            if name == "Date" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_date_call(property, &arg_values);
            }
            if name == "Math" {
                if is_call {
                    let arg_values = self.eval_call_args(args)?;
                    return self.builtin_math_call(property, &arg_values);
                } else {
                    return match self.builtin_math_constant(property) {
                        Ok(v) => Ok(v),
                        Err(_) => {
                            let obj_val = self.env.get("Math")?;
                            self.get_property(&obj_val, property)
                        }
                    };
                }
            }
            if name == "Promise" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_promise_static_call(property, &arg_values);
            }
            if name == "performance" && is_call {
                return self.builtin_performance_call(property);
            }
            if name == "Symbol" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_symbol_static_call(property, &arg_values);
            }
            if name == "Symbol" && !is_call {
                return self.builtin_symbol_property(property);
            }
            if name == "Proxy" && property == "revocable" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_proxy_revocable(&arg_values);
            }
            if name == "Reflect" && is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_reflect_call(property, &arg_values);
            }
            if name == "Number" {
                if is_call {
                    let arg_values = self.eval_call_args(args)?;
                    return self.builtin_number_static_call(property, &arg_values);
                } else {
                    return self.builtin_number_static_property(property);
                }
            }
            if name == "Array" && property == "isArray" && is_call {
                let arg_values = self.eval_call_args(args)?;
                let val = arg_values.first().cloned().unwrap_or(JsValue::Undefined);
                return Ok(JsValue::Boolean(matches!(val, JsValue::Array(_))));
            }
        }

        let obj_val = self.eval_expr(object)?;

        if let JsValue::Symbol(ref sym) = obj_val {
            return match property {
                "toString" if is_call => Ok(JsValue::String(sym.to_string())),
                "description" if !is_call => Ok(match &sym.description {
                    Some(desc) => JsValue::String(desc.clone()),
                    None => JsValue::Undefined,
                }),
                _ => Err(RuntimeError::TypeError {
                    message: format!("cannot access '{property}' on symbol"),
                }),
            };
        }

        if let JsValue::String(ref s) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return string_methods::call_string_method(
                    s,
                    property,
                    &arg_values,
                    &mut self.heap,
                );
            }
            return string_methods::resolve_string_property(s, property);
        }

        if let JsValue::Array(ref arr) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                if let Some(result) = call_array_method(arr, property, &arg_values, &mut self.heap)?
                {
                    return Ok(result);
                }
                return self.eval_array_callback_method(arr, property, &arg_values);
            }
            return self.get_property(&obj_val, property);
        }

        if let JsValue::Promise(ref promise) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.builtin_promise_instance_call(promise, property, &arg_values);
            }
            return Ok(JsValue::Undefined);
        }

        if let JsValue::Map(ref map) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.call_map_method(map, property, &arg_values);
            }
            return self.get_property(&obj_val, property);
        }

        if let JsValue::Set(ref set) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.call_set_method(set, property, &arg_values);
            }
            return self.get_property(&obj_val, property);
        }

        if let JsValue::WeakMap(ref wm) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.call_weak_map_method(wm, property, &arg_values);
            }
            return Ok(JsValue::Undefined);
        }

        if let JsValue::WeakSet(ref ws) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.call_weak_set_method(ws, property, &arg_values);
            }
            return Ok(JsValue::Undefined);
        }

        if let JsValue::RegExp(ref re) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                return self.call_regexp_method(re, property, &arg_values);
            }
            return self.get_regexp_property(re, property);
        }

        if let JsValue::Proxy(_) = obj_val {
            if is_call {
                let arg_values = self.eval_call_args(args)?;
                let method = self.get_property(&obj_val, property)?;
                return self.call_function_with_this(&method, &arg_values, Some(obj_val));
            }
            return self.get_property(&obj_val, property);
        }

        if !is_call {
            return self.get_property(&obj_val, property);
        }

        let arg_values = self.eval_call_args(args)?;
        let method = self.get_property(&obj_val, property)?;
        self.call_function_with_this(&method, &arg_values, Some(obj_val))
    }
}

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
            let name = name.clone();
            if let Some(result) = self.dispatch_static(&name, property, args, is_call)? {
                return Ok(result);
            }
        }

        let receiver = self.eval_expr(object)?;
        let vals = if is_call {
            Some(self.eval_call_args(args)?)
        } else {
            None
        };
        self.dispatch_instance(&receiver, property, vals)
    }

    fn dispatch_static(
        &mut self,
        name: &str,
        property: &str,
        arg_exprs: &[Expr],
        is_call: bool,
    ) -> Result<Option<JsValue>, RuntimeError> {
        macro_rules! args {
            () => {
                self.eval_call_args(arg_exprs)?
            };
        }

        let v = match name {
            "console"
                if is_call && matches!(property, "log" | "info" | "warn" | "error" | "debug") =>
            {
                let a = args!();
                self.builtin_console_log(&a)?
            }
            "Object" if is_call => {
                let a = args!();
                self.builtin_object_static(property, &a)?
            }
            "JSON" if is_call => {
                let a = args!();
                self.builtin_json_call(property, &a)?
            }
            "Date" if is_call => self.builtin_date_static(property)?,
            "Math" if is_call => {
                let a = args!();
                self.builtin_math_call(property, &a)?
            }
            "Math" => self.builtin_math_constant(property).or_else(|_| {
                let obj = self.env.get("Math")?;
                self.get_property(&obj, property)
            })?,
            "performance" if is_call && property == "now" => self.builtin_performance_now(),
            "Promise" if is_call => {
                let a = args!();
                self.builtin_promise_static(property, &a)?
            }
            "Symbol" if is_call => {
                let a = args!();
                self.builtin_symbol_static(property, &a)?
            }
            "Symbol" => self.builtin_symbol_property(property)?,
            "Proxy" if is_call && property == "revocable" => {
                let a = args!();
                self.builtin_proxy_revocable(&a)?
            }
            "Reflect" if is_call => {
                let a = args!();
                self.builtin_reflect(property, &a)?
            }
            "Number" if is_call => {
                let a = args!();
                self.builtin_number_static(property, &a)?
            }
            "Number" => self.builtin_number_property(property)?,
            "Array" if is_call && property == "isArray" => {
                let a = args!();
                let val = a.into_iter().next().unwrap_or(JsValue::Undefined);
                JsValue::Boolean(matches!(val, JsValue::Array(_)))
            }
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub(crate) fn dispatch_instance(
        &mut self,
        receiver: &JsValue,
        property: &str,
        vals: Option<Vec<JsValue>>,
    ) -> Result<JsValue, RuntimeError> {
        let is_call = vals.is_some();

        match receiver.clone() {
            JsValue::Symbol(sym) => match property {
                "toString" if is_call => Ok(JsValue::String(sym.to_string())),
                "description" => Ok(sym
                    .description
                    .map(JsValue::String)
                    .unwrap_or(JsValue::Undefined)),
                _ => Err(RuntimeError::TypeError {
                    message: format!("cannot access '{property}' on Symbol"),
                }),
            },
            JsValue::String(s) => {
                if is_call {
                    string_methods::call_string_method(&s, property, &vals.unwrap(), &mut self.heap)
                } else {
                    string_methods::resolve_string_property(&s, property)
                }
            }
            JsValue::Array(arr) => {
                if is_call {
                    let a = vals.unwrap();
                    if let Some(r) = call_array_method(&arr, property, &a, &mut self.heap)? {
                        return Ok(r);
                    }
                    self.eval_array_callback_method(&arr, property, &a)
                } else {
                    self.get_property(receiver, property)
                }
            }
            JsValue::Promise(promise) => {
                if is_call {
                    self.builtin_promise_instance(&promise, property, &vals.unwrap())
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::Map(map) => {
                if is_call {
                    self.call_map_method(&map, property, &vals.unwrap())
                } else if property == "size" {
                    Ok(JsValue::Number(map.borrow().entries.len() as f64))
                } else {
                    self.get_property(receiver, property)
                }
            }
            JsValue::Set(set) => {
                if is_call {
                    self.call_set_method(&set, property, &vals.unwrap())
                } else if property == "size" {
                    Ok(JsValue::Number(set.borrow().entries.len() as f64))
                } else {
                    self.get_property(receiver, property)
                }
            }
            JsValue::WeakMap(wm) => {
                if is_call {
                    self.call_weak_map_method(&wm, property, &vals.unwrap())
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::WeakSet(ws) => {
                if is_call {
                    self.call_weak_set_method(&ws, property, &vals.unwrap())
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::RegExp(re) => {
                if is_call {
                    self.call_regexp_method(&re, property, &vals.unwrap())
                } else {
                    self.get_regexp_property(&re, property)
                }
            }
            JsValue::Proxy(_) | _ => {
                if is_call {
                    let method = self.get_property(receiver, property)?;
                    self.call_function_with_this(&method, &vals.unwrap(), Some(receiver.clone()))
                } else {
                    self.get_property(receiver, property)
                }
            }
        }
    }
}

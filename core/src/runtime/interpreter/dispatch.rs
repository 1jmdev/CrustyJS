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
            match name.as_str() {
                "console"
                    if is_call
                        && matches!(property, "log" | "info" | "warn" | "error" | "debug") =>
                {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_console_log(&vals);
                }
                "Object" if is_call => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_object_static(property, &vals);
                }
                "JSON" if is_call => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_json_call(property, &vals);
                }
                "Date" if is_call => {
                    return self.builtin_date_static(property);
                }
                "Math" if is_call => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_math_call(property, &vals);
                }
                "Math" => {
                    return self.builtin_math_constant(property).or_else(|_| {
                        let obj = self.env.get("Math")?;
                        self.get_property(&obj, property)
                    });
                }
                "performance" if is_call && property == "now" => {
                    return Ok(self.builtin_performance_now());
                }
                "Promise" if is_call => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_promise_static(property, &vals);
                }
                "Symbol" if is_call => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_symbol_static(property, &vals);
                }
                "Symbol" => return self.builtin_symbol_property(property),
                "Proxy" if is_call && property == "revocable" => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_proxy_revocable(&vals);
                }
                "Reflect" if is_call => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_reflect(property, &vals);
                }
                "Number" if is_call => {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_number_static(property, &vals);
                }
                "Number" => return self.builtin_number_property(property),
                "Array" if is_call && property == "isArray" => {
                    let vals = self.eval_call_args(args)?;
                    let val = vals.into_iter().next().unwrap_or(JsValue::Undefined);
                    return Ok(JsValue::Boolean(matches!(val, JsValue::Array(_))));
                }
                _ => {}
            }
        }

        let receiver = self.eval_expr(object)?;

        match &receiver.clone() {
            JsValue::Symbol(sym) => {
                let sym = sym.clone();
                return match property {
                    "toString" if is_call => Ok(JsValue::String(sym.to_string())),
                    "description" => Ok(sym
                        .description
                        .map(JsValue::String)
                        .unwrap_or(JsValue::Undefined)),
                    _ => Err(RuntimeError::TypeError {
                        message: format!("cannot access '{property}' on Symbol"),
                    }),
                };
            }
            JsValue::String(s) => {
                let s = s.clone();
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    return string_methods::call_string_method(&s, property, &vals, &mut self.heap);
                }
                return string_methods::resolve_string_property(&s, property);
            }
            JsValue::Array(arr) => {
                let arr = *arr;
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    if let Some(r) = call_array_method(&arr, property, &vals, &mut self.heap)? {
                        return Ok(r);
                    }
                    return self.eval_array_callback_method(&arr, property, &vals);
                }
                return self.get_property(&receiver, property);
            }
            JsValue::Promise(promise) => {
                let promise = *promise;
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    return self.builtin_promise_instance(&promise, property, &vals);
                }
                return Ok(JsValue::Undefined);
            }
            JsValue::Map(map) => {
                let map = *map;
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    return self.call_map_method(&map, property, &vals);
                }
                if property == "size" {
                    return Ok(JsValue::Number(map.borrow().entries.len() as f64));
                }
                return self.get_property(&receiver, property);
            }
            JsValue::Set(set) => {
                let set = *set;
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    return self.call_set_method(&set, property, &vals);
                }
                if property == "size" {
                    return Ok(JsValue::Number(set.borrow().entries.len() as f64));
                }
                return self.get_property(&receiver, property);
            }
            JsValue::WeakMap(wm) => {
                let wm = *wm;
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    return self.call_weak_map_method(&wm, property, &vals);
                }
                return Ok(JsValue::Undefined);
            }
            JsValue::WeakSet(ws) => {
                let ws = *ws;
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    return self.call_weak_set_method(&ws, property, &vals);
                }
                return Ok(JsValue::Undefined);
            }
            JsValue::RegExp(re) => {
                let re = *re;
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    return self.call_regexp_method(&re, property, &vals);
                }
                return self.get_regexp_property(&re, property);
            }
            JsValue::Proxy(_) => {
                if is_call {
                    let vals = self.eval_call_args(args)?;
                    let method = self.get_property(&receiver, property)?;
                    return self.call_function_with_this(&method, &vals, Some(receiver));
                }
                return self.get_property(&receiver, property);
            }
            _ => {}
        }

        if !is_call {
            return self.get_property(&receiver, property);
        }
        let vals = self.eval_call_args(args)?;
        let method = self.get_property(&receiver, property)?;
        self.call_function_with_this(&method, &vals, Some(receiver))
    }
}

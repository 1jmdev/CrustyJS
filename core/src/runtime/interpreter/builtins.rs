use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
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

        if !is_call {
            return self.get_property(&obj_val, property);
        }

        Err(RuntimeError::TypeError {
            message: format!("cannot access property '{property}' on this value"),
        })
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

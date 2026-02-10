use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::string_methods;
use crate::runtime::value::JsValue;

impl Interpreter {
    /// Handle member access and method calls.
    /// `is_call` distinguishes `obj.prop` (false) from `obj.method()` (true).
    pub(crate) fn eval_member_call(
        &mut self,
        object: &Expr,
        property: &str,
        args: &[Expr],
        is_call: bool,
    ) -> Result<JsValue, RuntimeError> {
        // console.log special case
        if let Expr::Identifier(name) = object {
            if name == "console" && property == "log" {
                return self.builtin_console_log(args);
            }
        }

        // Evaluate the object to get its value
        let obj_val = self.eval_expr(object)?;

        // String property/method access
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

        Err(RuntimeError::TypeError {
            message: format!("cannot access property '{property}' on this value"),
        })
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

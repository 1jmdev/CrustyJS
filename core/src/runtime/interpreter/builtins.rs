use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::JsValue;

impl Interpreter {
    /// Handle member access like `console.log(...)`.
    pub(crate) fn eval_member_call(
        &mut self,
        object: &Expr,
        property: &str,
        args: &[Expr],
    ) -> Result<JsValue, RuntimeError> {
        if let Expr::Identifier(name) = object {
            if name == "console" && property == "log" {
                return self.builtin_console_log(args);
            }
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

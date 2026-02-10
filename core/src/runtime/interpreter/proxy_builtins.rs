use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::proxy::JsProxy;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_new_proxy(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        if args.len() < 2 {
            return Err(RuntimeError::TypeError {
                message: "Proxy requires target and handler arguments".to_string(),
            });
        }
        let target = self.eval_expr(&args[0])?;
        let handler_val = self.eval_expr(&args[1])?;

        let handler = match &handler_val {
            JsValue::Object(obj) => obj.clone(),
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Proxy handler must be an object".to_string(),
                });
            }
        };

        let proxy = JsProxy::new(target, handler);
        Ok(JsValue::Proxy(proxy.wrapped()))
    }
}

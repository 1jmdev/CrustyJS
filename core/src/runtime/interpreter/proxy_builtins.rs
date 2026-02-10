use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::proxy::JsProxy;
use crate::runtime::value::{JsValue, NativeFunction};

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
            JsValue::Object(obj) => *obj,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Proxy handler must be an object".to_string(),
                });
            }
        };
        let proxy = JsProxy::new(target, handler);
        Ok(JsValue::Proxy(self.heap.alloc_cell(proxy)))
    }

    pub(crate) fn builtin_proxy_revocable(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let handler_val = args.get(1).cloned().unwrap_or(JsValue::Undefined);

        let handler = match &handler_val {
            JsValue::Object(obj) => *obj,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Proxy handler must be an object".to_string(),
                });
            }
        };

        let proxy = JsProxy::new(target, handler);
        let proxy_gc = self.heap.alloc_cell(proxy);
        let proxy_val = JsValue::Proxy(proxy_gc);

        let revoke_fn = JsValue::NativeFunction {
            name: "revoke".to_string(),
            handler: NativeFunction::ProxyRevoke(proxy_gc),
        };

        let mut result = JsObject::new();
        result.set("proxy".to_string(), proxy_val);
        result.set("revoke".to_string(), revoke_fn);
        Ok(JsValue::Object(self.heap.alloc_cell(result)))
    }
}

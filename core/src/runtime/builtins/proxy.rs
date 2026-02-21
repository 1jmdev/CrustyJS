use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::proxy::JsProxy;
use crate::runtime::value::{JsValue, NativeFunction};

impl Interpreter {
    pub(crate) fn eval_new_proxy(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        if args.len() < 2 {
            return Err(RuntimeError::TypeError {
                message: "Proxy requires target and handler arguments".into(),
            });
        }
        let target = self.eval_expr(&args[0])?;
        let handler_val = self.eval_expr(&args[1])?;
        let handler = match &handler_val {
            JsValue::Object(obj) => *obj,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Proxy handler must be an object".into(),
                })
            }
        };
        Ok(JsValue::Proxy(
            self.heap.alloc_cell(JsProxy::new(target, handler)),
        ))
    }

    pub(crate) fn builtin_proxy_revocable(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let handler = match args.get(1).cloned().unwrap_or(JsValue::Undefined) {
            JsValue::Object(obj) => obj,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Proxy handler must be an object".into(),
                })
            }
        };
        let proxy_gc = self.heap.alloc_cell(JsProxy::new(target, handler));
        let mut result = JsObject::new();
        result.set("proxy".into(), JsValue::Proxy(proxy_gc));
        result.set(
            "revoke".into(),
            JsValue::NativeFunction {
                name: "revoke".into(),
                handler: NativeFunction::ProxyRevoke(proxy_gc),
            },
        );
        Ok(JsValue::Object(self.heap.alloc_cell(result)))
    }
}

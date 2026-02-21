use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::symbol;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn builtin_symbol_static(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "for" => {
                let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
                Ok(JsValue::Symbol(self.symbol_registry.for_key(key)))
            }
            "keyFor" => {
                let sym = args.first().ok_or_else(|| RuntimeError::TypeError {
                    message: "Symbol.keyFor requires a symbol".into(),
                })?;
                match sym {
                    JsValue::Symbol(s) => Ok(match self.symbol_registry.key_for(s) {
                        Some(k) => JsValue::String(k),
                        None => JsValue::Undefined,
                    }),
                    _ => Err(RuntimeError::TypeError {
                        message: "Symbol.keyFor: argument must be a symbol".into(),
                    }),
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Symbol.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn builtin_symbol_property(&self, prop: &str) -> Result<JsValue, RuntimeError> {
        match prop {
            "iterator" => Ok(JsValue::Symbol(symbol::symbol_iterator())),
            "toPrimitive" => Ok(JsValue::Symbol(symbol::symbol_to_primitive())),
            "hasInstance" => Ok(JsValue::Symbol(symbol::symbol_has_instance())),
            "toStringTag" => Ok(JsValue::Symbol(symbol::symbol_to_string_tag())),
            _ => Err(RuntimeError::TypeError {
                message: format!("Symbol.{prop} is not defined"),
            }),
        }
    }
}

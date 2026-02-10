use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{Param, Pattern};
use crate::runtime::value::object::JsObject;
use crate::runtime::value::symbol;
use crate::runtime::value::{JsValue, NativeFunction};
use std::time::{SystemTime, UNIX_EPOCH};

impl Interpreter {
    pub(crate) fn init_builtins(&mut self) {
        let error_ctor = JsValue::Function {
            name: "Error".to_string(),
            params: vec![Param {
                pattern: Pattern::Identifier("message".to_string()),
                default: None,
            }],
            body: Vec::new(),
            closure_env: self.env.capture(),
            is_async: false,
            is_generator: false,
            source_path: self.module_stack.last().map(|p| p.display().to_string()),
            source_offset: 0,
            properties: None,
        };
        self.env.define("Error".to_string(), error_ctor);

        self.env.define(
            "setTimeout".to_string(),
            JsValue::NativeFunction {
                name: "setTimeout".to_string(),
                handler: NativeFunction::SetTimeout,
            },
        );
        self.env.define(
            "setInterval".to_string(),
            JsValue::NativeFunction {
                name: "setInterval".to_string(),
                handler: NativeFunction::SetInterval,
            },
        );
        self.env.define(
            "clearTimeout".to_string(),
            JsValue::NativeFunction {
                name: "clearTimeout".to_string(),
                handler: NativeFunction::ClearTimeout,
            },
        );
        self.env.define(
            "clearInterval".to_string(),
            JsValue::NativeFunction {
                name: "clearInterval".to_string(),
                handler: NativeFunction::ClearInterval,
            },
        );
        self.env.define(
            "requestAnimationFrame".to_string(),
            JsValue::NativeFunction {
                name: "requestAnimationFrame".to_string(),
                handler: NativeFunction::RequestAnimationFrame,
            },
        );
        self.env.define(
            "cancelAnimationFrame".to_string(),
            JsValue::NativeFunction {
                name: "cancelAnimationFrame".to_string(),
                handler: NativeFunction::CancelAnimationFrame,
            },
        );
        self.env.define(
            "queueMicrotask".to_string(),
            JsValue::NativeFunction {
                name: "queueMicrotask".to_string(),
                handler: NativeFunction::QueueMicrotask,
            },
        );
        self.env.define(
            "Symbol".to_string(),
            JsValue::NativeFunction {
                name: "Symbol".to_string(),
                handler: NativeFunction::SymbolConstructor,
            },
        );
    }

    pub(crate) fn builtin_console_log_values(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let values: Vec<String> = args.iter().map(|v| v.to_string()).collect();
        let line = values.join(" ");
        println!("{line}");
        self.output.push(line);
        Ok(JsValue::Undefined)
    }

    pub(crate) fn builtin_object_create_values(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let proto = args.first().cloned().unwrap_or(JsValue::Null);
        let mut obj = JsObject::new();
        obj.prototype = match proto {
            JsValue::Object(parent) => Some(parent),
            JsValue::Null | JsValue::Undefined => None,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Object.create prototype must be object or null".to_string(),
                });
            }
        };
        Ok(JsValue::Object(self.heap.alloc_cell(obj)))
    }

    pub(crate) fn builtin_math_constant(&self, property: &str) -> Result<JsValue, RuntimeError> {
        let value = match property {
            "PI" => std::f64::consts::PI,
            "E" => std::f64::consts::E,
            "LN2" => std::f64::consts::LN_2,
            "LN10" => std::f64::consts::LN_10,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: format!("Math has no property '{property}'"),
                });
            }
        };
        Ok(JsValue::Number(value))
    }

    pub(crate) fn builtin_math_call(
        &self,
        property: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let n = |idx: usize| {
            args.get(idx)
                .cloned()
                .unwrap_or(JsValue::Undefined)
                .to_number()
        };
        let value = match property {
            "floor" => n(0).floor(),
            "ceil" => n(0).ceil(),
            "round" => n(0).round(),
            "abs" => n(0).abs(),
            "max" => args
                .iter()
                .map(|v| v.to_number())
                .fold(f64::NEG_INFINITY, f64::max),
            "min" => args
                .iter()
                .map(|v| v.to_number())
                .fold(f64::INFINITY, f64::min),
            "sqrt" => n(0).sqrt(),
            "pow" => n(0).powf(n(1)),
            "sign" => n(0).signum(),
            "trunc" => n(0).trunc(),
            "log" => n(0).ln(),
            "random" => {
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0);
                (nanos as f64) / (u32::MAX as f64)
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    message: format!("Math has no method '{property}'"),
                });
            }
        };

        Ok(JsValue::Number(value))
    }

    pub(crate) fn builtin_performance_call(&self, property: &str) -> Result<JsValue, RuntimeError> {
        match property {
            "now" => Ok(JsValue::Number(
                self.start_time.elapsed().as_secs_f64() * 1000.0,
            )),
            _ => Err(RuntimeError::TypeError {
                message: format!("performance has no method '{property}'"),
            }),
        }
    }

    pub(crate) fn builtin_symbol_static_call(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "for" => {
                let key = args.first().map(|v| v.to_js_string()).unwrap_or_default();
                let sym = self.symbol_registry.for_key(key);
                Ok(JsValue::Symbol(sym))
            }
            "keyFor" => {
                let sym = args.first().ok_or_else(|| RuntimeError::TypeError {
                    message: "Symbol.keyFor requires a symbol argument".to_string(),
                })?;
                match sym {
                    JsValue::Symbol(s) => match self.symbol_registry.key_for(s) {
                        Some(key) => Ok(JsValue::String(key)),
                        None => Ok(JsValue::Undefined),
                    },
                    _ => Err(RuntimeError::TypeError {
                        message: "Symbol.keyFor requires a symbol argument".to_string(),
                    }),
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Symbol.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn builtin_symbol_property(&self, property: &str) -> Result<JsValue, RuntimeError> {
        match property {
            "iterator" => Ok(JsValue::Symbol(symbol::symbol_iterator())),
            "toPrimitive" => Ok(JsValue::Symbol(symbol::symbol_to_primitive())),
            "hasInstance" => Ok(JsValue::Symbol(symbol::symbol_has_instance())),
            "toStringTag" => Ok(JsValue::Symbol(symbol::symbol_to_string_tag())),
            _ => Err(RuntimeError::TypeError {
                message: format!("Symbol has no property '{property}'"),
            }),
        }
    }
}

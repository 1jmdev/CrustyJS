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

        self.env
            .define("NaN".to_string(), JsValue::Number(f64::NAN));
        self.env
            .define("Infinity".to_string(), JsValue::Number(f64::INFINITY));
        self.env.define("undefined".to_string(), JsValue::Undefined);

        self.define_native_fn("isNaN", NativeFunction::IsNaN);
        self.define_native_fn("isFinite", NativeFunction::IsFinite);
        self.define_native_fn("parseInt", NativeFunction::ParseInt);
        self.define_native_fn("parseFloat", NativeFunction::ParseFloat);
        self.define_native_fn("Number", NativeFunction::NumberCtor);
        self.define_native_fn("Boolean", NativeFunction::BooleanCtor);
        self.define_native_fn("String", NativeFunction::StringCtor);
        self.define_native_fn("Object", NativeFunction::ObjectCtor);

        for kind in &[
            "TypeError",
            "ReferenceError",
            "SyntaxError",
            "RangeError",
            "URIError",
            "EvalError",
        ] {
            self.define_native_fn(kind, NativeFunction::ErrorCtor(kind.to_string()));
        }

        self.define_native_fn("setTimeout", NativeFunction::SetTimeout);
        self.define_native_fn("setInterval", NativeFunction::SetInterval);
        self.define_native_fn("clearTimeout", NativeFunction::ClearTimeout);
        self.define_native_fn("clearInterval", NativeFunction::ClearInterval);
        self.define_native_fn(
            "requestAnimationFrame",
            NativeFunction::RequestAnimationFrame,
        );
        self.define_native_fn("cancelAnimationFrame", NativeFunction::CancelAnimationFrame);
        self.define_native_fn("queueMicrotask", NativeFunction::QueueMicrotask);
        self.define_native_fn("Symbol", NativeFunction::SymbolConstructor);

        let mut math_obj = JsObject::new();
        math_obj.set("PI".to_string(), JsValue::Number(std::f64::consts::PI));
        math_obj.set("E".to_string(), JsValue::Number(std::f64::consts::E));
        math_obj.set("LN2".to_string(), JsValue::Number(std::f64::consts::LN_2));
        math_obj.set("LN10".to_string(), JsValue::Number(std::f64::consts::LN_10));
        math_obj.set(
            "LOG2E".to_string(),
            JsValue::Number(std::f64::consts::LOG2_E),
        );
        math_obj.set(
            "LOG10E".to_string(),
            JsValue::Number(std::f64::consts::LOG10_E),
        );
        math_obj.set(
            "SQRT2".to_string(),
            JsValue::Number(std::f64::consts::SQRT_2),
        );
        math_obj.set(
            "SQRT1_2".to_string(),
            JsValue::Number(std::f64::consts::FRAC_1_SQRT_2),
        );

        // Add Math methods as NativeFunction values so typeof Math.xxx returns "function"
        let math_methods = [
            "abs", "floor", "ceil", "round", "trunc", "sqrt", "pow", "min", "max", "random", "log",
            "sign", "exp", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "log2", "log10",
            "cbrt", "hypot", "fround", "clz32", "imul",
        ];
        for method_name in &math_methods {
            math_obj.set(
                method_name.to_string(),
                JsValue::NativeFunction {
                    name: method_name.to_string(),
                    handler: NativeFunction::MathMethod(method_name.to_string()),
                },
            );
        }

        self.env.define(
            "Math".to_string(),
            JsValue::Object(self.heap.alloc_cell(math_obj)),
        );

        // Add global type constructor stubs for typeof/identity checks
        // Date as a function (Date() returns a string, new Date() returns an object)
        self.env.define(
            "Date".to_string(),
            JsValue::NativeFunction {
                name: "Date".to_string(),
                handler: NativeFunction::DateCtor,
            },
        );

        // RegExp
        self.env.define(
            "RegExp".to_string(),
            JsValue::NativeFunction {
                name: "RegExp".to_string(),
                handler: NativeFunction::RegExpCtor,
            },
        );

        // Function
        self.env.define(
            "Function".to_string(),
            JsValue::NativeFunction {
                name: "Function".to_string(),
                handler: NativeFunction::FunctionCtor,
            },
        );

        // Array as a function
        self.env.define(
            "Array".to_string(),
            JsValue::NativeFunction {
                name: "Array".to_string(),
                handler: NativeFunction::ArrayCtor,
            },
        );

        // RegExp
        self.env.define(
            "RegExp".to_string(),
            JsValue::NativeFunction {
                name: "RegExp".to_string(),
                handler: NativeFunction::NativeClassConstructor("RegExp".to_string()),
            },
        );

        // Function
        self.env.define(
            "Function".to_string(),
            JsValue::NativeFunction {
                name: "Function".to_string(),
                handler: NativeFunction::NativeClassConstructor("Function".to_string()),
            },
        );

        // Reflect as an object
        let reflect_obj = JsObject::new();
        self.env.define(
            "Reflect".to_string(),
            JsValue::Object(self.heap.alloc_cell(reflect_obj)),
        );

        // Array as a function
        self.env.define(
            "Array".to_string(),
            JsValue::NativeFunction {
                name: "Array".to_string(),
                handler: NativeFunction::NativeClassConstructor("Array".to_string()),
            },
        );

        // globalThis / this
        let global_obj = JsObject::new();
        let global_val = JsValue::Object(self.heap.alloc_cell(global_obj));
        self.env
            .define("globalThis".to_string(), global_val.clone());
        // Set `this` at global scope to refer to the global object
        self.env.set_global_this(global_val);
    }

    fn define_native_fn(&mut self, name: &str, handler: NativeFunction) {
        self.env.define(
            name.to_string(),
            JsValue::NativeFunction {
                name: name.to_string(),
                handler,
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
            "LOG2E" => std::f64::consts::LOG2_E,
            "LOG10E" => std::f64::consts::LOG10_E,
            "SQRT2" => std::f64::consts::SQRT_2,
            "SQRT1_2" => std::f64::consts::FRAC_1_SQRT_2,
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
            "max" => {
                if args.is_empty() {
                    f64::NEG_INFINITY
                } else {
                    args.iter()
                        .map(|v| v.to_number())
                        .fold(f64::NEG_INFINITY, f64::max)
                }
            }
            "min" => {
                if args.is_empty() {
                    f64::INFINITY
                } else {
                    args.iter()
                        .map(|v| v.to_number())
                        .fold(f64::INFINITY, f64::min)
                }
            }
            "sqrt" => n(0).sqrt(),
            "pow" => n(0).powf(n(1)),
            "sign" => {
                let v = n(0);
                if v.is_nan() {
                    f64::NAN
                } else if v == 0.0 {
                    v
                } else {
                    v.signum()
                }
            }
            "trunc" => n(0).trunc(),
            "log" => n(0).ln(),
            "log2" => n(0).log2(),
            "log10" => n(0).log10(),
            "exp" => n(0).exp(),
            "sin" => n(0).sin(),
            "cos" => n(0).cos(),
            "tan" => n(0).tan(),
            "asin" => n(0).asin(),
            "acos" => n(0).acos(),
            "atan" => n(0).atan(),
            "atan2" => n(0).atan2(n(1)),
            "cbrt" => n(0).cbrt(),
            "hypot" => {
                if args.is_empty() {
                    0.0
                } else {
                    args.iter()
                        .map(|v| v.to_number())
                        .fold(0.0f64, |acc, x| acc.hypot(x))
                }
            }
            "fround" => (n(0) as f32) as f64,
            "clz32" => {
                let v = n(0) as u32;
                v.leading_zeros() as f64
            }
            "imul" => {
                let a = n(0) as i32;
                let b = n(1) as i32;
                (a.wrapping_mul(b)) as f64
            }
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

    pub(crate) fn builtin_number_static_call(
        &self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "isNaN" => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(
                    matches!(val, JsValue::Number(n) if n.is_nan()),
                ))
            }
            "isFinite" => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(
                    matches!(val, JsValue::Number(n) if n.is_finite()),
                ))
            }
            "isInteger" => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(
                    matches!(val, JsValue::Number(n) if n.is_finite() && n == n.trunc()),
                ))
            }
            "isSafeInteger" => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(
                    matches!(val, JsValue::Number(n) if n.is_finite() && n == n.trunc() && n.abs() <= 9007199254740991.0),
                ))
            }
            "parseInt" => {
                let s = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_js_string();
                let radix = args.get(1).map(|v| v.to_number() as i32).unwrap_or(0);
                Ok(JsValue::Number(super::promise_runtime::parse_int_impl(
                    &s, radix,
                )))
            }
            "parseFloat" => {
                let s = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_js_string();
                let trimmed = s.trim();
                Ok(JsValue::Number(trimmed.parse::<f64>().unwrap_or(f64::NAN)))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Number.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn builtin_number_static_property(
        &self,
        property: &str,
    ) -> Result<JsValue, RuntimeError> {
        match property {
            "MAX_VALUE" => Ok(JsValue::Number(f64::MAX)),
            "MIN_VALUE" => Ok(JsValue::Number(f64::MIN_POSITIVE)),
            "MAX_SAFE_INTEGER" => Ok(JsValue::Number(9007199254740991.0)),
            "MIN_SAFE_INTEGER" => Ok(JsValue::Number(-9007199254740991.0)),
            "POSITIVE_INFINITY" => Ok(JsValue::Number(f64::INFINITY)),
            "NEGATIVE_INFINITY" => Ok(JsValue::Number(f64::NEG_INFINITY)),
            "NaN" => Ok(JsValue::Number(f64::NAN)),
            "EPSILON" => Ok(JsValue::Number(f64::EPSILON)),
            _ => Err(RuntimeError::TypeError {
                message: format!("Number has no property '{property}'"),
            }),
        }
    }
}

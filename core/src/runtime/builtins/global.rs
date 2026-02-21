use super::number::parse_int;
use crate::embedding::function_args::FunctionArgs;
use crate::errors::RuntimeError;
use crate::runtime::event_loop::Microtask;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::{JsValue, NativeFunction};
impl Interpreter {
    pub(crate) fn init_builtins(&mut self) {
        self.env.define("NaN".into(), JsValue::Number(f64::NAN));
        self.env
            .define("Infinity".into(), JsValue::Number(f64::INFINITY));
        self.env.define("undefined".into(), JsValue::Undefined);
        self.def_native("isNaN", NativeFunction::IsNaN);
        self.def_native("isFinite", NativeFunction::IsFinite);
        self.def_native("parseInt", NativeFunction::ParseInt);
        self.def_native("parseFloat", NativeFunction::ParseFloat);
        self.def_native("Number", NativeFunction::NumberCtor);
        self.def_native("Boolean", NativeFunction::BooleanCtor);
        self.def_native("String", NativeFunction::StringCtor);
        self.def_native("Object", NativeFunction::ObjectCtor);
        self.def_native("Date", NativeFunction::DateCtor);
        self.def_native("Symbol", NativeFunction::SymbolConstructor);
        self.def_native("Function", NativeFunction::FunctionCtor);
        self.def_native("Array", NativeFunction::ArrayCtor);
        self.def_native("RegExp", NativeFunction::RegExpCtor);
        for kind in &[
            "Error",
            "TypeError",
            "ReferenceError",
            "SyntaxError",
            "RangeError",
            "URIError",
            "EvalError",
        ] {
            self.def_native(kind, NativeFunction::ErrorCtor(kind.to_string()));
        }
        self.def_native("setTimeout", NativeFunction::SetTimeout);
        self.def_native("setInterval", NativeFunction::SetInterval);
        self.def_native("clearTimeout", NativeFunction::ClearTimeout);
        self.def_native("clearInterval", NativeFunction::ClearInterval);
        self.def_native(
            "requestAnimationFrame",
            NativeFunction::RequestAnimationFrame,
        );
        self.def_native("cancelAnimationFrame", NativeFunction::CancelAnimationFrame);
        self.def_native("queueMicrotask", NativeFunction::QueueMicrotask);
        self.init_math_object();
        self.env.define(
            "Reflect".into(),
            JsValue::Object(self.heap.alloc_cell(JsObject::new())),
        );
        let global_val = JsValue::Object(self.heap.alloc_cell(JsObject::new()));
        self.env.define("globalThis".into(), global_val.clone());
        self.env.set_global_this(global_val);
    }
    fn def_native(&mut self, name: &str, handler: NativeFunction) {
        self.env.define(
            name.into(),
            JsValue::NativeFunction {
                name: name.into(),
                handler,
            },
        );
    }
    fn init_math_object(&mut self) {
        use std::f64::consts;
        let mut math = JsObject::new();
        let constants = [
            ("PI", consts::PI),
            ("E", consts::E),
            ("LN2", consts::LN_2),
            ("LN10", consts::LN_10),
            ("LOG2E", consts::LOG2_E),
            ("LOG10E", consts::LOG10_E),
            ("SQRT2", consts::SQRT_2),
            ("SQRT1_2", consts::FRAC_1_SQRT_2),
        ];
        for (name, val) in constants {
            math.set(name.into(), JsValue::Number(val));
        }
        let methods = [
            "abs", "floor", "ceil", "round", "trunc", "sqrt", "cbrt", "exp", "log", "log2",
            "log10", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "pow", "fround",
            "clz32", "imul", "sign", "max", "min", "hypot", "random",
        ];
        for m in methods {
            math.set(
                m.into(),
                JsValue::NativeFunction {
                    name: m.into(),
                    handler: NativeFunction::MathMethod(m.into()),
                },
            );
        }
        self.env
            .define("Math".into(), JsValue::Object(self.heap.alloc_cell(math)));
    }
    pub(crate) fn call_native_function(
        &mut self,
        handler: &NativeFunction,
        args: &[JsValue],
        this: Option<JsValue>,
    ) -> Result<JsValue, RuntimeError> {
        match handler {
            NativeFunction::PromiseResolve(p) => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.settle_promise(p, false, val)
            }
            NativeFunction::PromiseReject(p) => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.settle_promise(p, true, val)
            }
            NativeFunction::SetTimeout => self.schedule_timer(args, false),
            NativeFunction::SetInterval => self.schedule_timer(args, true),
            NativeFunction::ClearTimeout | NativeFunction::ClearInterval => {
                let id = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_number();
                if id.is_finite() && id >= 0.0 {
                    self.event_loop.clear_timer(id as u64);
                }
                Ok(JsValue::Undefined)
            }
            NativeFunction::RequestAnimationFrame => {
                let cb = args
                    .first()
                    .cloned()
                    .ok_or_else(|| RuntimeError::TypeError {
                        message: "requestAnimationFrame requires callback".into(),
                    })?;
                Ok(JsValue::Number(
                    self.event_loop.schedule_animation_frame(cb) as f64,
                ))
            }
            NativeFunction::CancelAnimationFrame => {
                let id = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_number();
                if id.is_finite() && id >= 0.0 {
                    self.event_loop.cancel_animation_frame(id as u64);
                }
                Ok(JsValue::Undefined)
            }
            NativeFunction::QueueMicrotask => {
                let cb = args
                    .first()
                    .cloned()
                    .ok_or_else(|| RuntimeError::TypeError {
                        message: "queueMicrotask requires callback".into(),
                    })?;
                self.event_loop
                    .enqueue_microtask(Microtask::Callback { callback: cb });
                Ok(JsValue::Undefined)
            }
            NativeFunction::SymbolConstructor => {
                let desc = args.first().and_then(|v| match v {
                    JsValue::String(s) => Some(s.clone()),
                    JsValue::Undefined => None,
                    other => Some(other.to_js_string()),
                });
                Ok(JsValue::Symbol(
                    crate::runtime::value::symbol::JsSymbol::new(desc),
                ))
            }
            NativeFunction::Host(cb) => {
                let this_val = this.unwrap_or(JsValue::Undefined);
                cb.call(FunctionArgs::new(this_val, args.to_vec()))
            }
            NativeFunction::GeneratorNext(gc_gen) => {
                let gc_gen = *gc_gen;
                self.step_generator(&gc_gen)
            }
            NativeFunction::GeneratorReturn(gc_gen) => {
                let gc_gen = *gc_gen;
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                let mut g = gc_gen.borrow_mut();
                g.state = crate::runtime::value::generator::GeneratorState::Completed;
                g.yielded_values.clear();
                g.return_value = val.clone();
                drop(g);
                Ok(crate::runtime::value::iterator::iter_result(
                    val,
                    true,
                    &mut self.heap,
                ))
            }
            NativeFunction::GeneratorThrow => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                Err(RuntimeError::Thrown { value: val })
            }
            NativeFunction::GeneratorIterator => Ok(this.unwrap_or(JsValue::Undefined)),
            NativeFunction::NativeClassConstructor(name) => {
                self.construct_native_class(name, args, this)
            }
            NativeFunction::ProxyRevoke(proxy) => {
                proxy.borrow_mut().revoked = true;
                Ok(JsValue::Undefined)
            }
            NativeFunction::IsNaN => Ok(JsValue::Boolean(
                args.first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_number()
                    .is_nan(),
            )),
            NativeFunction::IsFinite => Ok(JsValue::Boolean(
                args.first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_number()
                    .is_finite(),
            )),
            NativeFunction::ParseInt => {
                let s = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_js_string();
                let radix = args.get(1).map(|v| v.to_number() as i32).unwrap_or(0);
                Ok(JsValue::Number(parse_int(&s, radix)))
            }
            NativeFunction::ParseFloat => {
                let s = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_js_string();
                Ok(JsValue::Number(s.trim().parse::<f64>().unwrap_or(f64::NAN)))
            }
            NativeFunction::NumberCtor => Ok(JsValue::Number(
                args.first()
                    .cloned()
                    .unwrap_or(JsValue::Number(0.0))
                    .to_number(),
            )),
            NativeFunction::BooleanCtor => Ok(JsValue::Boolean(
                args.first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_boolean(),
            )),
            NativeFunction::StringCtor => Ok(JsValue::String(
                args.first()
                    .cloned()
                    .unwrap_or(JsValue::String(String::new()))
                    .to_js_string(),
            )),
            NativeFunction::ObjectCtor => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                match val {
                    JsValue::Object(_) => Ok(val),
                    _ => Ok(JsValue::Object(self.heap.alloc_cell(JsObject::new()))),
                }
            }
            NativeFunction::ErrorCtor(kind) => {
                let msg = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_js_string();
                let mut obj = JsObject::new();
                obj.set("name".into(), JsValue::String(kind.clone()));
                obj.set("message".into(), JsValue::String(msg));
                let constructor = self.env.get(kind).unwrap_or(JsValue::Undefined);
                obj.set("constructor".into(), constructor);
                obj.set("[[ErrorType]]".into(), JsValue::String(kind.clone()));
                Ok(JsValue::Object(self.heap.alloc_cell(obj)))
            }
            NativeFunction::MathMethod(method) => {
                let m = method.clone();
                self.builtin_math_call(&m, args)
            }
            NativeFunction::DateCtor => {
                Ok(JsValue::String("Thu Jan 01 1970 00:00:00 GMT+0000".into()))
            }
            NativeFunction::RegExpCtor => {
                let pattern = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::String(String::new()));
                let flags_str = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();
                let flags = crate::runtime::value::regexp::RegExpFlags::from_str(&flags_str)
                    .map_err(|e| RuntimeError::TypeError { message: e })?;
                let re =
                    crate::runtime::value::regexp::JsRegExp::new(&pattern.to_js_string(), flags)
                        .map_err(|e| RuntimeError::TypeError { message: e })?;
                Ok(JsValue::RegExp(self.heap.alloc_cell(re)))
            }
            NativeFunction::FunctionCtor => Ok(JsValue::Function {
                name: "anonymous".into(),
                params: vec![],
                body: vec![],
                closure_env: self.env.capture(),
                is_async: false,
                is_generator: false,
                source_path: None,
                source_offset: 0,
                properties: None,
            }),
            NativeFunction::ArrayCtor => {
                let elements = if args.len() == 1 {
                    if let JsValue::Number(n) = &args[0] {
                        vec![JsValue::Undefined; (*n as usize).min(1 << 20)]
                    } else {
                        args.to_vec()
                    }
                } else {
                    args.to_vec()
                };
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(elements))))
            }
        }
    }
    fn schedule_timer(
        &mut self,
        args: &[JsValue],
        interval: bool,
    ) -> Result<JsValue, RuntimeError> {
        let cb = args
            .first()
            .cloned()
            .ok_or_else(|| RuntimeError::TypeError {
                message: "timer requires a callback".into(),
            })?;
        let delay = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Number(0.0))
            .to_number();
        let delay_ms = if delay.is_nan() || delay <= 0.0 {
            0
        } else {
            delay as u64
        };
        Ok(JsValue::Number(
            self.event_loop.schedule_timer(cb, delay_ms, interval) as f64,
        ))
    }
}

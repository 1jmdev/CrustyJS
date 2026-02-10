use super::Interpreter;
use crate::embedding::function_args::FunctionArgs;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::event_loop::Microtask;
use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::value::promise::{JsPromise, PromiseReaction, PromiseState};
use crate::runtime::value::{JsValue, NativeFunction};

impl Interpreter {
    pub(crate) fn call_native_function(
        &mut self,
        handler: &NativeFunction,
        args: &[JsValue],
        _this_binding: Option<JsValue>,
    ) -> Result<JsValue, RuntimeError> {
        match handler {
            NativeFunction::PromiseResolve(promise) => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.settle_promise(promise, false, value)
            }
            NativeFunction::PromiseReject(promise) => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.settle_promise(promise, true, value)
            }
            NativeFunction::SetTimeout => self.native_set_timeout(args, false),
            NativeFunction::SetInterval => self.native_set_timeout(args, true),
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
                let callback = args
                    .first()
                    .cloned()
                    .ok_or_else(|| RuntimeError::TypeError {
                        message: "requestAnimationFrame requires callback".to_string(),
                    })?;
                let id = self.event_loop.schedule_animation_frame(callback);
                Ok(JsValue::Number(id as f64))
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
                let callback = args
                    .first()
                    .cloned()
                    .ok_or_else(|| RuntimeError::TypeError {
                        message: "queueMicrotask requires callback".to_string(),
                    })?;
                self.event_loop
                    .enqueue_microtask(Microtask::Callback { callback });
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
            NativeFunction::Host(callback) => {
                let this = _this_binding.unwrap_or(JsValue::Undefined);
                let args = FunctionArgs::new(this, args.to_vec());
                callback.call(args)
            }
            NativeFunction::GeneratorNext(generator) => {
                self.step_generator(generator)
            }
            NativeFunction::GeneratorReturn(generator) => {
                let mut g = generator.borrow_mut();
                g.state = crate::runtime::value::generator::GeneratorState::Completed;
                g.yielded_values.clear();
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                g.return_value = value.clone();
                drop(g);
                Ok(crate::runtime::value::iterator::iter_result(
                    value,
                    true,
                    &mut self.heap,
                ))
            }
            NativeFunction::GeneratorThrow => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Err(RuntimeError::Thrown { value })
            }
            NativeFunction::GeneratorIterator => {
                Ok(_this_binding.unwrap_or(JsValue::Undefined))
            }
            NativeFunction::NativeClassConstructor(class_name) => {
                self.construct_native_class(class_name, args, _this_binding)
            }
            NativeFunction::ProxyRevoke(proxy) => {
                proxy.borrow_mut().revoked = true;
                Ok(JsValue::Undefined)
            }
            NativeFunction::IsNaN => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined).to_number();
                Ok(JsValue::Boolean(val.is_nan()))
            }
            NativeFunction::IsFinite => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined).to_number();
                Ok(JsValue::Boolean(val.is_finite()))
            }
            NativeFunction::ParseInt => {
                let s = args.first().cloned().unwrap_or(JsValue::Undefined).to_js_string();
                let radix = args.get(1).map(|v| v.to_number() as i32).unwrap_or(0);
                Ok(JsValue::Number(parse_int_impl(&s, radix)))
            }
            NativeFunction::ParseFloat => {
                let s = args.first().cloned().unwrap_or(JsValue::Undefined).to_js_string();
                let trimmed = s.trim();
                Ok(JsValue::Number(
                    trimmed.parse::<f64>().unwrap_or(f64::NAN),
                ))
            }
            NativeFunction::NumberCtor => {
                let val = args.first().cloned().unwrap_or(JsValue::Number(0.0));
                Ok(JsValue::Number(val.to_number()))
            }
            NativeFunction::BooleanCtor => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(val.to_boolean()))
            }
            NativeFunction::StringCtor => {
                let val = args.first().cloned().unwrap_or(JsValue::String(String::new()));
                Ok(JsValue::String(val.to_js_string()))
            }
            NativeFunction::ObjectCtor => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                match val {
                    JsValue::Null | JsValue::Undefined => {
                        let obj = crate::runtime::value::object::JsObject::new();
                        Ok(JsValue::Object(self.heap.alloc_cell(obj)))
                    }
                    JsValue::Object(_) => Ok(val),
                    _ => {
                        let obj = crate::runtime::value::object::JsObject::new();
                        Ok(JsValue::Object(self.heap.alloc_cell(obj)))
                    }
                }
            }
            NativeFunction::ErrorCtor(kind) => {
                let message = args.first().cloned().unwrap_or(JsValue::Undefined);
                let mut obj = crate::runtime::value::object::JsObject::new();
                obj.set("name".to_string(), JsValue::String(kind.clone()));
                obj.set(
                    "message".to_string(),
                    JsValue::String(message.to_js_string()),
                );
                obj.set("[[ErrorType]]".to_string(), JsValue::String(kind.clone()));
                Ok(JsValue::Object(self.heap.alloc_cell(obj)))
            }
            NativeFunction::MathMethod(method) => {
                self.builtin_math_call(method, args)
            }
            NativeFunction::DateCtor => {
                Ok(JsValue::String(
                    "Thu Jan 01 1970 00:00:00 GMT+0000".to_string(),
                ))
            }
            NativeFunction::RegExpCtor => {
                // RegExp(pattern) creates a RegExp object
                let pattern = args.first().cloned().unwrap_or(JsValue::String(String::new()));
                let flags_str = args.get(1).map(|v| v.to_js_string()).unwrap_or_default();
                let flags = crate::runtime::value::regexp::RegExpFlags::from_str(&flags_str)
                    .map_err(|e| RuntimeError::TypeError { message: e })?;
                let re = crate::runtime::value::regexp::JsRegExp::new(
                    &pattern.to_js_string(),
                    flags,
                ).map_err(|e| RuntimeError::TypeError { message: e })?;
                Ok(JsValue::RegExp(self.heap.alloc_cell(re)))
            }
            NativeFunction::FunctionCtor => {
                // Function() constructor - stub, return empty function
                Ok(JsValue::Function {
                    name: "anonymous".to_string(),
                    params: vec![],
                    body: vec![],
                    closure_env: self.env.capture(),
                    is_async: false,
                    is_generator: false,
                    source_path: None,
                    source_offset: 0,
                    properties: None,
                })
            }
            NativeFunction::ArrayCtor => {
                // Array(n) or Array(a, b, c)
                let elements = if args.len() == 1 {
                    if let JsValue::Number(n) = &args[0] {
                        let len = (*n as usize).min(1 << 20);
                        vec![JsValue::Undefined; len]
                    } else {
                        args.to_vec()
                    }
                } else {
                    args.to_vec()
                };
                Ok(JsValue::Array(
                    self.heap
                        .alloc_cell(crate::runtime::value::array::JsArray::new(elements)),
                ))
            }
        }
    }

    pub(crate) fn eval_new_promise(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let promise = self.heap.alloc_cell(JsPromise::pending());
        let executor_expr = args.first().ok_or_else(|| RuntimeError::TypeError {
            message: "Promise constructor requires an executor".to_string(),
        })?;
        let executor = self.eval_expr(executor_expr)?;

        let resolve = JsValue::NativeFunction {
            name: "resolve".to_string(),
            handler: NativeFunction::PromiseResolve(promise),
        };
        let reject = JsValue::NativeFunction {
            name: "reject".to_string(),
            handler: NativeFunction::PromiseReject(promise),
        };

        if let Err(err) = self.call_function(&executor, &[resolve, reject]) {
            let rejection = self.runtime_error_to_value(err);
            let _ = self.settle_promise(&promise, true, rejection)?;
        }

        Ok(JsValue::Promise(promise))
    }

    pub(crate) fn builtin_promise_static_call(
        &mut self,
        property: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match property {
            "resolve" => {
                if let Some(JsValue::Promise(promise)) = args.first() {
                    return Ok(JsValue::Promise(*promise));
                }
                let promise = self.heap.alloc_cell(JsPromise::pending());
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                let _ = self.settle_promise(&promise, false, value)?;
                Ok(JsValue::Promise(promise))
            }
            "reject" => {
                let promise = self.heap.alloc_cell(JsPromise::pending());
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                let _ = self.settle_promise(&promise, true, value)?;
                Ok(JsValue::Promise(promise))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Promise has no static method '{property}'"),
            }),
        }
    }

    pub(crate) fn builtin_promise_instance_call(
        &mut self,
        promise: &Gc<GcCell<JsPromise>>,
        property: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match property {
            "then" => {
                let on_fulfilled = args.first().cloned();
                let on_rejected = args.get(1).cloned();
                self.promise_then(promise, on_fulfilled, on_rejected)
            }
            "catch" => {
                let on_rejected = args.first().cloned();
                self.promise_then(promise, None, on_rejected)
            }
            "finally" => {
                if let Some(callback) = Self::normalize_handler(args.first().cloned())
                    && let Err(err) = self.call_function(&callback, &[])
                {
                    let rejected = self.heap.alloc_cell(JsPromise::pending());
                    let _ =
                        self.settle_promise(&rejected, true, self.runtime_error_to_value(err))?;
                    return Ok(JsValue::Promise(rejected));
                }
                Ok(JsValue::Promise(*promise))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Promise has no method '{property}'"),
            }),
        }
    }

    pub(crate) fn settle_promise(
        &mut self,
        promise: &Gc<GcCell<JsPromise>>,
        is_reject: bool,
        value: JsValue,
    ) -> Result<JsValue, RuntimeError> {
        if !is_reject && let JsValue::Promise(inner) = &value {
            if Gc::ptr_eq(*promise, *inner) {
                return self.settle_promise(
                    promise,
                    true,
                    JsValue::String("Cannot resolve promise with itself".to_string()),
                );
            }

            let passthrough = PromiseReaction {
                on_fulfilled: None,
                on_rejected: None,
                next: *promise,
            };

            let settled = {
                let borrowed = inner.borrow();
                match &borrowed.state {
                    PromiseState::Pending => None,
                    PromiseState::Fulfilled(v) => Some((false, v.clone())),
                    PromiseState::Rejected(v) => Some((true, v.clone())),
                }
            };

            if let Some((inner_reject, inner_value)) = settled {
                self.event_loop
                    .enqueue_microtask(Microtask::PromiseReaction {
                        reaction: Box::new(passthrough),
                        is_reject: inner_reject,
                        value: inner_value,
                    });
            } else {
                inner.borrow_mut().reactions.push(passthrough);
            }
            return Ok(JsValue::Undefined);
        }

        let reactions = {
            let mut borrowed = promise.borrow_mut();
            if !matches!(borrowed.state, PromiseState::Pending) {
                return Ok(JsValue::Undefined);
            }
            borrowed.state = if is_reject {
                PromiseState::Rejected(value.clone())
            } else {
                PromiseState::Fulfilled(value.clone())
            };
            std::mem::take(&mut borrowed.reactions)
        };

        for reaction in reactions {
            self.event_loop
                .enqueue_microtask(Microtask::PromiseReaction {
                    reaction: Box::new(reaction),
                    is_reject,
                    value: value.clone(),
                });
        }

        Ok(JsValue::Undefined)
    }

    fn promise_then(
        &mut self,
        promise: &Gc<GcCell<JsPromise>>,
        on_fulfilled: Option<JsValue>,
        on_rejected: Option<JsValue>,
    ) -> Result<JsValue, RuntimeError> {
        let next = self.heap.alloc_cell(JsPromise::pending());
        let reaction = PromiseReaction {
            on_fulfilled: Self::normalize_handler(on_fulfilled),
            on_rejected: Self::normalize_handler(on_rejected),
            next,
        };

        let settled = {
            let borrowed = promise.borrow();
            match &borrowed.state {
                PromiseState::Pending => None,
                PromiseState::Fulfilled(value) => Some((false, value.clone())),
                PromiseState::Rejected(value) => Some((true, value.clone())),
            }
        };

        if let Some((is_reject, settled_value)) = settled {
            self.event_loop
                .enqueue_microtask(Microtask::PromiseReaction {
                    reaction: Box::new(reaction),
                    is_reject,
                    value: settled_value,
                });
        } else {
            promise.borrow_mut().reactions.push(reaction);
        }

        Ok(JsValue::Promise(next))
    }

    pub(crate) fn run_promise_reaction(
        &mut self,
        reaction: PromiseReaction,
        is_reject: bool,
        value: JsValue,
    ) -> Result<(), RuntimeError> {
        let callback = if is_reject {
            reaction.on_rejected.clone()
        } else {
            reaction.on_fulfilled.clone()
        };

        match callback {
            Some(cb) => match self.call_function(&cb, std::slice::from_ref(&value)) {
                Ok(result) => self.resolve_then_result(&reaction.next, result),
                Err(err) => {
                    let rejection = self.runtime_error_to_value(err);
                    let _ = self.settle_promise(&reaction.next, true, rejection)?;
                    Ok(())
                }
            },
            None => {
                let _ = self.settle_promise(&reaction.next, is_reject, value)?;
                Ok(())
            }
        }
    }

    fn resolve_then_result(
        &mut self,
        next: &Gc<GcCell<JsPromise>>,
        result: JsValue,
    ) -> Result<(), RuntimeError> {
        if let JsValue::Promise(inner) = result {
            let passthrough = PromiseReaction {
                on_fulfilled: None,
                on_rejected: None,
                next: *next,
            };
            let settled = {
                let borrowed = inner.borrow();
                match &borrowed.state {
                    PromiseState::Pending => None,
                    PromiseState::Fulfilled(value) => Some((false, value.clone())),
                    PromiseState::Rejected(value) => Some((true, value.clone())),
                }
            };

            if let Some((is_reject, value)) = settled {
                self.run_promise_reaction(passthrough, is_reject, value)
            } else {
                inner.borrow_mut().reactions.push(passthrough);
                Ok(())
            }
        } else {
            let _ = self.settle_promise(next, false, result)?;
            Ok(())
        }
    }

    fn normalize_handler(handler: Option<JsValue>) -> Option<JsValue> {
        handler.and_then(|value| match value {
            JsValue::Function { .. } | JsValue::NativeFunction { .. } => Some(value),
            _ => None,
        })
    }

    fn runtime_error_to_value(&self, err: RuntimeError) -> JsValue {
        match err {
            RuntimeError::Thrown { value } => value,
            other => JsValue::String(other.to_string()),
        }
    }

    fn native_set_timeout(
        &mut self,
        args: &[JsValue],
        interval: bool,
    ) -> Result<JsValue, RuntimeError> {
        let callback = args
            .first()
            .cloned()
            .ok_or_else(|| RuntimeError::TypeError {
                message: "timer requires callback".to_string(),
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
        let id = self.event_loop.schedule_timer(callback, delay_ms, interval);
        Ok(JsValue::Number(id as f64))
    }
}

pub(crate) fn parse_int_impl(s: &str, radix: i32) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return f64::NAN;
    }
    let (negative, rest) = if let Some(s) = trimmed.strip_prefix('-') {
        (true, s)
    } else if let Some(s) = trimmed.strip_prefix('+') {
        (false, s)
    } else {
        (false, trimmed)
    };
    let radix = if radix == 0 {
        if rest.starts_with("0x") || rest.starts_with("0X") {
            16
        } else {
            10
        }
    } else {
        radix
    };
    if !(2..=36).contains(&radix) {
        return f64::NAN;
    }
    let digits = if radix == 16 {
        rest.strip_prefix("0x")
            .or_else(|| rest.strip_prefix("0X"))
            .unwrap_or(rest)
    } else {
        rest
    };
    let mut result: f64 = 0.0;
    let mut found = false;
    for ch in digits.chars() {
        let d = match ch.to_digit(radix as u32) {
            Some(d) => d,
            None => break,
        };
        found = true;
        result = result * (radix as f64) + (d as f64);
    }
    if !found {
        return f64::NAN;
    }
    if negative {
        -result
    } else {
        result
    }
}

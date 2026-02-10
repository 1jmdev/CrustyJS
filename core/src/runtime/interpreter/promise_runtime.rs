use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::event_loop::Microtask;
use crate::runtime::value::promise::{JsPromise, PromiseReaction, PromiseState};
use crate::runtime::value::{JsValue, NativeFunction};
use std::cell::RefCell;
use std::rc::Rc;

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
        }
    }

    pub(crate) fn eval_new_promise(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let promise = Rc::new(RefCell::new(JsPromise::pending()));
        let executor_expr = args.first().ok_or_else(|| RuntimeError::TypeError {
            message: "Promise constructor requires an executor".to_string(),
        })?;
        let executor = self.eval_expr(executor_expr)?;

        let resolve = JsValue::NativeFunction {
            name: "resolve".to_string(),
            handler: NativeFunction::PromiseResolve(promise.clone()),
        };
        let reject = JsValue::NativeFunction {
            name: "reject".to_string(),
            handler: NativeFunction::PromiseReject(promise.clone()),
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
                    return Ok(JsValue::Promise(promise.clone()));
                }
                let promise = Rc::new(RefCell::new(JsPromise::pending()));
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                let _ = self.settle_promise(&promise, false, value)?;
                Ok(JsValue::Promise(promise))
            }
            "reject" => {
                let promise = Rc::new(RefCell::new(JsPromise::pending()));
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
        promise: &Rc<RefCell<JsPromise>>,
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
                if let Some(callback) = Self::normalize_handler(args.first().cloned()) {
                    if let Err(err) = self.call_function(&callback, &[]) {
                        let rejected = Rc::new(RefCell::new(JsPromise::pending()));
                        let _ =
                            self.settle_promise(&rejected, true, self.runtime_error_to_value(err))?;
                        return Ok(JsValue::Promise(rejected));
                    }
                }
                Ok(JsValue::Promise(promise.clone()))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Promise has no method '{property}'"),
            }),
        }
    }

    pub(crate) fn settle_promise(
        &mut self,
        promise: &Rc<RefCell<JsPromise>>,
        is_reject: bool,
        value: JsValue,
    ) -> Result<JsValue, RuntimeError> {
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
                    reaction,
                    is_reject,
                    value: value.clone(),
                });
        }

        Ok(JsValue::Undefined)
    }

    fn promise_then(
        &mut self,
        promise: &Rc<RefCell<JsPromise>>,
        on_fulfilled: Option<JsValue>,
        on_rejected: Option<JsValue>,
    ) -> Result<JsValue, RuntimeError> {
        let next = Rc::new(RefCell::new(JsPromise::pending()));
        let reaction = PromiseReaction {
            on_fulfilled: Self::normalize_handler(on_fulfilled),
            on_rejected: Self::normalize_handler(on_rejected),
            next: next.clone(),
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
                    reaction,
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
        next: &Rc<RefCell<JsPromise>>,
        result: JsValue,
    ) -> Result<(), RuntimeError> {
        if let JsValue::Promise(inner) = result {
            let passthrough = PromiseReaction {
                on_fulfilled: None,
                on_rejected: None,
                next: next.clone(),
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

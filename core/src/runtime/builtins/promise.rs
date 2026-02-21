use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::event_loop::Microtask;
use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::promise::{JsPromise, PromiseReaction, PromiseState};
use crate::runtime::value::{JsValue, NativeFunction};

impl Interpreter {
    pub(crate) fn eval_new_promise(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let promise = self.heap.alloc_cell(JsPromise::pending());
        let executor = self.eval_expr(args.first().ok_or_else(|| RuntimeError::TypeError {
            message: "Promise constructor requires an executor".into(),
        })?)?;

        let resolve = JsValue::NativeFunction {
            name: "resolve".into(),
            handler: NativeFunction::PromiseResolve(promise),
        };
        let reject = JsValue::NativeFunction {
            name: "reject".into(),
            handler: NativeFunction::PromiseReject(promise),
        };

        if let Err(err) = self.call_function(&executor, &[resolve, reject]) {
            let val = self.error_to_value(err);
            let _ = self.settle_promise(&promise, true, val)?;
        }
        Ok(JsValue::Promise(promise))
    }

    pub(crate) fn builtin_promise_static(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "resolve" => {
                if let Some(JsValue::Promise(p)) = args.first() {
                    return Ok(JsValue::Promise(*p));
                }
                let p = self.heap.alloc_cell(JsPromise::pending());
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.settle_promise(&p, false, val)?;
                Ok(JsValue::Promise(p))
            }
            "reject" => {
                let p = self.heap.alloc_cell(JsPromise::pending());
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.settle_promise(&p, true, val)?;
                Ok(JsValue::Promise(p))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Promise.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn builtin_promise_instance(
        &mut self,
        promise: &Gc<GcCell<JsPromise>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "then" => {
                let on_fulfilled = Self::normalize_callback(args.first().cloned());
                let on_rejected = Self::normalize_callback(args.get(1).cloned());
                self.promise_then(promise, on_fulfilled, on_rejected)
            }
            "catch" => {
                let on_rejected = Self::normalize_callback(args.first().cloned());
                self.promise_then(promise, None, on_rejected)
            }
            "finally" => {
                if let Some(cb) = Self::normalize_callback(args.first().cloned()) {
                    if let Err(err) = self.call_function(&cb, &[]) {
                        let p = self.heap.alloc_cell(JsPromise::pending());
                        let val = self.error_to_value(err);
                        self.settle_promise(&p, true, val)?;
                        return Ok(JsValue::Promise(p));
                    }
                }
                Ok(JsValue::Promise(*promise))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Promise.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn settle_promise(
        &mut self,
        promise: &Gc<GcCell<JsPromise>>,
        is_reject: bool,
        value: JsValue,
    ) -> Result<JsValue, RuntimeError> {

        if !is_reject {
            if let JsValue::Promise(inner) = &value {
                if Gc::ptr_eq(*promise, *inner) {
                    return self.settle_promise(
                        promise,
                        true,
                        JsValue::String("Cannot resolve promise with itself".into()),
                    );
                }
                let passthrough = PromiseReaction {
                    on_fulfilled: None,
                    on_rejected: None,
                    next: *promise,
                };
                let settled = {
                    let b = inner.borrow();
                    match &b.state {
                        PromiseState::Pending => None,
                        PromiseState::Fulfilled(v) => Some((false, v.clone())),
                        PromiseState::Rejected(v) => Some((true, v.clone())),
                    }
                };
                if let Some((rej, val)) = settled {
                    self.event_loop
                        .enqueue_microtask(Microtask::PromiseReaction {
                            reaction: Box::new(passthrough),
                            is_reject: rej,
                            value: val,
                        });
                } else {
                    inner.borrow_mut().reactions.push(passthrough);
                }
                return Ok(JsValue::Undefined);
            }
        }

        let reactions = {
            let mut b = promise.borrow_mut();
            if !matches!(b.state, PromiseState::Pending) {
                return Ok(JsValue::Undefined);
            }
            b.state = if is_reject {
                PromiseState::Rejected(value.clone())
            } else {
                PromiseState::Fulfilled(value.clone())
            };
            std::mem::take(&mut b.reactions)
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
            on_fulfilled,
            on_rejected,
            next,
        };

        let settled = {
            let b = promise.borrow();
            match &b.state {
                PromiseState::Pending => None,
                PromiseState::Fulfilled(v) => Some((false, v.clone())),
                PromiseState::Rejected(v) => Some((true, v.clone())),
            }
        };

        if let Some((is_reject, val)) = settled {
            self.event_loop
                .enqueue_microtask(Microtask::PromiseReaction {
                    reaction: Box::new(reaction),
                    is_reject,
                    value: val,
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
                Ok(result) => self.resolve_promise_result(&reaction.next, result),
                Err(err) => {
                    let val = self.error_to_value(err);
                    self.settle_promise(&reaction.next, true, val)?;
                    Ok(())
                }
            },
            None => {
                self.settle_promise(&reaction.next, is_reject, value)?;
                Ok(())
            }
        }
    }

    fn resolve_promise_result(
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
                let b = inner.borrow();
                match &b.state {
                    PromiseState::Pending => None,
                    PromiseState::Fulfilled(v) => Some((false, v.clone())),
                    PromiseState::Rejected(v) => Some((true, v.clone())),
                }
            };
            if let Some((is_reject, val)) = settled {
                self.run_promise_reaction(passthrough, is_reject, val)
            } else {
                inner.borrow_mut().reactions.push(passthrough);
                Ok(())
            }
        } else {
            self.settle_promise(next, false, result)?;
            Ok(())
        }
    }

    fn normalize_callback(handler: Option<JsValue>) -> Option<JsValue> {
        handler.filter(|v| matches!(v, JsValue::Function { .. } | JsValue::NativeFunction { .. }))
    }

    pub(crate) fn error_to_value(&self, err: RuntimeError) -> JsValue {
        match err {
            RuntimeError::Thrown { value } => value,
            other => JsValue::String(other.to_string()),
        }
    }
}

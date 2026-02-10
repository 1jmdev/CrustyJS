use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::value::promise::PromiseState;
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
        }
    }

    pub(crate) fn settle_promise(
        &mut self,
        promise: &std::rc::Rc<std::cell::RefCell<crate::runtime::value::promise::JsPromise>>,
        is_reject: bool,
        value: JsValue,
    ) -> Result<JsValue, RuntimeError> {
        let mut borrowed = promise.borrow_mut();
        if !matches!(borrowed.state, PromiseState::Pending) {
            return Ok(JsValue::Undefined);
        }
        borrowed.state = if is_reject {
            PromiseState::Rejected(value)
        } else {
            PromiseState::Fulfilled(value)
        };
        Ok(JsValue::Undefined)
    }
}

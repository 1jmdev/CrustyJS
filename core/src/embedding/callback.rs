use std::sync::Arc;

use crate::embedding::function_args::FunctionArgs;
use crate::errors::RuntimeError;
use crate::runtime::value::JsValue;

pub trait NativeFunction: Send + Sync {
    fn call(&self, args: FunctionArgs) -> Result<JsValue, RuntimeError>;
}

#[derive(Clone)]
pub struct NativeFunctionBoxed {
    callback: Arc<dyn NativeFunction>,
}

impl std::fmt::Debug for NativeFunctionBoxed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NativeFunctionBoxed(..)")
    }
}

impl NativeFunctionBoxed {
    pub fn new<T: NativeFunction + 'static>(callback: T) -> Self {
        Self {
            callback: Arc::new(callback),
        }
    }

    pub fn from_closure<F>(callback: F) -> Self
    where
        F: Fn(FunctionArgs) -> Result<JsValue, RuntimeError> + Send + Sync + 'static,
    {
        Self::new(ClosureFunction { callback })
    }

    pub fn call(&self, args: FunctionArgs) -> Result<JsValue, RuntimeError> {
        self.callback.call(args)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.callback, &other.callback)
    }
}

struct ClosureFunction<F> {
    callback: F,
}

impl<F> NativeFunction for ClosureFunction<F>
where
    F: Fn(FunctionArgs) -> Result<JsValue, RuntimeError> + Send + Sync,
{
    fn call(&self, args: FunctionArgs) -> Result<JsValue, RuntimeError> {
        (self.callback)(args)
    }
}

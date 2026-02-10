pub mod array;
mod coercion;
mod display;
pub mod iterator;
pub mod object;
pub mod promise;
pub mod string_methods;
pub mod symbol;

pub use coercion::abstract_equals;

use std::cell::RefCell;
use std::rc::Rc;

use crate::embedding::callback::NativeFunctionBoxed;
use crate::parser::ast::{Param, Stmt};
use crate::runtime::environment::Scope;
use crate::runtime::gc::{Trace, Tracer};
use array::JsArray;
use object::JsObject;
use promise::JsPromise;
use symbol::JsSymbol;

#[derive(Debug, Clone)]
pub enum NativeFunction {
    PromiseResolve(Rc<RefCell<JsPromise>>),
    PromiseReject(Rc<RefCell<JsPromise>>),
    SetTimeout,
    SetInterval,
    ClearTimeout,
    ClearInterval,
    RequestAnimationFrame,
    CancelAnimationFrame,
    QueueMicrotask,
    SymbolConstructor,
    Host(NativeFunctionBoxed),
}

#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Function {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
        closure_env: Vec<Rc<RefCell<Scope>>>,
        is_async: bool,
        is_generator: bool,
        source_path: Option<String>,
        source_offset: usize,
    },
    NativeFunction {
        name: String,
        handler: NativeFunction,
    },
    Symbol(JsSymbol),
    Object(Rc<RefCell<JsObject>>),
    Array(Rc<RefCell<JsArray>>),
    Promise(Rc<RefCell<JsPromise>>),
}

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => a == b,
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (
                JsValue::NativeFunction {
                    handler: NativeFunction::PromiseResolve(a),
                    ..
                },
                JsValue::NativeFunction {
                    handler: NativeFunction::PromiseResolve(b),
                    ..
                },
            ) => Rc::ptr_eq(a, b),
            (
                JsValue::NativeFunction {
                    handler: NativeFunction::PromiseReject(a),
                    ..
                },
                JsValue::NativeFunction {
                    handler: NativeFunction::PromiseReject(b),
                    ..
                },
            ) => Rc::ptr_eq(a, b),
            (JsValue::Symbol(a), JsValue::Symbol(b)) => a == b,
            (JsValue::Object(a), JsValue::Object(b)) => Rc::ptr_eq(a, b),
            (JsValue::Array(a), JsValue::Array(b)) => Rc::ptr_eq(a, b),
            (JsValue::Promise(a), JsValue::Promise(b)) => Rc::ptr_eq(a, b),
            (
                JsValue::NativeFunction {
                    handler: NativeFunction::Host(a),
                    ..
                },
                JsValue::NativeFunction {
                    handler: NativeFunction::Host(b),
                    ..
                },
            ) => a.ptr_eq(b),
            _ => false,
        }
    }
}

impl Trace for NativeFunction {
    fn trace(&self, tracer: &mut Tracer) {
        match self {
            NativeFunction::PromiseResolve(promise) | NativeFunction::PromiseReject(promise) => {
                promise.borrow().trace(tracer);
            }
            NativeFunction::SetTimeout
            | NativeFunction::SetInterval
            | NativeFunction::ClearTimeout
            | NativeFunction::ClearInterval
            | NativeFunction::RequestAnimationFrame
            | NativeFunction::CancelAnimationFrame
            | NativeFunction::QueueMicrotask
            | NativeFunction::SymbolConstructor
            | NativeFunction::Host(_) => {}
        }
    }
}

impl Trace for JsValue {
    fn trace(&self, tracer: &mut Tracer) {
        match self {
            JsValue::Function { closure_env, .. } => {
                for scope in closure_env {
                    scope.borrow().trace(tracer);
                }
            }
            JsValue::NativeFunction { handler, .. } => handler.trace(tracer),
            JsValue::Object(object) => object.borrow().trace(tracer),
            JsValue::Array(array) => array.borrow().trace(tracer),
            JsValue::Promise(promise) => promise.borrow().trace(tracer),
            JsValue::Undefined
            | JsValue::Null
            | JsValue::Boolean(_)
            | JsValue::Number(_)
            | JsValue::String(_)
            | JsValue::Symbol(_) => {}
        }
    }
}

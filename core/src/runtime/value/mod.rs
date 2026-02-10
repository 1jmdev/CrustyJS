pub mod array;
mod coercion;
mod display;
pub mod object;
pub mod promise;
pub mod string_methods;

pub use coercion::abstract_equals;

use std::cell::RefCell;
use std::rc::Rc;

use crate::parser::ast::{Param, Stmt};
use crate::runtime::environment::Scope;
use array::JsArray;
use object::JsObject;
use promise::JsPromise;

#[derive(Debug, Clone)]
pub enum NativeFunction {
    PromiseResolve(Rc<RefCell<JsPromise>>),
    PromiseReject(Rc<RefCell<JsPromise>>),
    SetTimeout,
    SetInterval,
    ClearTimeout,
    ClearInterval,
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
    },
    NativeFunction {
        name: String,
        handler: NativeFunction,
    },
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
            (JsValue::Object(a), JsValue::Object(b)) => Rc::ptr_eq(a, b),
            (JsValue::Array(a), JsValue::Array(b)) => Rc::ptr_eq(a, b),
            (JsValue::Promise(a), JsValue::Promise(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

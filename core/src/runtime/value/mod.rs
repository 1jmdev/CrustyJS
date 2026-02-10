pub mod array;
mod coercion;
pub mod collections;
mod display;
pub mod generator;
pub mod iterator;
pub mod object;
pub mod promise;
pub mod proxy;
pub mod regexp;
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
use collections::map::JsMap;
use collections::set::JsSet;
use collections::weak_map::JsWeakMap;
use collections::weak_set::JsWeakSet;
use generator::JsGenerator;
use object::JsObject;
use promise::JsPromise;
use proxy::JsProxy;
use regexp::JsRegExp;
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
    GeneratorNext(Rc<RefCell<JsGenerator>>),
    GeneratorReturn(Rc<RefCell<JsGenerator>>),
    GeneratorThrow,
    GeneratorIterator,
    ProxyRevoke(Rc<RefCell<JsProxy>>),
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
    Map(Rc<RefCell<JsMap>>),
    Set(Rc<RefCell<JsSet>>),
    WeakMap(Rc<RefCell<JsWeakMap>>),
    WeakSet(Rc<RefCell<JsWeakSet>>),
    RegExp(Rc<RefCell<JsRegExp>>),
    Proxy(Rc<RefCell<JsProxy>>),
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
            (JsValue::Map(a), JsValue::Map(b)) => Rc::ptr_eq(a, b),
            (JsValue::Set(a), JsValue::Set(b)) => Rc::ptr_eq(a, b),
            (JsValue::WeakMap(a), JsValue::WeakMap(b)) => Rc::ptr_eq(a, b),
            (JsValue::WeakSet(a), JsValue::WeakSet(b)) => Rc::ptr_eq(a, b),
            (JsValue::RegExp(a), JsValue::RegExp(b)) => Rc::ptr_eq(a, b),
            (JsValue::Proxy(a), JsValue::Proxy(b)) => Rc::ptr_eq(a, b),
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
            | NativeFunction::GeneratorThrow
            | NativeFunction::GeneratorIterator
            | NativeFunction::Host(_) => {}
            NativeFunction::GeneratorNext(generator)
            | NativeFunction::GeneratorReturn(generator) => {
                let g = generator.borrow();
                for val in &g.yielded_values {
                    val.trace(tracer);
                }
                g.return_value.trace(tracer);
                for scope in &g.captured_env {
                    scope.borrow().trace(tracer);
                }
                for arg in &g.args {
                    arg.trace(tracer);
                }
                if let Some(this) = &g.this_binding {
                    this.trace(tracer);
                }
            }
            NativeFunction::ProxyRevoke(proxy) => {
                let p = proxy.borrow();
                p.target.trace(tracer);
                p.handler.borrow().trace(tracer);
            }
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
            JsValue::Map(map) => {
                for (k, v) in &map.borrow().entries {
                    k.trace(tracer);
                    v.trace(tracer);
                }
            }
            JsValue::Set(set) => {
                for v in &set.borrow().entries {
                    v.trace(tracer);
                }
            }
            JsValue::WeakMap(wm) => {
                for (k, v) in &wm.borrow().entries {
                    k.trace(tracer);
                    v.trace(tracer);
                }
            }
            JsValue::WeakSet(ws) => {
                for v in &ws.borrow().entries {
                    v.trace(tracer);
                }
            }
            JsValue::Undefined
            | JsValue::Null
            | JsValue::Boolean(_)
            | JsValue::Number(_)
            | JsValue::String(_)
            | JsValue::Symbol(_)
            | JsValue::RegExp(_) => {}
            JsValue::Proxy(proxy) => {
                let p = proxy.borrow();
                p.target.trace(tracer);
                p.handler.borrow().trace(tracer);
            }
        }
    }
}

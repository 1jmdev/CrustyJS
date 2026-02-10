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

use crate::embedding::callback::NativeFunctionBoxed;
use crate::parser::ast::{Param, Stmt};
use crate::runtime::environment::Scope;
use crate::runtime::gc::{Gc, GcCell, Trace, Tracer};
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
    PromiseResolve(Gc<GcCell<JsPromise>>),
    PromiseReject(Gc<GcCell<JsPromise>>),
    SetTimeout,
    SetInterval,
    ClearTimeout,
    ClearInterval,
    RequestAnimationFrame,
    CancelAnimationFrame,
    QueueMicrotask,
    SymbolConstructor,
    GeneratorNext(Gc<GcCell<JsGenerator>>),
    GeneratorReturn(Gc<GcCell<JsGenerator>>),
    GeneratorThrow,
    GeneratorIterator,
    ProxyRevoke(Gc<GcCell<JsProxy>>),
    Host(NativeFunctionBoxed),
    NativeClassConstructor(String),
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
        closure_env: Vec<Gc<GcCell<Scope>>>,
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
    Object(Gc<GcCell<JsObject>>),
    Array(Gc<GcCell<JsArray>>),
    Promise(Gc<GcCell<JsPromise>>),
    Map(Gc<GcCell<JsMap>>),
    Set(Gc<GcCell<JsSet>>),
    WeakMap(Gc<GcCell<JsWeakMap>>),
    WeakSet(Gc<GcCell<JsWeakSet>>),
    RegExp(Gc<GcCell<JsRegExp>>),
    Proxy(Gc<GcCell<JsProxy>>),
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
            ) => Gc::ptr_eq(*a, *b),
            (
                JsValue::NativeFunction {
                    handler: NativeFunction::PromiseReject(a),
                    ..
                },
                JsValue::NativeFunction {
                    handler: NativeFunction::PromiseReject(b),
                    ..
                },
            ) => Gc::ptr_eq(*a, *b),
            (JsValue::Symbol(a), JsValue::Symbol(b)) => a == b,
            (JsValue::Object(a), JsValue::Object(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::Array(a), JsValue::Array(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::Promise(a), JsValue::Promise(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::Map(a), JsValue::Map(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::Set(a), JsValue::Set(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::WeakMap(a), JsValue::WeakMap(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::WeakSet(a), JsValue::WeakSet(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::RegExp(a), JsValue::RegExp(b)) => Gc::ptr_eq(*a, *b),
            (JsValue::Proxy(a), JsValue::Proxy(b)) => Gc::ptr_eq(*a, *b),
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
            NativeFunction::PromiseResolve(p) | NativeFunction::PromiseReject(p) => {
                tracer.mark(*p);
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
            | NativeFunction::Host(_)
            | NativeFunction::NativeClassConstructor(_) => {}
            NativeFunction::GeneratorNext(g) | NativeFunction::GeneratorReturn(g) => {
                tracer.mark(*g);
            }
            NativeFunction::ProxyRevoke(p) => {
                tracer.mark(*p);
            }
        }
    }
}

impl Trace for JsValue {
    fn trace(&self, tracer: &mut Tracer) {
        match self {
            JsValue::Function { closure_env, .. } => {
                for scope in closure_env {
                    tracer.mark(*scope);
                }
            }
            JsValue::NativeFunction { handler, .. } => handler.trace(tracer),
            JsValue::Object(gc) => tracer.mark(*gc),
            JsValue::Array(gc) => tracer.mark(*gc),
            JsValue::Promise(gc) => tracer.mark(*gc),
            JsValue::Map(gc) => tracer.mark(*gc),
            JsValue::Set(gc) => tracer.mark(*gc),
            JsValue::WeakMap(gc) => tracer.mark(*gc),
            JsValue::WeakSet(gc) => tracer.mark(*gc),
            JsValue::RegExp(gc) => tracer.mark(*gc),
            JsValue::Proxy(gc) => tracer.mark(*gc),
            JsValue::Undefined
            | JsValue::Null
            | JsValue::Boolean(_)
            | JsValue::Number(_)
            | JsValue::String(_)
            | JsValue::Symbol(_) => {}
        }
    }
}

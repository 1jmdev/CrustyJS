use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::parser::ast::{Param, Stmt};
use crate::runtime::environment::Scope;
use crate::runtime::value::JsValue;

/// The state of a generator object.
#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorState {
    /// Created but `.next()` has not been called yet.
    Suspended,
    /// The generator body has been fully executed; all yields have been collected.
    Completed,
}

/// A JS generator object, returned by calling a `function*`.
///
/// Because CrustyJS uses a tree-walk interpreter, we cannot truly suspend
/// execution midway through statements.  Instead, on the first `.next()` call
/// we eagerly execute the **entire** generator body, collecting every yielded
/// value into a buffer.  Subsequent `.next()` calls drain values from that
/// buffer until it is empty, at which point we return `{ done: true }`.
///
/// The optional `return_value` captures the value produced by a `return`
/// statement inside the generator body (if any).
#[derive(Debug, Clone)]
pub struct JsGenerator {
    /// The generator function's formal parameters.
    pub params: Vec<Param>,
    /// The generator function's body statements.
    pub body: Vec<Stmt>,
    /// The captured closure environment at the time the generator was created.
    pub closure_env: Vec<Rc<RefCell<Scope>>>,
    /// Current state of the generator.
    pub state: GeneratorState,
    /// Buffer of yielded values (filled eagerly on first `.next()`).
    pub yielded_values: VecDeque<JsValue>,
    /// The value produced by the generator's `return` statement (if any).
    pub return_value: JsValue,
}

impl JsGenerator {
    pub fn new(params: Vec<Param>, body: Vec<Stmt>, closure_env: Vec<Rc<RefCell<Scope>>>) -> Self {
        Self {
            params,
            body,
            closure_env,
            state: GeneratorState::Suspended,
            yielded_values: VecDeque::new(),
            return_value: JsValue::Undefined,
        }
    }

    pub fn wrapped(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

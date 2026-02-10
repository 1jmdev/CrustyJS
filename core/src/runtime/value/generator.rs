use std::collections::VecDeque;

use crate::parser::ast::{Param, Stmt};
use crate::runtime::environment::Scope;
use crate::runtime::gc::{Gc, GcCell, Trace, Tracer};
use crate::runtime::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorState {
    SuspendedStart,
    Executing,
    Completed,
}

#[derive(Debug, Clone)]
pub struct JsGenerator {
    pub state: GeneratorState,
    pub body: Vec<Stmt>,
    pub params: Vec<Param>,
    pub captured_env: Vec<Gc<GcCell<Scope>>>,
    pub this_binding: Option<JsValue>,
    pub args: Vec<JsValue>,
    pub yielded_values: VecDeque<JsValue>,
    pub return_value: JsValue,
}

impl JsGenerator {
    pub fn new(
        params: Vec<Param>,
        body: Vec<Stmt>,
        captured_env: Vec<Gc<GcCell<Scope>>>,
        this_binding: Option<JsValue>,
        args: Vec<JsValue>,
    ) -> Self {
        Self {
            state: GeneratorState::SuspendedStart,
            body,
            params,
            captured_env,
            this_binding,
            args,
            yielded_values: VecDeque::new(),
            return_value: JsValue::Undefined,
        }
    }

    pub fn from_values(items: VecDeque<JsValue>) -> Self {
        Self {
            state: GeneratorState::Completed,
            body: Vec::new(),
            params: Vec::new(),
            captured_env: Vec::new(),
            this_binding: None,
            args: Vec::new(),
            yielded_values: items,
            return_value: JsValue::Undefined,
        }
    }
}

impl Trace for JsGenerator {
    fn trace(&self, tracer: &mut Tracer) {
        for scope in &self.captured_env {
            tracer.mark(*scope);
        }
        self.this_binding.trace(tracer);
        for arg in &self.args {
            arg.trace(tracer);
        }
        for val in &self.yielded_values {
            val.trace(tracer);
        }
        self.return_value.trace(tracer);
    }
}

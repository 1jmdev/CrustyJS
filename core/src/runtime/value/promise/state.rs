use crate::runtime::gc::{Trace, Tracer};
use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub enum PromiseState {
    Pending,
    Fulfilled(JsValue),
    Rejected(JsValue),
}

impl Trace for PromiseState {
    fn trace(&self, tracer: &mut Tracer) {
        match self {
            PromiseState::Pending => {}
            PromiseState::Fulfilled(v) | PromiseState::Rejected(v) => v.trace(tracer),
        }
    }
}

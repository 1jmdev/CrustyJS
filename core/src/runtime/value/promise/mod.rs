mod state;

use crate::runtime::gc::{Gc, GcCell, Trace, Tracer};
use crate::runtime::value::JsValue;

pub use state::PromiseState;

#[derive(Debug, Clone)]
pub struct PromiseReaction {
    pub on_fulfilled: Option<JsValue>,
    pub on_rejected: Option<JsValue>,
    pub next: Gc<GcCell<JsPromise>>,
}

#[derive(Debug, Clone)]
pub struct JsPromise {
    pub state: PromiseState,
    pub reactions: Vec<PromiseReaction>,
}

impl JsPromise {
    pub fn pending() -> Self {
        Self {
            state: PromiseState::Pending,
            reactions: Vec::new(),
        }
    }
}

impl Trace for PromiseReaction {
    fn trace(&self, tracer: &mut Tracer) {
        self.on_fulfilled.trace(tracer);
        self.on_rejected.trace(tracer);
        tracer.mark(self.next);
    }
}

impl Trace for JsPromise {
    fn trace(&self, tracer: &mut Tracer) {
        self.reactions.trace(tracer);
    }
}

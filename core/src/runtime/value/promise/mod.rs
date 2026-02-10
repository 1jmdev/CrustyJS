mod state;

use std::cell::RefCell;
use std::rc::Rc;

use crate::runtime::value::JsValue;

pub use state::PromiseState;

#[derive(Debug, Clone)]
pub struct PromiseReaction {
    pub on_fulfilled: Option<JsValue>,
    pub on_rejected: Option<JsValue>,
    pub next: Rc<RefCell<JsPromise>>,
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

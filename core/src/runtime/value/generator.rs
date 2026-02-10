use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::runtime::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub enum GeneratorState {
    Suspended,
    Completed,
}

#[derive(Debug, Clone)]
pub struct JsGenerator {
    pub state: GeneratorState,
    pub yielded_values: VecDeque<JsValue>,
    pub return_value: JsValue,
}

impl JsGenerator {
    pub fn new() -> Self {
        Self {
            state: GeneratorState::Suspended,
            yielded_values: VecDeque::new(),
            return_value: JsValue::Undefined,
        }
    }

    pub fn wrapped(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

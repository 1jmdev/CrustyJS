pub mod methods;

use super::JsValue;
use crate::runtime::gc::{Trace, Tracer};

#[derive(Debug, Clone)]
pub struct JsArray {
    pub elements: Vec<JsValue>,
}

impl JsArray {
    pub fn new(elements: Vec<JsValue>) -> Self {
        Self { elements }
    }

    pub fn get(&self, index: usize) -> JsValue {
        self.elements
            .get(index)
            .cloned()
            .unwrap_or(JsValue::Undefined)
    }

    pub fn set(&mut self, index: usize, value: JsValue) {
        // Guard against absurdly large sparse arrays that would exhaust memory
        const MAX_ARRAY_LENGTH: usize = 4 * 1024 * 1024; // 4M elements
        if index >= MAX_ARRAY_LENGTH {
            return;
        }
        if index >= self.elements.len() {
            self.elements.resize(index + 1, JsValue::Undefined);
        }
        self.elements[index] = value;
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

impl Trace for JsArray {
    fn trace(&self, tracer: &mut Tracer) {
        self.elements.trace(tracer);
    }
}

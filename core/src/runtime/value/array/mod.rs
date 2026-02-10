pub mod methods;

use std::cell::RefCell;
use std::rc::Rc;

use super::JsValue;

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
        if index >= self.elements.len() {
            self.elements.resize(index + 1, JsValue::Undefined);
        }
        self.elements[index] = value;
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn wrapped(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

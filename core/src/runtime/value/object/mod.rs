pub mod exotic;
mod property;
pub mod property_descriptor;
pub mod prototype;
pub mod slots;

pub use property::Property;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::JsValue;
use super::symbol::JsSymbol;
use crate::runtime::gc::{Trace, Tracer};

#[derive(Debug, Clone)]
pub struct JsObject {
    pub properties: HashMap<String, Property>,
    pub symbol_properties: HashMap<u64, (JsSymbol, Property)>,
    pub prototype: Option<Rc<RefCell<JsObject>>>,
}

impl Default for JsObject {
    fn default() -> Self {
        Self::new()
    }
}

impl JsObject {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            symbol_properties: HashMap::new(),
            prototype: None,
        }
    }

    pub fn get(&self, key: &str) -> Option<JsValue> {
        self.properties.get(key).map(|p| p.value.clone())
    }

    pub fn get_symbol(&self, sym: &JsSymbol) -> Option<JsValue> {
        self.symbol_properties
            .get(&sym.id)
            .map(|(_, p)| p.value.clone())
    }

    pub fn set(&mut self, key: String, value: JsValue) {
        if let Some(existing) = self.properties.get_mut(&key) {
            existing.value = value;
            return;
        }
        self.properties.insert(key, Property::new(value));
    }

    pub fn set_symbol(&mut self, sym: JsSymbol, value: JsValue) {
        self.symbol_properties
            .insert(sym.id, (sym, Property::new(value)));
    }

    pub fn set_getter(&mut self, key: String, getter: JsValue) {
        if let Some(existing) = self.properties.get_mut(&key) {
            existing.getter = Some(getter);
            return;
        }
        self.properties.insert(key, Property::with_getter(getter));
    }

    pub fn set_setter(&mut self, key: String, setter: JsValue) {
        if let Some(existing) = self.properties.get_mut(&key) {
            existing.setter = Some(setter);
            return;
        }
        self.properties.insert(key, Property::with_setter(setter));
    }

    pub fn wrapped(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }
}

impl Trace for JsObject {
    fn trace(&self, tracer: &mut Tracer) {
        for property in self.properties.values() {
            property.value.trace(tracer);
        }
        for (_, property) in self.symbol_properties.values() {
            property.value.trace(tracer);
        }

        if let Some(prototype) = &self.prototype {
            prototype.borrow().trace(tracer);
        }
    }
}

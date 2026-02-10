use std::collections::HashMap;

use crate::runtime::value::JsValue;

#[derive(Default, Clone)]
pub struct EventTarget {
    listeners: HashMap<String, Vec<JsValue>>,
}

impl EventTarget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_listener(&mut self, event_type: impl Into<String>, callback: JsValue) {
        self.listeners
            .entry(event_type.into())
            .or_default()
            .push(callback);
    }

    pub fn remove_event_listener(&mut self, event_type: &str, callback: &JsValue) {
        if let Some(listeners) = self.listeners.get_mut(event_type) {
            listeners.retain(|listener| !same_listener(listener, callback));
        }
    }

    pub fn listeners_for(&self, event_type: &str) -> Vec<JsValue> {
        self.listeners.get(event_type).cloned().unwrap_or_default()
    }
}

fn same_listener(a: &JsValue, b: &JsValue) -> bool {
    match (a, b) {
        (
            JsValue::Function {
                name: a_name,
                source_path: a_path,
                source_offset: a_offset,
                ..
            },
            JsValue::Function {
                name: b_name,
                source_path: b_path,
                source_offset: b_offset,
                ..
            },
        ) => a_name == b_name && a_path == b_path && a_offset == b_offset,
        _ => a == b,
    }
}

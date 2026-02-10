use crate::runtime::gc::{ErasedGc, Trace, Tracer};

#[derive(Debug, Clone)]
pub struct JsWeakSet {
    pub entries: Vec<ErasedGc>,
}

impl JsWeakSet {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn has(&self, key: ErasedGc) -> bool {
        self.entries.iter().any(|k| *k == key)
    }

    pub fn add(&mut self, key: ErasedGc) {
        if !self.has(key) {
            self.entries.push(key);
        }
    }

    pub fn delete(&mut self, key: ErasedGc) -> bool {
        let len = self.entries.len();
        self.entries.retain(|k| *k != key);
        self.entries.len() != len
    }
}

impl Trace for JsWeakSet {
    fn trace(&self, _tracer: &mut Tracer) {}
}

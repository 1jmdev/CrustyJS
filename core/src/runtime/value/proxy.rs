use super::object::JsObject;
use super::JsValue;
use crate::runtime::gc::{Gc, GcCell, Heap, Trace, Tracer};

#[derive(Debug, Clone)]
pub struct JsProxy {
    pub target: JsValue,
    pub handler: Gc<GcCell<JsObject>>,
    pub revoked: bool,
}

impl JsProxy {
    pub fn new(target: JsValue, handler: Gc<GcCell<JsObject>>) -> Self {
        Self {
            target,
            handler,
            revoked: false,
        }
    }

    pub fn check_revoked(&self) -> Result<(), String> {
        if self.revoked {
            Err("Cannot perform operation on a revoked proxy".to_string())
        } else {
            Ok(())
        }
    }

    pub fn get_trap(&self, trap_name: &str, heap: &Heap) -> Option<JsValue> {
        let handler = heap.borrow(self.handler);
        handler
            .get(trap_name)
            .filter(|v| !matches!(v, JsValue::Undefined))
    }
}

impl Trace for JsProxy {
    fn trace(&self, tracer: &mut Tracer) {
        self.target.trace(tracer);
        tracer.mark(self.handler);
    }
}

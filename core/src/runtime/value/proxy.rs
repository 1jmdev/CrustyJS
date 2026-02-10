use std::cell::RefCell;
use std::rc::Rc;

use super::object::JsObject;
use super::JsValue;

/// A JavaScript Proxy exotic object wrapping a target and handler.
#[derive(Debug, Clone)]
pub struct JsProxy {
    pub target: JsValue,
    pub handler: Rc<RefCell<JsObject>>,
    pub revoked: bool,
}

impl JsProxy {
    pub fn new(target: JsValue, handler: Rc<RefCell<JsObject>>) -> Self {
        Self {
            target,
            handler,
            revoked: false,
        }
    }

    pub fn wrapped(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    /// Check if this proxy has been revoked.
    pub fn check_revoked(&self) -> Result<(), String> {
        if self.revoked {
            Err("Cannot perform operation on a revoked proxy".to_string())
        } else {
            Ok(())
        }
    }

    /// Get a trap function from the handler by name.
    pub fn get_trap(&self, trap_name: &str) -> Option<JsValue> {
        let handler = self.handler.borrow();
        handler
            .get(trap_name)
            .filter(|v| !matches!(v, JsValue::Undefined))
    }
}

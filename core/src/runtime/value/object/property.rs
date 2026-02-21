use crate::runtime::gc::{Trace, Tracer};
use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct Property {
    pub value: JsValue,
    pub getter: Option<JsValue>,
    pub setter: Option<JsValue>,
    pub writable: bool,
    pub enumerable: bool,
    pub configurable: bool,
}

impl Property {
    pub fn new(value: JsValue) -> Self {
        Self {
            value,
            getter: None,
            setter: None,
            writable: true,
            enumerable: true,
            configurable: true,
        }
    }

    pub fn with_getter(getter: JsValue) -> Self {
        Self {
            value: JsValue::Undefined,
            getter: Some(getter),
            setter: None,
            writable: false,
            enumerable: true,
            configurable: true,
        }
    }

    pub fn with_setter(setter: JsValue) -> Self {
        Self {
            value: JsValue::Undefined,
            getter: None,
            setter: Some(setter),
            writable: false,
            enumerable: true,
            configurable: true,
        }
    }
}

impl Trace for Property {
    fn trace(&self, tracer: &mut Tracer) {
        self.value.trace(tracer);
        self.getter.trace(tracer);
        self.setter.trace(tracer);
    }
}

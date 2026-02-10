use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub value: Option<JsValue>,
    pub writable: bool,
    pub enumerable: bool,
    pub configurable: bool,
    pub get: Option<JsValue>,
    pub set: Option<JsValue>,
}

impl PropertyDescriptor {
    pub fn data(value: JsValue) -> Self {
        Self {
            value: Some(value),
            writable: true,
            enumerable: true,
            configurable: true,
            get: None,
            set: None,
        }
    }

    pub fn accessor(get: Option<JsValue>, set: Option<JsValue>) -> Self {
        Self {
            value: None,
            writable: false,
            enumerable: true,
            configurable: true,
            get,
            set,
        }
    }
}

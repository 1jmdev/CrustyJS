use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct Property {
    pub value: JsValue,
}

impl Property {
    pub fn new(value: JsValue) -> Self {
        Self { value }
    }
}

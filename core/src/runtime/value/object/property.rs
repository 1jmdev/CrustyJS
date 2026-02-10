use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct Property {
    pub value: JsValue,
    pub getter: Option<JsValue>,
    pub setter: Option<JsValue>,
}

impl Property {
    pub fn new(value: JsValue) -> Self {
        Self {
            value,
            getter: None,
            setter: None,
        }
    }

    pub fn with_getter(getter: JsValue) -> Self {
        Self {
            value: JsValue::Undefined,
            getter: Some(getter),
            setter: None,
        }
    }

    pub fn with_setter(setter: JsValue) -> Self {
        Self {
            value: JsValue::Undefined,
            getter: None,
            setter: Some(setter),
        }
    }
}

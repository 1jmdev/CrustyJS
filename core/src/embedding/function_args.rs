use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub struct FunctionArgs {
    this_value: JsValue,
    values: Vec<JsValue>,
}

impl FunctionArgs {
    pub fn new(this_value: JsValue, values: Vec<JsValue>) -> Self {
        Self { this_value, values }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&JsValue> {
        self.values.get(index)
    }

    pub fn this(&self) -> &JsValue {
        &self.this_value
    }
}

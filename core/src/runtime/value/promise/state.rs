use crate::runtime::value::JsValue;

#[derive(Debug, Clone)]
pub enum PromiseState {
    Pending,
    Fulfilled(JsValue),
    Rejected(JsValue),
}

use super::JsValue;
use std::fmt;

impl fmt::Display for JsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsValue::Undefined => write!(f, "undefined"),
            JsValue::Null => write!(f, "null"),
            JsValue::Boolean(b) => write!(f, "{b}"),
            JsValue::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            JsValue::String(s) => write!(f, "{s}"),
            JsValue::Function { name, .. } => {
                write!(f, "function {name}() {{ [native code] }}")
            }
            JsValue::NativeFunction { name, .. } => {
                write!(f, "function {name}() {{ [native code] }}")
            }
            JsValue::Object(obj) => {
                let obj = obj.borrow();
                let mut pairs: Vec<String> = obj
                    .properties
                    .iter()
                    .map(|(k, p)| format!("{k}: {}", p.value))
                    .collect();
                pairs.sort();
                write!(f, "{{ {} }}", pairs.join(", "))
            }
            JsValue::Array(arr) => {
                let arr = arr.borrow();
                let items: Vec<String> = arr.elements.iter().map(|v| v.to_js_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            JsValue::Promise(promise) => {
                use crate::runtime::value::promise::PromiseState;
                match &promise.borrow().state {
                    PromiseState::Pending => write!(f, "Promise {{ <pending> }}"),
                    PromiseState::Fulfilled(value) => {
                        write!(f, "Promise {{ <fulfilled>: {} }}", value)
                    }
                    PromiseState::Rejected(value) => {
                        write!(f, "Promise {{ <rejected>: {} }}", value)
                    }
                }
            }
        }
    }
}

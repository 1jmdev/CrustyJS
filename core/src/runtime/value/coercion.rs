use super::JsValue;

impl JsValue {
    /// Convert to a number (basic JS coercion rules).
    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::Number(n) => *n,
            JsValue::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            JsValue::Function { .. } => f64::NAN,
            JsValue::Object(_) => f64::NAN,
            JsValue::Array(_) => f64::NAN,
        }
    }

    /// Convert to a boolean (JS truthiness rules).
    pub fn to_boolean(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Boolean(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::Function { .. } => true,
            JsValue::Object(_) => true,
            JsValue::Array(_) => true,
        }
    }

    /// Convert to a string for concatenation (JS coercion rules).
    pub fn to_js_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.is_finite() && n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Function { name, .. } => {
                format!("function {name}() {{ [native code] }}")
            }
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(arr) => {
                let arr = arr.borrow();
                let items: Vec<String> = arr.elements.iter().map(|v| v.to_js_string()).collect();
                items.join(",")
            }
        }
    }
}

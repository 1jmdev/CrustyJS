use super::JsValue;

impl JsValue {
    /// Try to extract [[PrimitiveValue]] from a wrapper object (new Number, new Boolean, new String).
    /// Returns None if not a wrapper object.
    pub fn get_primitive_value(&self) -> Option<JsValue> {
        if let JsValue::Object(obj) = self {
            let borrowed = obj.borrow();
            if let Some(prop) = borrowed.properties.get("[[PrimitiveValue]]") {
                return Some(prop.value.clone());
            }
        }
        None
    }

    /// Convert to a number (basic JS coercion rules).
    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::Number(n) => *n,
            JsValue::String(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    0.0
                } else if let Some(hex) = trimmed.strip_prefix("0x").or(trimmed.strip_prefix("0X"))
                {
                    u64::from_str_radix(hex, 16)
                        .map(|v| v as f64)
                        .unwrap_or(f64::NAN)
                } else if let Some(oct) = trimmed.strip_prefix("0o").or(trimmed.strip_prefix("0O"))
                {
                    u64::from_str_radix(oct, 8)
                        .map(|v| v as f64)
                        .unwrap_or(f64::NAN)
                } else if let Some(bin) = trimmed.strip_prefix("0b").or(trimmed.strip_prefix("0B"))
                {
                    u64::from_str_radix(bin, 2)
                        .map(|v| v as f64)
                        .unwrap_or(f64::NAN)
                } else {
                    trimmed.parse::<f64>().unwrap_or(f64::NAN)
                }
            }
            JsValue::Function { .. } => f64::NAN,
            JsValue::NativeFunction { .. } => f64::NAN,
            JsValue::Symbol(_) => f64::NAN,
            JsValue::Object(_) => {
                // Check for [[PrimitiveValue]] on wrapper objects
                if let Some(prim) = self.get_primitive_value() {
                    return prim.to_number();
                }
                f64::NAN
            }
            JsValue::Array(arr) => {
                let borrowed = arr.borrow();
                if borrowed.elements.is_empty() {
                    return 0.0;
                }
                if borrowed.elements.len() == 1 {
                    return borrowed.elements[0].to_number();
                }
                f64::NAN
            }
            JsValue::Promise(_) => f64::NAN,
            JsValue::Map(_) => f64::NAN,
            JsValue::Set(_) => f64::NAN,
            JsValue::WeakMap(_) => f64::NAN,
            JsValue::WeakSet(_) => f64::NAN,
            JsValue::RegExp(_) => f64::NAN,
            JsValue::Proxy(_) => f64::NAN,
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
            JsValue::NativeFunction { .. } => true,
            JsValue::Symbol(_) => true,
            JsValue::Object(_) => true,
            JsValue::Array(_) => true,
            JsValue::Promise(_) => true,
            JsValue::Map(_) => true,
            JsValue::Set(_) => true,
            JsValue::WeakMap(_) => true,
            JsValue::WeakSet(_) => true,
            JsValue::RegExp(_) => true,
            JsValue::Proxy(_) => true,
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
            JsValue::NativeFunction { name, .. } => {
                format!("function {name}() {{ [native code] }}")
            }
            JsValue::Symbol(sym) => sym.to_string(),
            JsValue::Object(_) => {
                // Check for [[PrimitiveValue]] on wrapper objects
                if let Some(prim) = self.get_primitive_value() {
                    return prim.to_js_string();
                }
                "[object Object]".to_string()
            }
            JsValue::Array(arr) => {
                let arr = arr.borrow();
                let items: Vec<String> = arr.elements.iter().map(|v| v.to_js_string()).collect();
                items.join(",")
            }
            JsValue::Promise(_) => "[object Promise]".to_string(),
            JsValue::Map(_) => "[object Map]".to_string(),
            JsValue::Set(_) => "[object Set]".to_string(),
            JsValue::WeakMap(_) => "[object WeakMap]".to_string(),
            JsValue::WeakSet(_) => "[object WeakSet]".to_string(),
            JsValue::RegExp(re) => re.borrow().to_string(),
            JsValue::Proxy(_) => "[object Object]".to_string(),
        }
    }
}

pub fn abstract_equals(a: &JsValue, b: &JsValue) -> bool {
    use JsValue::*;
    match (a, b) {
        (Undefined, Undefined) | (Null, Null) => true,
        (Undefined, Null) | (Null, Undefined) => true,
        (Boolean(_), _) => abstract_equals(&Number(a.to_number()), b),
        (_, Boolean(_)) => abstract_equals(a, &Number(b.to_number())),
        (Number(_), String(_)) => abstract_equals(a, &Number(b.to_number())),
        (String(_), Number(_)) => abstract_equals(&Number(a.to_number()), b),
        _ => a == b,
    }
}

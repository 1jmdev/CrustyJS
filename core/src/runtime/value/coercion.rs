use super::JsValue;
use crate::errors::RuntimeError;
use crate::parser::ast::{Literal, UnaryOp};

impl JsValue {
    pub fn get_primitive_value(&self) -> Option<JsValue> {
        if let JsValue::Object(obj) = self {
            let b = obj.borrow();
            if let Some(prop) = b.properties.get("[[PrimitiveValue]]") {
                return Some(prop.value.clone());
            }
        }
        None
    }

    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::Number(n) => *n,
            JsValue::String(s) => {
                let t = s.trim();
                if t.is_empty() {
                    0.0
                } else if let Some(h) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                    u64::from_str_radix(h, 16)
                        .map(|v| v as f64)
                        .unwrap_or(f64::NAN)
                } else if let Some(o) = t.strip_prefix("0o").or_else(|| t.strip_prefix("0O")) {
                    u64::from_str_radix(o, 8)
                        .map(|v| v as f64)
                        .unwrap_or(f64::NAN)
                } else if let Some(b) = t.strip_prefix("0b").or_else(|| t.strip_prefix("0B")) {
                    u64::from_str_radix(b, 2)
                        .map(|v| v as f64)
                        .unwrap_or(f64::NAN)
                } else {
                    t.parse::<f64>().unwrap_or(f64::NAN)
                }
            }
            JsValue::Object(_) => self
                .get_primitive_value()
                .map(|p| p.to_number())
                .unwrap_or(f64::NAN),
            JsValue::Array(arr) => {
                let b = arr.borrow();
                match b.elements.len() {
                    0 => 0.0,
                    1 => b.elements[0].to_number(),
                    _ => f64::NAN,
                }
            }
            _ => f64::NAN,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Boolean(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    pub fn to_js_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".into(),
            JsValue::Null => "null".into(),
            JsValue::Boolean(b) => b.to_string(),
            JsValue::Number(n) => {
                if n.is_finite() && n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            JsValue::String(s) => s.clone(),
            JsValue::Function { name, .. } | JsValue::NativeFunction { name, .. } => {
                format!("function {name}() {{ [native code] }}")
            }
            JsValue::Symbol(sym) => sym.to_string(),
            JsValue::Object(_) => self
                .get_primitive_value()
                .map(|p| p.to_js_string())
                .unwrap_or_else(|| "[object Object]".into()),
            JsValue::Array(arr) => arr
                .borrow()
                .elements
                .iter()
                .map(|v| v.to_js_string())
                .collect::<Vec<_>>()
                .join(","),
            JsValue::Promise(_) => "[object Promise]".into(),
            JsValue::Map(_) => "[object Map]".into(),
            JsValue::Set(_) => "[object Set]".into(),
            JsValue::WeakMap(_) => "[object WeakMap]".into(),
            JsValue::WeakSet(_) => "[object WeakSet]".into(),
            JsValue::RegExp(re) => re.borrow().to_string(),
            JsValue::Proxy(_) => "[object Object]".into(),
        }
    }
}

pub fn abstract_equals(a: &JsValue, b: &JsValue) -> bool {
    use JsValue::*;
    match (a, b) {
        (Undefined, Undefined) | (Null, Null) | (Undefined, Null) | (Null, Undefined) => true,
        (Boolean(_), _) => abstract_equals(&Number(a.to_number()), b),
        (_, Boolean(_)) => abstract_equals(a, &Number(b.to_number())),
        (Number(_), String(_)) => abstract_equals(a, &Number(b.to_number())),
        (String(_), Number(_)) => abstract_equals(&Number(a.to_number()), b),
        _ => a == b,
    }
}

pub fn eval_literal(lit: &Literal) -> JsValue {
    match lit {
        Literal::Number(n) => JsValue::Number(*n),
        Literal::String(s) => JsValue::String(s.clone()),
        Literal::Boolean(b) => JsValue::Boolean(*b),
        Literal::Null => JsValue::Null,
        Literal::Undefined => JsValue::Undefined,
    }
}

pub fn eval_unary(op: &UnaryOp, val: JsValue) -> Result<JsValue, RuntimeError> {
    match op {
        UnaryOp::Neg => Ok(JsValue::Number(-val.to_number())),
        UnaryOp::Not => Ok(JsValue::Boolean(!val.to_boolean())),
        UnaryOp::Void => Ok(JsValue::Undefined),
        UnaryOp::Pos => Ok(JsValue::Number(val.to_number())),
    }
}

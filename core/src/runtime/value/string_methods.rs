use crate::errors::RuntimeError;
use crate::runtime::value::JsValue;

/// Resolve a property access on a string value (e.g. `s.length`).
pub fn resolve_string_property(s: &str, property: &str) -> Result<JsValue, RuntimeError> {
    match property {
        "length" => Ok(JsValue::Number(s.len() as f64)),
        _ => Err(RuntimeError::TypeError {
            message: format!("cannot access property '{property}' on string"),
        }),
    }
}

/// Call a method on a string value (e.g. `s.toUpperCase()`).
pub fn call_string_method(
    s: &str,
    method: &str,
    args: &[JsValue],
) -> Result<JsValue, RuntimeError> {
    match method {
        "toUpperCase" => Ok(JsValue::String(s.to_uppercase())),
        "toLowerCase" => Ok(JsValue::String(s.to_lowercase())),
        "trim" => Ok(JsValue::String(s.trim().to_string())),
        "includes" => {
            let substr = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            Ok(JsValue::Boolean(s.contains(&substr)))
        }
        "indexOf" => {
            let substr = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            let idx = s.find(&substr).map(|i| i as f64).unwrap_or(-1.0);
            Ok(JsValue::Number(idx))
        }
        "slice" => {
            let len = s.len() as i64;
            let start = normalize_index(args.first(), len);
            let end = args.get(1).map_or(len, |a| normalize_index(Some(a), len));
            if start >= end || start >= len {
                return Ok(JsValue::String(String::new()));
            }
            let result: String = s
                .chars()
                .skip(start as usize)
                .take((end - start) as usize)
                .collect();
            Ok(JsValue::String(result))
        }
        "split" => {
            let sep = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            let parts: Vec<JsValue> = s
                .split(&sep)
                .map(|part| JsValue::String(part.to_string()))
                .collect();
            // Return a string representation for now; proper arrays come in Phase 8
            let display: Vec<String> = parts.iter().map(|v| format!("{v}")).collect();
            Ok(JsValue::String(display.join(",")))
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("'{method}' is not a function"),
        }),
    }
}

/// Normalize a slice index: negative values count from end.
fn normalize_index(arg: Option<&JsValue>, len: i64) -> i64 {
    let n = arg.map(|a| a.to_number() as i64).unwrap_or(0);
    if n < 0 {
        (len + n).max(0)
    } else {
        n.min(len)
    }
}

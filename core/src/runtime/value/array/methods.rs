use crate::errors::RuntimeError;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::JsValue;
use std::cell::RefCell;
use std::rc::Rc;

pub fn call_array_method(
    arr: &Rc<RefCell<JsArray>>,
    method: &str,
    args: &[JsValue],
) -> Result<Option<JsValue>, RuntimeError> {
    match method {
        "push" => {
            let mut borrowed = arr.borrow_mut();
            for arg in args {
                borrowed.elements.push(arg.clone());
            }
            Ok(Some(JsValue::Number(borrowed.len() as f64)))
        }
        "pop" => {
            let mut borrowed = arr.borrow_mut();
            Ok(Some(borrowed.elements.pop().unwrap_or(JsValue::Undefined)))
        }
        "includes" => {
            let target = args.first().unwrap_or(&JsValue::Undefined);
            let borrowed = arr.borrow();
            let found = borrowed.elements.iter().any(|v| v == target);
            Ok(Some(JsValue::Boolean(found)))
        }
        "indexOf" => {
            let target = args.first().unwrap_or(&JsValue::Undefined);
            let borrowed = arr.borrow();
            let idx = borrowed.elements.iter().position(|v| v == target);
            Ok(Some(JsValue::Number(idx.map_or(-1.0, |i| i as f64))))
        }
        "join" => {
            let sep = match args.first() {
                Some(JsValue::String(s)) => s.clone(),
                _ => ",".to_string(),
            };
            let borrowed = arr.borrow();
            let items: Vec<String> = borrowed.elements.iter().map(|v| v.to_js_string()).collect();
            Ok(Some(JsValue::String(items.join(&sep))))
        }
        "slice" => {
            let borrowed = arr.borrow();
            let len = borrowed.len() as i64;
            let start = normalize_index(args.first(), 0, len);
            let end = normalize_index(args.get(1), len, len);
            let sliced: Vec<JsValue> = borrowed.elements[start..end].to_vec();
            Ok(Some(JsValue::Array(JsArray::new(sliced).wrapped())))
        }
        "concat" => {
            let borrowed = arr.borrow();
            let mut result = borrowed.elements.clone();
            for arg in args {
                if let JsValue::Array(other) = arg {
                    result.extend(other.borrow().elements.clone());
                } else {
                    result.push(arg.clone());
                }
            }
            Ok(Some(JsValue::Array(JsArray::new(result).wrapped())))
        }
        _ => Ok(None),
    }
}

fn normalize_index(arg: Option<&JsValue>, default: i64, len: i64) -> usize {
    let val = match arg {
        Some(v) => v.to_number() as i64,
        None => default,
    };
    let idx = if val < 0 {
        (len + val).max(0)
    } else {
        val.min(len)
    };
    idx as usize
}

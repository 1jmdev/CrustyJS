use crate::errors::RuntimeError;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::JsValue;

pub fn resolve_string_property(s: &str, property: &str) -> Result<JsValue, RuntimeError> {
    match property {
        "length" => Ok(JsValue::Number(s.len() as f64)),
        _ => Err(RuntimeError::TypeError {
            message: format!("cannot access property '{property}' on string"),
        }),
    }
}

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
            if let Some(JsValue::RegExp(re)) = args.first() {
                return split_with_regex(s, re);
            }
            let sep = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            let parts: Vec<JsValue> = s
                .split(&sep)
                .map(|part| JsValue::String(part.to_string()))
                .collect();
            Ok(JsValue::Array(JsArray::new(parts).wrapped()))
        }
        "match" => {
            if let Some(JsValue::RegExp(re)) = args.first() {
                return match_with_regex(s, re);
            }
            let pattern = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            match s.find(&pattern) {
                Some(_) => {
                    let arr = JsArray::new(vec![JsValue::String(pattern)]).wrapped();
                    Ok(JsValue::Array(arr))
                }
                None => Ok(JsValue::Null),
            }
        }
        "replace" => {
            if let Some(JsValue::RegExp(re)) = args.first() {
                let replacement = args.get(1).map(|a| a.to_js_string()).unwrap_or_default();
                return replace_with_regex(s, re, &replacement, false);
            }
            let pattern = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            let replacement = args.get(1).map(|a| a.to_js_string()).unwrap_or_default();
            Ok(JsValue::String(s.replacen(&pattern, &replacement, 1)))
        }
        "replaceAll" => {
            if let Some(JsValue::RegExp(re)) = args.first() {
                let replacement = args.get(1).map(|a| a.to_js_string()).unwrap_or_default();
                return replace_with_regex(s, re, &replacement, true);
            }
            let pattern = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            let replacement = args.get(1).map(|a| a.to_js_string()).unwrap_or_default();
            Ok(JsValue::String(s.replace(&pattern, &replacement)))
        }
        "search" => {
            if let Some(JsValue::RegExp(re)) = args.first() {
                return search_with_regex(s, re);
            }
            let pattern = args.first().map(|a| a.to_js_string()).unwrap_or_default();
            let idx = s.find(&pattern).map(|i| i as f64).unwrap_or(-1.0);
            Ok(JsValue::Number(idx))
        }
        _ => Err(RuntimeError::TypeError {
            message: format!("'{method}' is not a function"),
        }),
    }
}

fn normalize_index(arg: Option<&JsValue>, len: i64) -> i64 {
    let n = arg.map(|a| a.to_number() as i64).unwrap_or(0);
    if n < 0 {
        (len + n).max(0)
    } else {
        n.min(len)
    }
}

fn match_with_regex(
    s: &str,
    re: &std::rc::Rc<std::cell::RefCell<super::regexp::JsRegExp>>,
) -> Result<JsValue, RuntimeError> {
    let mut re = re.borrow_mut();
    if re.flags.global {
        let matches = re.match_all(s);
        if matches.is_empty() {
            return Ok(JsValue::Null);
        }
        let vals: Vec<JsValue> = matches.into_iter().map(JsValue::String).collect();
        Ok(JsValue::Array(JsArray::new(vals).wrapped()))
    } else {
        match re.exec(s) {
            Some(m) => {
                let vals: Vec<JsValue> = m
                    .captures
                    .iter()
                    .map(|c| match c {
                        Some(s) => JsValue::String(s.clone()),
                        None => JsValue::Undefined,
                    })
                    .collect();
                Ok(JsValue::Array(JsArray::new(vals).wrapped()))
            }
            None => Ok(JsValue::Null),
        }
    }
}

fn replace_with_regex(
    s: &str,
    re: &std::rc::Rc<std::cell::RefCell<super::regexp::JsRegExp>>,
    replacement: &str,
    replace_all: bool,
) -> Result<JsValue, RuntimeError> {
    let re = re.borrow();
    let compiled = re.compiled();
    if re.flags.global || replace_all {
        Ok(JsValue::String(
            compiled.replace_all(s, replacement).into_owned(),
        ))
    } else {
        Ok(JsValue::String(
            compiled.replace(s, replacement).into_owned(),
        ))
    }
}

fn search_with_regex(
    s: &str,
    re: &std::rc::Rc<std::cell::RefCell<super::regexp::JsRegExp>>,
) -> Result<JsValue, RuntimeError> {
    let re = re.borrow();
    match re.compiled().find(s) {
        Some(m) => Ok(JsValue::Number(m.start() as f64)),
        None => Ok(JsValue::Number(-1.0)),
    }
}

fn split_with_regex(
    s: &str,
    re: &std::rc::Rc<std::cell::RefCell<super::regexp::JsRegExp>>,
) -> Result<JsValue, RuntimeError> {
    let re = re.borrow();
    let parts: Vec<JsValue> = re
        .compiled()
        .split(s)
        .map(|part| JsValue::String(part.to_string()))
        .collect();
    Ok(JsValue::Array(JsArray::new(parts).wrapped()))
}

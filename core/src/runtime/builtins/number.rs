use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::JsValue;

pub(crate) fn parse_int(s: &str, radix: i32) -> f64 {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return f64::NAN;
    }
    let (negative, rest) = if let Some(s) = trimmed.strip_prefix('-') {
        (true, s)
    } else if let Some(s) = trimmed.strip_prefix('+') {
        (false, s)
    } else {
        (false, trimmed)
    };
    let radix = if radix == 0 {
        if rest.starts_with("0x") || rest.starts_with("0X") {
            16
        } else {
            10
        }
    } else {
        radix
    };
    if !(2..=36).contains(&radix) {
        return f64::NAN;
    }
    let digits = if radix == 16 {
        rest.strip_prefix("0x")
            .or_else(|| rest.strip_prefix("0X"))
            .unwrap_or(rest)
    } else {
        rest
    };
    let mut result = 0.0f64;
    let mut found = false;
    for ch in digits.chars() {
        match ch.to_digit(radix as u32) {
            Some(d) => {
                found = true;
                result = result * radix as f64 + d as f64;
            }
            None => break,
        }
    }
    if !found {
        return f64::NAN;
    }
    if negative {
        -result
    } else {
        result
    }
}

impl Interpreter {
    pub(crate) fn builtin_number_static(
        &self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let first = || args.first().cloned().unwrap_or(JsValue::Undefined);
        match method {
            "isNaN" => Ok(JsValue::Boolean(
                matches!(first(), JsValue::Number(n) if n.is_nan()),
            )),
            "isFinite" => Ok(JsValue::Boolean(
                matches!(first(), JsValue::Number(n) if n.is_finite()),
            )),
            "isInteger" => Ok(JsValue::Boolean(
                matches!(first(), JsValue::Number(n) if n.is_finite() && n == n.trunc()),
            )),
            "isSafeInteger" => Ok(JsValue::Boolean(
                matches!(first(), JsValue::Number(n) if n.is_finite() && n == n.trunc() && n.abs() <= 9007199254740991.0),
            )),
            "parseInt" => {
                let s = first().to_js_string();
                let radix = args.get(1).map(|v| v.to_number() as i32).unwrap_or(0);
                Ok(JsValue::Number(parse_int(&s, radix)))
            }
            "parseFloat" => {
                let s = first().to_js_string();
                Ok(JsValue::Number(s.trim().parse::<f64>().unwrap_or(f64::NAN)))
            }
            _ => self.builtin_number_property(method),
        }
    }

    pub(crate) fn builtin_number_property(&self, prop: &str) -> Result<JsValue, RuntimeError> {
        let v = match prop {
            "MAX_VALUE" => f64::MAX,
            "MIN_VALUE" => f64::MIN_POSITIVE,
            "MAX_SAFE_INTEGER" => 9007199254740991.0,
            "MIN_SAFE_INTEGER" => -9007199254740991.0,
            "POSITIVE_INFINITY" => f64::INFINITY,
            "NEGATIVE_INFINITY" => f64::NEG_INFINITY,
            "NaN" => f64::NAN,
            "EPSILON" => f64::EPSILON,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: format!("Number.{prop} is not defined"),
                })
            }
        };
        Ok(JsValue::Number(v))
    }
}

use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::JsValue;
use std::time::{SystemTime, UNIX_EPOCH};

impl Interpreter {
    pub(crate) fn builtin_math_constant(&self, property: &str) -> Result<JsValue, RuntimeError> {
        use std::f64::consts;
        let v = match property {
            "PI" => consts::PI,
            "E" => consts::E,
            "LN2" => consts::LN_2,
            "LN10" => consts::LN_10,
            "LOG2E" => consts::LOG2_E,
            "LOG10E" => consts::LOG10_E,
            "SQRT2" => consts::SQRT_2,
            "SQRT1_2" => consts::FRAC_1_SQRT_2,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: format!("Math.{property} is not defined"),
                })
            }
        };
        Ok(JsValue::Number(v))
    }

    pub(crate) fn builtin_math_call(
        &self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let n = |i: usize| {
            args.get(i)
                .cloned()
                .unwrap_or(JsValue::Undefined)
                .to_number()
        };

        let v = match method {
            "abs" => n(0).abs(),
            "floor" => n(0).floor(),
            "ceil" => n(0).ceil(),
            "round" => n(0).round(),
            "trunc" => n(0).trunc(),
            "sqrt" => n(0).sqrt(),
            "cbrt" => n(0).cbrt(),
            "exp" => n(0).exp(),
            "log" => n(0).ln(),
            "log2" => n(0).log2(),
            "log10" => n(0).log10(),
            "sin" => n(0).sin(),
            "cos" => n(0).cos(),
            "tan" => n(0).tan(),
            "asin" => n(0).asin(),
            "acos" => n(0).acos(),
            "atan" => n(0).atan(),
            "atan2" => n(0).atan2(n(1)),
            "pow" => n(0).powf(n(1)),
            "fround" => (n(0) as f32) as f64,
            "clz32" => (n(0) as u32).leading_zeros() as f64,
            "imul" => ((n(0) as i32).wrapping_mul(n(1) as i32)) as f64,
            "sign" => {
                let v = n(0);
                if v.is_nan() {
                    f64::NAN
                } else if v == 0.0 {
                    v
                } else {
                    v.signum()
                }
            }
            "max" => {
                if args.is_empty() {
                    f64::NEG_INFINITY
                } else {
                    args.iter()
                        .map(|v| v.to_number())
                        .fold(f64::NEG_INFINITY, f64::max)
                }
            }
            "min" => {
                if args.is_empty() {
                    f64::INFINITY
                } else {
                    args.iter()
                        .map(|v| v.to_number())
                        .fold(f64::INFINITY, f64::min)
                }
            }
            "hypot" => {
                if args.is_empty() {
                    0.0
                } else {
                    args.iter()
                        .map(|v| v.to_number())
                        .fold(0.0f64, |a, x| a.hypot(x))
                }
            }
            "random" => {
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0);
                (nanos as f64) / (u32::MAX as f64)
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    message: format!("Math.{method} is not a function"),
                })
            }
        };
        Ok(JsValue::Number(v))
    }
}

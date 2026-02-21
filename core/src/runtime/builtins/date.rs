use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::JsValue;
use std::time::{SystemTime, UNIX_EPOCH};

impl Interpreter {
    pub(crate) fn builtin_date_static(&self, method: &str) -> Result<JsValue, RuntimeError> {
        match method {
            "now" => {
                let ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0);
                Ok(JsValue::Number(ms))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Date.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn builtin_performance_now(&self) -> JsValue {
        JsValue::Number(self.start_time.elapsed().as_secs_f64() * 1000.0)
    }
}

use crate::errors::RuntimeError;
use crate::runtime::value::JsValue;

const MAX_STACK: usize = 256;

pub struct Stack {
    values: Vec<JsValue>,
}

impl Stack {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, value: JsValue) -> Result<(), RuntimeError> {
        if self.values.len() >= MAX_STACK {
            return Err(RuntimeError::TypeError {
                message: "VM stack overflow".to_string(),
            });
        }
        self.values.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<JsValue, RuntimeError> {
        self.values.pop().ok_or_else(|| RuntimeError::TypeError {
            message: "VM stack underflow".to_string(),
        })
    }

    pub fn peek(&self) -> Result<&JsValue, RuntimeError> {
        self.values.last().ok_or_else(|| RuntimeError::TypeError {
            message: "VM stack is empty".to_string(),
        })
    }

    #[allow(dead_code)]
    pub fn peek_at(&self, offset: usize) -> Result<&JsValue, RuntimeError> {
        if offset >= self.values.len() {
            return Err(RuntimeError::TypeError {
                message: "VM stack peek out of bounds".to_string(),
            });
        }
        let idx = self.values.len() - 1 - offset;
        Ok(&self.values[idx])
    }
}

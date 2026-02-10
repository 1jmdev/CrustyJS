use crate::errors::RuntimeError;
use crate::vm::bytecode::VmValue;

const MAX_STACK: usize = 256;

pub struct Stack {
    values: Vec<VmValue>,
}

impl Stack {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn push(&mut self, value: VmValue) -> Result<(), RuntimeError> {
        if self.values.len() >= MAX_STACK {
            return Err(RuntimeError::TypeError {
                message: "VM stack overflow".to_string(),
            });
        }
        self.values.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<VmValue, RuntimeError> {
        self.values.pop().ok_or_else(|| RuntimeError::TypeError {
            message: "VM stack underflow".to_string(),
        })
    }

    #[allow(dead_code)]
    pub fn peek_at(&self, offset: usize) -> Result<&VmValue, RuntimeError> {
        if offset >= self.values.len() {
            return Err(RuntimeError::TypeError {
                message: "VM stack peek out of bounds".to_string(),
            });
        }
        let idx = self.values.len() - 1 - offset;
        Ok(&self.values[idx])
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn truncate(&mut self, len: usize) {
        self.values.truncate(len);
    }

    pub fn get(&self, index: usize) -> Result<VmValue, RuntimeError> {
        self.values
            .get(index)
            .cloned()
            .ok_or_else(|| RuntimeError::TypeError {
                message: "VM stack index out of bounds".to_string(),
            })
    }

    pub fn set(&mut self, index: usize, value: VmValue) -> Result<(), RuntimeError> {
        if let Some(slot) = self.values.get_mut(index) {
            *slot = value;
            Ok(())
        } else {
            Err(RuntimeError::TypeError {
                message: "VM stack set index out of bounds".to_string(),
            })
        }
    }
}

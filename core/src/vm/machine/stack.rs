use crate::errors::RuntimeError;
use crate::vm::bytecode::nan_boxing::{HeapStore, NanBoxedValue};
use crate::vm::bytecode::VmValue;

const MAX_STACK: usize = 256;

pub struct Stack {
    values: Vec<NanBoxedValue>,
    pub heap: HeapStore,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            heap: HeapStore::new(),
        }
    }

    pub fn push_vm(&mut self, value: VmValue) -> Result<(), RuntimeError> {
        if self.values.len() >= MAX_STACK {
            return Err(RuntimeError::TypeError {
                message: "VM stack overflow".to_string(),
            });
        }
        let boxed = NanBoxedValue::encode(&value, &mut self.heap);
        self.values.push(boxed);
        Ok(())
    }

    pub fn push_boxed(&mut self, value: NanBoxedValue) -> Result<(), RuntimeError> {
        if self.values.len() >= MAX_STACK {
            return Err(RuntimeError::TypeError {
                message: "VM stack overflow".to_string(),
            });
        }
        self.values.push(value);
        Ok(())
    }

    pub fn pop_boxed(&mut self) -> Result<NanBoxedValue, RuntimeError> {
        self.values.pop().ok_or_else(|| RuntimeError::TypeError {
            message: "VM stack underflow".to_string(),
        })
    }

    pub fn pop_vm(&mut self) -> Result<VmValue, RuntimeError> {
        let boxed = self.pop_boxed()?;
        Ok(boxed.decode_to_vm(&self.heap))
    }

    #[allow(dead_code)]
    pub fn peek_at_boxed(&self, offset: usize) -> Result<NanBoxedValue, RuntimeError> {
        if offset >= self.values.len() {
            return Err(RuntimeError::TypeError {
                message: "VM stack peek out of bounds".to_string(),
            });
        }
        let idx = self.values.len() - 1 - offset;
        Ok(self.values[idx])
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn truncate(&mut self, len: usize) {
        self.values.truncate(len);
    }

    pub fn get_boxed(&self, index: usize) -> Result<NanBoxedValue, RuntimeError> {
        self.values
            .get(index)
            .copied()
            .ok_or_else(|| RuntimeError::TypeError {
                message: "VM stack index out of bounds".to_string(),
            })
    }

    pub fn get_vm(&self, index: usize) -> Result<VmValue, RuntimeError> {
        let boxed = self.get_boxed(index)?;
        Ok(boxed.decode_to_vm(&self.heap))
    }

    pub fn set_vm(&mut self, index: usize, value: VmValue) -> Result<(), RuntimeError> {
        if index >= self.values.len() {
            return Err(RuntimeError::TypeError {
                message: "VM stack set index out of bounds".to_string(),
            });
        }
        let boxed = NanBoxedValue::encode(&value, &mut self.heap);
        self.values[index] = boxed;
        Ok(())
    }

    pub fn set_boxed(&mut self, index: usize, value: NanBoxedValue) -> Result<(), RuntimeError> {
        if index >= self.values.len() {
            return Err(RuntimeError::TypeError {
                message: "VM stack set index out of bounds".to_string(),
            });
        }
        self.values[index] = value;
        Ok(())
    }
}

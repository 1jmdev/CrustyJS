mod call_frame;
mod stack;

use std::collections::HashMap;

use crate::errors::RuntimeError;
use crate::vm::bytecode::nan_boxing::{Decoded, NanBoxedValue};
use crate::vm::bytecode::{Chunk, Opcode, VmValue};

use call_frame::CallFrame;
use stack::Stack;

pub struct VM {
    stack: Stack,
    frames: Vec<CallFrame>,
    globals: HashMap<String, NanBoxedValue>,
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run(
        &mut self,
        chunk: Chunk,
        _source: Option<String>,
        _source_path: Option<std::path::PathBuf>,
    ) -> Result<(), RuntimeError> {
        self.frames.push(CallFrame::new(chunk));

        while !self.frames.is_empty() {
            let op = {
                let frame = self.frames.last_mut().expect("frame should exist");
                if frame.ip >= frame.chunk.instructions.len() {
                    self.handle_return(NanBoxedValue::undefined())?;
                    continue;
                }
                let op = frame.chunk.instructions[frame.ip].clone();
                frame.ip += 1;
                op
            };

            match op {
                Opcode::Constant(idx) => {
                    let val = self.current_chunk()?.constants[idx as usize].clone();
                    self.stack.push_vm(val)?;
                }
                Opcode::Add => self.exec_add()?,
                Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod => {
                    self.exec_numeric_binary(&op)?;
                }
                Opcode::Negate => {
                    let val = self.stack.pop_boxed()?;
                    let result = NanBoxedValue::from_f64(-val.to_f64());
                    self.stack.push_boxed(result)?;
                }
                Opcode::Not => {
                    let val = self.stack.pop_boxed()?;
                    self.stack
                        .push_boxed(NanBoxedValue::from_bool(!val.to_bool()))?;
                }
                Opcode::Equal | Opcode::StrictEqual => {
                    let rhs = self.stack.pop_vm()?;
                    let lhs = self.stack.pop_vm()?;
                    let equal = lhs.to_output() == rhs.to_output();
                    self.stack.push_boxed(NanBoxedValue::from_bool(equal))?;
                }
                Opcode::LessThan => {
                    let rhs = self.stack.pop_boxed()?;
                    let lhs = self.stack.pop_boxed()?;
                    let result = lhs.to_f64() < rhs.to_f64();
                    self.stack.push_boxed(NanBoxedValue::from_bool(result))?;
                }
                Opcode::GreaterThan => {
                    let rhs = self.stack.pop_boxed()?;
                    let lhs = self.stack.pop_boxed()?;
                    let result = lhs.to_f64() > rhs.to_f64();
                    self.stack.push_boxed(NanBoxedValue::from_bool(result))?;
                }
                Opcode::SetGlobal(name_idx) => {
                    let key = self.constant_name(name_idx)?;
                    let val = self.stack.pop_boxed()?;
                    self.globals.insert(key, val);
                }
                Opcode::GetGlobal(name_idx) => {
                    let key = self.constant_name(name_idx)?;
                    let val = self
                        .globals
                        .get(&key)
                        .copied()
                        .unwrap_or(NanBoxedValue::undefined());
                    self.stack.push_boxed(val)?;
                }
                Opcode::SetLocal(slot) => {
                    let val = self.stack.pop_boxed()?;
                    let base = self.current_slot();
                    self.stack.set_boxed(base + slot as usize, val)?;
                }
                Opcode::GetLocal(slot) => {
                    let base = self.current_slot();
                    let val = self.stack.get_boxed(base + slot as usize)?;
                    self.stack.push_boxed(val)?;
                }
                Opcode::Call(arg_count) => {
                    self.exec_call(arg_count)?;
                }
                Opcode::Return => {
                    let result = self.stack.pop_boxed().unwrap_or(NanBoxedValue::undefined());
                    self.handle_return(result)?;
                }
                Opcode::Pop => {
                    let _ = self.stack.pop_boxed()?;
                }
                Opcode::Print => {
                    let value = self.stack.pop_vm()?;
                    println!("{}", value.to_output());
                }
                Opcode::Nil => self.stack.push_boxed(NanBoxedValue::null())?,
                Opcode::True => self.stack.push_boxed(NanBoxedValue::from_bool(true))?,
                Opcode::False => self.stack.push_boxed(NanBoxedValue::from_bool(false))?,
                Opcode::JumpIfFalse(target) => {
                    let cond = self.stack.pop_boxed()?;
                    if !cond.to_bool() {
                        self.current_frame_mut()?.ip = target as usize;
                    }
                }
                Opcode::Jump(target) => {
                    self.current_frame_mut()?.ip = target as usize;
                }
                Opcode::Loop(target) => {
                    self.current_frame_mut()?.ip = target as usize;
                }
                Opcode::Nop => {}
                Opcode::GetPropertyIC(idx) => {
                    let prop_name = self.constant_name(idx)?;
                    let obj = self.stack.pop_vm()?;
                    let result = self.get_property_value(&obj, &prop_name);
                    self.stack.push_vm(result)?;
                }
                other => {
                    return Err(RuntimeError::TypeError {
                        message: format!("unsupported opcode in VM: {other:?}"),
                    });
                }
            }
        }
        Ok(())
    }

    fn exec_add(&mut self) -> Result<(), RuntimeError> {
        let rhs_b = self.stack.pop_boxed()?;
        let lhs_b = self.stack.pop_boxed()?;
        let lhs_is_str = matches!(lhs_b.decode(), Decoded::Pointer(_))
            && matches!(lhs_b.decode_to_vm(&self.stack.heap), VmValue::String(_));
        let rhs_is_str = matches!(rhs_b.decode(), Decoded::Pointer(_))
            && matches!(rhs_b.decode_to_vm(&self.stack.heap), VmValue::String(_));
        if lhs_is_str || rhs_is_str {
            let lhs = lhs_b.decode_to_vm(&self.stack.heap);
            let rhs = rhs_b.decode_to_vm(&self.stack.heap);
            let s = format!("{}{}", lhs.to_output(), rhs.to_output());
            self.stack.push_vm(VmValue::String(s))?;
        } else {
            let result = NanBoxedValue::from_f64(lhs_b.to_f64() + rhs_b.to_f64());
            self.stack.push_boxed(result)?;
        }
        Ok(())
    }

    fn exec_numeric_binary(&mut self, op: &Opcode) -> Result<(), RuntimeError> {
        let rhs = self.stack.pop_boxed()?.to_f64();
        let lhs = self.stack.pop_boxed()?.to_f64();
        let result = match op {
            Opcode::Sub => lhs - rhs,
            Opcode::Mul => lhs * rhs,
            Opcode::Div => lhs / rhs,
            Opcode::Mod => lhs % rhs,
            _ => unreachable!(),
        };
        self.stack.push_boxed(NanBoxedValue::from_f64(result))
    }

    fn exec_call(&mut self, arg_count: u8) -> Result<(), RuntimeError> {
        let mut args = Vec::new();
        for _ in 0..arg_count {
            args.push(self.stack.pop_boxed()?);
        }
        args.reverse();
        let callee = self.stack.pop_vm()?;
        match callee {
            VmValue::Function(func) => {
                if func.arity != arg_count as usize {
                    return Err(RuntimeError::ArityMismatch {
                        expected: func.arity,
                        got: arg_count as usize,
                    });
                }
                let slot = self.stack.len();
                for arg in args {
                    self.stack.push_boxed(arg)?;
                }
                self.frames.push(CallFrame {
                    chunk: (*func.chunk).clone(),
                    ip: 0,
                    slot,
                });
            }
            _ => {
                return Err(RuntimeError::NotAFunction {
                    name: callee.to_output(),
                });
            }
        }
        Ok(())
    }

    fn handle_return(&mut self, value: NanBoxedValue) -> Result<(), RuntimeError> {
        let frame = self.frames.pop().ok_or_else(|| RuntimeError::TypeError {
            message: "return with empty frame stack".to_string(),
        })?;
        self.stack.truncate(frame.slot);
        if self.frames.is_empty() {
            return Ok(());
        }
        self.stack.push_boxed(value)?;
        Ok(())
    }

    fn current_chunk(&self) -> Result<&Chunk, RuntimeError> {
        self.frames
            .last()
            .map(|f| &f.chunk)
            .ok_or_else(|| RuntimeError::TypeError {
                message: "VM has no active frame".to_string(),
            })
    }

    fn current_slot(&self) -> usize {
        self.frames.last().map(|f| f.slot).unwrap_or(0)
    }

    fn current_frame_mut(&mut self) -> Result<&mut CallFrame, RuntimeError> {
        self.frames
            .last_mut()
            .ok_or_else(|| RuntimeError::TypeError {
                message: "VM has no active frame".to_string(),
            })
    }

    fn constant_name(&self, idx: u16) -> Result<String, RuntimeError> {
        match self.current_chunk()?.constants.get(idx as usize) {
            Some(VmValue::String(name)) => Ok(name.clone()),
            _ => Err(RuntimeError::TypeError {
                message: "global name constant must be a string".to_string(),
            }),
        }
    }

    fn get_property_value(&self, obj: &VmValue, prop: &str) -> VmValue {
        match obj {
            VmValue::String(s) => match prop {
                "length" => VmValue::Number(s.len() as f64),
                _ => VmValue::Undefined,
            },
            _ => VmValue::Undefined,
        }
    }
}

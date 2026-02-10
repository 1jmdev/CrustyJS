mod call_frame;
mod stack;

use std::collections::HashMap;

use crate::errors::RuntimeError;
use crate::lexer;
use crate::parser;
use crate::runtime::interpreter::Interpreter;
use crate::vm::bytecode::{Chunk, Opcode, VmValue};

use call_frame::CallFrame;
use stack::Stack;

pub struct VM {
    stack: Stack,
    frames: Vec<CallFrame>,
    globals: HashMap<String, VmValue>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self, chunk: Chunk, source: Option<String>) -> Result<(), RuntimeError> {
        self.frames.push(CallFrame::new(chunk));

        while !self.frames.is_empty() {
            let op = {
                let frame = self.frames.last_mut().expect("frame should exist");
                if frame.ip >= frame.chunk.instructions.len() {
                    self.handle_return(VmValue::Undefined)?;
                    continue;
                }
                let op = frame.chunk.instructions[frame.ip].clone();
                frame.ip += 1;
                op
            };

            match op {
                Opcode::Constant(idx) => {
                    let val = self.current_chunk()?.constants[idx as usize].clone();
                    self.stack.push(val)?;
                }
                Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod => {
                    self.exec_arithmetic(&op)?;
                }
                Opcode::Negate => {
                    let val = self.stack.pop()?;
                    self.stack.push(VmValue::Number(-val.to_number()))?;
                }
                Opcode::Not => {
                    let val = self.stack.pop()?;
                    self.stack.push(VmValue::Boolean(!val.to_boolean()))?;
                }
                Opcode::Equal | Opcode::StrictEqual => {
                    let rhs = self.stack.pop()?;
                    let lhs = self.stack.pop()?;
                    let equal = lhs.to_output() == rhs.to_output();
                    self.stack.push(VmValue::Boolean(equal))?;
                }
                Opcode::LessThan => {
                    let rhs = self.stack.pop()?;
                    let lhs = self.stack.pop()?;
                    self.stack
                        .push(VmValue::Boolean(lhs.to_number() < rhs.to_number()))?;
                }
                Opcode::GreaterThan => {
                    let rhs = self.stack.pop()?;
                    let lhs = self.stack.pop()?;
                    self.stack
                        .push(VmValue::Boolean(lhs.to_number() > rhs.to_number()))?;
                }
                Opcode::SetGlobal(name_idx) => {
                    let key = self.constant_name(name_idx)?;
                    let val = self.stack.pop()?;
                    self.globals.insert(key, val);
                }
                Opcode::GetGlobal(name_idx) => {
                    let key = self.constant_name(name_idx)?;
                    let val = self
                        .globals
                        .get(&key)
                        .cloned()
                        .unwrap_or(VmValue::Undefined);
                    self.stack.push(val)?;
                }
                Opcode::SetLocal(slot) => {
                    let val = self.stack.pop()?;
                    let base = self.current_slot();
                    self.stack.set(base + slot as usize, val)?;
                }
                Opcode::GetLocal(slot) => {
                    let base = self.current_slot();
                    let val = self.stack.get(base + slot as usize)?;
                    self.stack.push(val)?;
                }
                Opcode::Call(arg_count) => {
                    let mut args = Vec::new();
                    for _ in 0..arg_count {
                        args.push(self.stack.pop()?);
                    }
                    args.reverse();
                    let callee = self.stack.pop()?;
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
                                self.stack.push(arg)?;
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
                }
                Opcode::Return => {
                    let result = self.stack.pop().unwrap_or(VmValue::Undefined);
                    self.handle_return(result)?;
                }
                Opcode::Pop => {
                    let _ = self.stack.pop()?;
                }
                Opcode::Print => {
                    let value = self.stack.pop()?;
                    println!("{}", value.to_output());
                }
                Opcode::Nil => self.stack.push(VmValue::Null)?,
                Opcode::True => self.stack.push(VmValue::Boolean(true))?,
                Opcode::False => self.stack.push(VmValue::Boolean(false))?,
                Opcode::JumpIfFalse(target) => {
                    let cond = self.stack.pop()?;
                    if !cond.to_boolean() {
                        self.current_frame_mut()?.ip = target as usize;
                    }
                }
                Opcode::Jump(target) => {
                    self.current_frame_mut()?.ip = target as usize;
                }
                Opcode::Loop(target) => {
                    self.current_frame_mut()?.ip = target as usize;
                }
                Opcode::RunTreeWalk => {
                    if let Some(src) = source.as_ref() {
                        let tokens = lexer::lex(src).map_err(|e| RuntimeError::TypeError {
                            message: format!("VM bridge lex error: {e}"),
                        })?;
                        let program =
                            parser::parse(tokens).map_err(|e| RuntimeError::TypeError {
                                message: format!("VM bridge parse error: {e}"),
                            })?;
                        let mut interp = Interpreter::new_with_realtime_timers(true);
                        interp.run(&program)?;
                        return Ok(());
                    }
                    return Err(RuntimeError::TypeError {
                        message: "RunTreeWalk opcode requires source text".to_string(),
                    });
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

    fn handle_return(&mut self, value: VmValue) -> Result<(), RuntimeError> {
        let frame = self.frames.pop().ok_or_else(|| RuntimeError::TypeError {
            message: "return with empty frame stack".to_string(),
        })?;
        self.stack.truncate(frame.slot);
        if self.frames.is_empty() {
            return Ok(());
        }
        self.stack.push(value)?;
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

    fn exec_arithmetic(&mut self, op: &Opcode) -> Result<(), RuntimeError> {
        let rhs = self.stack.pop()?;
        let lhs = self.stack.pop()?;
        let out = match op {
            Opcode::Add => {
                if matches!(lhs, VmValue::String(_)) || matches!(rhs, VmValue::String(_)) {
                    VmValue::String(format!("{}{}", lhs.to_output(), rhs.to_output()))
                } else {
                    VmValue::Number(lhs.to_number() + rhs.to_number())
                }
            }
            Opcode::Sub => VmValue::Number(lhs.to_number() - rhs.to_number()),
            Opcode::Mul => VmValue::Number(lhs.to_number() * rhs.to_number()),
            Opcode::Div => VmValue::Number(lhs.to_number() / rhs.to_number()),
            Opcode::Mod => VmValue::Number(lhs.to_number() % rhs.to_number()),
            _ => unreachable!(),
        };
        self.stack.push(out)
    }
}

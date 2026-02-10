mod call_frame;
mod stack;

use std::collections::HashMap;

use crate::errors::RuntimeError;
use crate::lexer;
use crate::parser;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::JsValue;
use crate::vm::bytecode::{Chunk, Opcode};

use call_frame::CallFrame;
use stack::Stack;

pub struct VM {
    stack: Stack,
    frames: Vec<CallFrame>,
    globals: HashMap<String, JsValue>,
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
        self.frames.push(CallFrame::new(chunk.clone()));

        let mut ip = 0usize;
        while ip < chunk.instructions.len() {
            match &chunk.instructions[ip] {
                Opcode::Constant(idx) => {
                    self.stack.push(chunk.constants[*idx as usize].clone())?;
                }
                Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Mod => {
                    self.exec_arithmetic(&chunk.instructions[ip])?;
                }
                Opcode::Negate => {
                    let val = self.stack.pop()?;
                    self.stack.push(JsValue::Number(-val.to_number()))?;
                }
                Opcode::Not => {
                    let val = self.stack.pop()?;
                    self.stack.push(JsValue::Boolean(!val.to_boolean()))?;
                }
                Opcode::Equal | Opcode::StrictEqual => {
                    let rhs = self.stack.pop()?;
                    let lhs = self.stack.pop()?;
                    self.stack.push(JsValue::Boolean(lhs == rhs))?;
                }
                Opcode::LessThan => {
                    let rhs = self.stack.pop()?;
                    let lhs = self.stack.pop()?;
                    self.stack
                        .push(JsValue::Boolean(lhs.to_number() < rhs.to_number()))?;
                }
                Opcode::GreaterThan => {
                    let rhs = self.stack.pop()?;
                    let lhs = self.stack.pop()?;
                    self.stack
                        .push(JsValue::Boolean(lhs.to_number() > rhs.to_number()))?;
                }
                Opcode::SetGlobal(name_idx) => {
                    let key = self.constant_name(&chunk, *name_idx)?;
                    let val = self.stack.pop()?;
                    self.globals.insert(key, val);
                }
                Opcode::GetGlobal(name_idx) => {
                    let key = self.constant_name(&chunk, *name_idx)?;
                    let val = self
                        .globals
                        .get(&key)
                        .cloned()
                        .unwrap_or(JsValue::Undefined);
                    self.stack.push(val)?;
                }
                Opcode::Pop => {
                    let _ = self.stack.pop()?;
                }
                Opcode::Nil => self.stack.push(JsValue::Null)?,
                Opcode::True => self.stack.push(JsValue::Boolean(true))?,
                Opcode::False => self.stack.push(JsValue::Boolean(false))?,
                Opcode::JumpIfFalse(target) => {
                    if !self.stack.peek()?.to_boolean() {
                        ip = *target as usize;
                        continue;
                    }
                }
                Opcode::Jump(target) => {
                    ip = *target as usize;
                    continue;
                }
                Opcode::Loop(target) => {
                    ip = *target as usize;
                    continue;
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
                        let mut interp = Interpreter::new();
                        interp.run(&program)?;
                        return Ok(());
                    }
                    return Err(RuntimeError::TypeError {
                        message: "RunTreeWalk opcode requires source text".to_string(),
                    });
                }
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: format!("unsupported opcode in VM: {:?}", chunk.instructions[ip]),
                    });
                }
            }
            ip += 1;
        }

        Ok(())
    }

    fn constant_name(&self, chunk: &Chunk, idx: u16) -> Result<String, RuntimeError> {
        match chunk.constants.get(idx as usize) {
            Some(JsValue::String(name)) => Ok(name.clone()),
            _ => Err(RuntimeError::TypeError {
                message: "global name constant must be a string".to_string(),
            }),
        }
    }

    fn exec_arithmetic(&mut self, op: &Opcode) -> Result<(), RuntimeError> {
        let rhs = self.stack.pop()?;
        let lhs = self.stack.pop()?;
        let ln = lhs.to_number();
        let rn = rhs.to_number();
        let out = match op {
            Opcode::Add => JsValue::Number(ln + rn),
            Opcode::Sub => JsValue::Number(ln - rn),
            Opcode::Mul => JsValue::Number(ln * rn),
            Opcode::Div => JsValue::Number(ln / rn),
            Opcode::Mod => JsValue::Number(ln % rn),
            _ => unreachable!(),
        };
        self.stack.push(out)
    }
}

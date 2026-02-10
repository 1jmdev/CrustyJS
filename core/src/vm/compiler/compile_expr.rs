use crate::parser::ast::{BinOp, Expr, Literal, UnaryOp};

use super::Compiler;
use crate::vm::bytecode::{Opcode, VmValue};

impl Compiler {
    pub fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.compile_literal(lit),
            Expr::Identifier(name) => {
                if let Some(local_idx) = self.resolve_local(name) {
                    self.chunk.write(Opcode::GetLocal(local_idx), 0);
                } else {
                    let idx = self.chunk.add_constant(VmValue::String(name.clone()));
                    self.chunk.write(Opcode::GetGlobal(idx), 0);
                }
            }
            Expr::Binary { left, op, right } => {
                self.compile_expr(left);
                self.compile_expr(right);
                self.compile_binop(op);
            }
            Expr::Unary { op, operand } => {
                self.compile_expr(operand);
                match op {
                    UnaryOp::Neg => self.chunk.write(Opcode::Negate, 0),
                    UnaryOp::Not => self.chunk.write(Opcode::Not, 0),
                }
            }
            Expr::Call { callee, args } => {
                self.compile_expr(callee);
                for arg in args {
                    self.compile_expr(arg);
                }
                self.chunk.write(Opcode::Call(args.len() as u8), 0);
            }
            Expr::MemberAccess { object, property } => {
                self.compile_expr(object);
                let idx = self.chunk.add_constant(VmValue::String(property.clone()));
                self.chunk.write(Opcode::Constant(idx), 0);
                self.chunk.write(Opcode::GetProperty, 0);
            }
            Expr::Typeof(inner) => {
                self.compile_expr(inner);
                self.chunk.write(Opcode::Typeof, 0);
            }
            Expr::ArrayLiteral { .. } => self.require_tree_walk(),
            Expr::ObjectLiteral { .. } => self.require_tree_walk(),
            Expr::Spread(_) => self.require_tree_walk(),
            _ => {
                self.require_tree_walk();
            }
        }
    }

    fn compile_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Number(n) => {
                let idx = self.chunk.add_constant(VmValue::Number(*n));
                self.chunk.write(Opcode::Constant(idx), 0);
            }
            Literal::String(s) => {
                let idx = self.chunk.add_constant(VmValue::String(s.clone()));
                self.chunk.write(Opcode::Constant(idx), 0);
            }
            Literal::Boolean(true) => self.chunk.write(Opcode::True, 0),
            Literal::Boolean(false) => self.chunk.write(Opcode::False, 0),
            Literal::Null | Literal::Undefined => self.chunk.write(Opcode::Nil, 0),
        }
    }

    fn compile_binop(&mut self, op: &BinOp) {
        match op {
            BinOp::Add => self.chunk.write(Opcode::Add, 0),
            BinOp::Sub => self.chunk.write(Opcode::Sub, 0),
            BinOp::Mul => self.chunk.write(Opcode::Mul, 0),
            BinOp::Div => self.chunk.write(Opcode::Div, 0),
            BinOp::Mod => self.chunk.write(Opcode::Mod, 0),
            BinOp::Less => self.chunk.write(Opcode::LessThan, 0),
            BinOp::LessEq => {
                self.chunk.write(Opcode::GreaterThan, 0);
                self.chunk.write(Opcode::Not, 0);
            }
            BinOp::Greater => self.chunk.write(Opcode::GreaterThan, 0),
            BinOp::GreaterEq => {
                self.chunk.write(Opcode::LessThan, 0);
                self.chunk.write(Opcode::Not, 0);
            }
            BinOp::EqEq | BinOp::EqEqEq => self.chunk.write(Opcode::Equal, 0),
            BinOp::NotEq | BinOp::NotEqEq => {
                self.chunk.write(Opcode::Equal, 0);
                self.chunk.write(Opcode::Not, 0);
            }
            _ => self.require_tree_walk(),
        }
    }
}

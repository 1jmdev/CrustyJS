use crate::parser::ast::{Expr, Stmt};

use super::Compiler;
use crate::vm::bytecode::{Opcode, VmFunction, VmValue};

impl Compiler {
    pub fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { pattern, init, .. } => {
                let Some(name) = pattern.as_identifier() else {
                    self.require_tree_walk();
                    return;
                };

                if let Some(expr) = init {
                    self.compile_expr(expr);
                } else {
                    self.chunk.write(Opcode::Nil, 0);
                }

                if self.scope_depth > 0 {
                    let local_idx = self.define_local(name.to_string());
                    self.chunk.write(Opcode::SetLocal(local_idx), 0);
                } else {
                    let idx = self.chunk.add_constant(VmValue::String(name.to_string()));
                    self.chunk.write(Opcode::SetGlobal(idx), 0);
                }
            }
            Stmt::ExprStmt(expr) => {
                if let Expr::Call { callee, args } = expr {
                    if let Expr::MemberAccess { object, property } = &**callee {
                        if let Expr::Identifier(name) = &**object {
                            if name == "console" && property == "log" && args.len() == 1 {
                                self.compile_expr(&args[0]);
                                self.chunk.write(Opcode::Print, 0);
                                return;
                            }
                        }
                    }
                }
                self.compile_expr(expr);
                self.chunk.write(Opcode::Pop, 0);
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
                self.end_scope();
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(condition);
                let jump_false_pos = self.chunk.instructions.len();
                self.chunk.write(Opcode::JumpIfFalse(0), 0);
                self.compile_stmt(then_branch);
                let jump_end_pos = self.chunk.instructions.len();
                self.chunk.write(Opcode::Jump(0), 0);
                let else_start = self.chunk.instructions.len() as u16;
                if let Some(else_stmt) = else_branch {
                    self.compile_stmt(else_stmt);
                }
                let end = self.chunk.instructions.len() as u16;
                self.chunk.instructions[jump_false_pos] = Opcode::JumpIfFalse(else_start);
                self.chunk.instructions[jump_end_pos] = Opcode::Jump(end);
            }
            Stmt::While { condition, body } => {
                let loop_start = self.chunk.instructions.len() as u16;
                self.compile_expr(condition);
                let jump_out_pos = self.chunk.instructions.len();
                self.chunk.write(Opcode::JumpIfFalse(0), 0);
                self.compile_stmt(body);
                self.chunk.write(Opcode::Loop(loop_start), 0);
                let end = self.chunk.instructions.len() as u16;
                self.chunk.instructions[jump_out_pos] = Opcode::JumpIfFalse(end);
            }
            Stmt::FunctionDecl {
                name, params, body, ..
            } => {
                let mut fn_compiler = Compiler::new();
                fn_compiler.scope_depth = 1;
                for param in params {
                    let Some(param_name) = param.pattern.as_identifier() else {
                        self.require_tree_walk();
                        return;
                    };
                    fn_compiler.define_local(param_name.to_string());
                }
                for stmt in body {
                    fn_compiler.compile_stmt(stmt);
                }
                if fn_compiler.requires_tree_walk {
                    self.require_tree_walk();
                    return;
                }
                fn_compiler.chunk.write(Opcode::Nil, 0);
                fn_compiler.chunk.write(Opcode::Return, 0);
                let function = VmFunction {
                    name: name.clone(),
                    arity: params.len(),
                    chunk: Box::new(fn_compiler.chunk),
                };
                let fn_idx = self
                    .chunk
                    .add_constant(VmValue::Function(Box::new(function)));
                self.chunk.write(Opcode::Constant(fn_idx), 0);
                let name_idx = self.chunk.add_constant(VmValue::String(name.clone()));
                self.chunk.write(Opcode::SetGlobal(name_idx), 0);
            }
            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.compile_expr(expr);
                } else {
                    self.chunk.write(Opcode::Nil, 0);
                }
                self.chunk.write(Opcode::Return, 0);
            }
            Stmt::ForLoop { .. }
            | Stmt::ForOf { .. }
            | Stmt::ForIn { .. }
            | Stmt::Break
            | Stmt::TryCatch { .. }
            | Stmt::Throw(_)
            | Stmt::Switch { .. }
            | Stmt::Class(_)
            | Stmt::Import(_)
            | Stmt::Export(_) => {
                self.require_tree_walk();
            }
        }
    }
}

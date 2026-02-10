use crate::parser::ast::Stmt;
use crate::runtime::value::JsValue;

use super::Compiler;
use crate::vm::bytecode::Opcode;

impl Compiler {
    pub fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, init } => {
                if let Some(expr) = init {
                    self.compile_expr(expr);
                } else {
                    self.chunk.write(Opcode::Nil, 0);
                }
                let idx = self.chunk.add_constant(JsValue::String(name.clone()));
                self.chunk.write(Opcode::SetGlobal(idx), 0);
            }
            Stmt::ExprStmt(expr) => {
                self.compile_expr(expr);
                self.chunk.write(Opcode::Pop, 0);
            }
            Stmt::Block(stmts) => {
                self.scope_depth += 1;
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
                self.scope_depth -= 1;
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
            Stmt::FunctionDecl { .. }
            | Stmt::Return(_)
            | Stmt::ForLoop { .. }
            | Stmt::ForOf { .. }
            | Stmt::TryCatch { .. }
            | Stmt::Throw(_)
            | Stmt::Class(_) => {
                self.chunk.write(Opcode::RunTreeWalk, 0);
            }
        }
    }
}

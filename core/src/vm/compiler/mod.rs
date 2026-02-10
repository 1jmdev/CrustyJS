mod compile_expr;
mod compile_stmt;

use crate::parser::ast::Program;

use super::bytecode::Chunk;

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub depth: usize,
}

pub struct Compiler {
    pub chunk: Chunk,
    pub locals: Vec<Local>,
    pub scope_depth: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn compile(&mut self, program: Program) -> Chunk {
        self.compile_program(&program);
        self.chunk.clone()
    }

    pub fn compile_program(&mut self, program: &Program) {
        for stmt in &program.body {
            self.compile_stmt(stmt);
        }
    }
}

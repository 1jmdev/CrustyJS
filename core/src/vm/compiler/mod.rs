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
    pub requires_tree_walk: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            requires_tree_walk: false,
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

    pub(crate) fn resolve_local(&self, name: &str) -> Option<u16> {
        self.locals
            .iter()
            .rposition(|local| local.name == name)
            .map(|idx| idx as u16)
    }

    pub(crate) fn define_local(&mut self, name: String) -> u16 {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
        (self.locals.len() - 1) as u16
    }

    pub(crate) fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub(crate) fn end_scope(&mut self) {
        if self.scope_depth == 0 {
            return;
        }
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    pub(crate) fn require_tree_walk(&mut self) {
        self.requires_tree_walk = true;
    }
}

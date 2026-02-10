mod builtins;
mod error_handling;
mod eval_class;
mod eval_expr;
mod eval_pattern;
mod eval_stmt;
mod global_builtins;

use crate::errors::RuntimeError;
use crate::parser::ast::Program;
use crate::runtime::environment::Environment;
use std::collections::HashMap;

/// Control flow signal from statement evaluation.
pub(crate) enum ControlFlow {
    None,
    Return(crate::runtime::value::JsValue),
}

/// The tree-walk interpreter.
pub struct Interpreter {
    pub(crate) env: Environment,
    pub(crate) output: Vec<String>,
    pub(crate) classes: HashMap<String, eval_class::RuntimeClass>,
    pub(crate) super_stack: Vec<Option<String>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interp = Self {
            env: Environment::new(),
            output: Vec::new(),
            classes: HashMap::new(),
            super_stack: Vec::new(),
        };
        interp.init_builtins();
        interp
    }

    /// Run a parsed program.
    pub fn run(&mut self, program: &Program) -> Result<(), RuntimeError> {
        for stmt in &program.body {
            if let ControlFlow::Return(_) = self.eval_stmt(stmt)? {
                break;
            }
        }
        Ok(())
    }

    /// Get captured output lines (from console.log calls).
    pub fn output(&self) -> &[String] {
        &self.output
    }
}

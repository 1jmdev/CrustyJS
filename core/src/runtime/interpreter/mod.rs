mod builtins;
mod error_handling;
mod eval_expr;
mod eval_stmt;

use crate::errors::RuntimeError;
use crate::parser::ast::Program;
use crate::runtime::environment::Environment;

/// Control flow signal from statement evaluation.
pub(crate) enum ControlFlow {
    None,
    Return(crate::runtime::value::JsValue),
}

/// The tree-walk interpreter.
pub struct Interpreter {
    pub(crate) env: Environment,
    pub(crate) output: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            output: Vec::new(),
        }
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

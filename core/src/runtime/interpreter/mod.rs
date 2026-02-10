mod builtins;
mod error_handling;
mod eval_async;
mod eval_class;
mod eval_expr;
mod eval_expr_helpers;
mod eval_pattern;
mod eval_stmt;
mod eval_stmt_control;
mod event_loop_driver;
mod function_call;
mod global_builtins;
mod module_runtime;
mod object_json_date_builtins;
mod promise_runtime;
mod property_access;

use crate::errors::RuntimeError;
use crate::parser::ast::Program;
use crate::runtime::environment::Environment;
use crate::runtime::event_loop::EventLoop;
use crate::runtime::modules::cache::ModuleCache;
use std::collections::HashMap;
use std::path::PathBuf;

/// Control flow signal from statement evaluation.
pub(crate) enum ControlFlow {
    None,
    Return(crate::runtime::value::JsValue),
    Break,
}

/// The tree-walk interpreter.
pub struct Interpreter {
    pub(crate) env: Environment,
    pub(crate) output: Vec<String>,
    pub(crate) classes: HashMap<String, eval_class::RuntimeClass>,
    pub(crate) super_stack: Vec<Option<String>>,
    pub(crate) event_loop: EventLoop,
    pub(crate) async_depth: usize,
    pub(crate) module_cache: ModuleCache,
    pub(crate) module_stack: Vec<PathBuf>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self::new_with_realtime_timers(false)
    }

    pub fn new_with_realtime_timers(realtime_timers: bool) -> Self {
        let mut interp = Self {
            env: Environment::new(),
            output: Vec::new(),
            classes: HashMap::new(),
            super_stack: Vec::new(),
            event_loop: EventLoop::new_with_realtime(realtime_timers),
            async_depth: 0,
            module_cache: ModuleCache::default(),
            module_stack: Vec::new(),
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
        self.run_event_loop_until_idle()?;
        Ok(())
    }

    pub fn run_with_path(&mut self, program: &Program, path: PathBuf) -> Result<(), RuntimeError> {
        self.module_stack.push(path);
        let out = self.run(program);
        self.module_stack.pop();
        out
    }

    /// Get captured output lines (from console.log calls).
    pub fn output(&self) -> &[String] {
        &self.output
    }
}

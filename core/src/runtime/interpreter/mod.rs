mod builtins;
mod collection_builtins;
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

use crate::diagnostics::source_map::{SourceMap, SourcePos};
use crate::diagnostics::stack_trace::CallStack;
use crate::errors::RuntimeError;
use crate::parser::ast::Program;
use crate::runtime::environment::Environment;
use crate::runtime::event_loop::EventLoop;
use crate::runtime::modules::cache::ModuleCache;
use crate::runtime::value::symbol::SymbolRegistry;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// Control flow signal from statement evaluation.
pub(crate) enum ControlFlow {
    None,
    Return(crate::runtime::value::JsValue),
    Break,
    Yield(crate::runtime::value::JsValue),
}

/// The tree-walk interpreter.
pub struct Interpreter {
    pub(crate) env: Environment,
    pub(crate) output: Vec<String>,
    pub(crate) classes: HashMap<String, eval_class::RuntimeClass>,
    pub(crate) super_stack: Vec<Option<String>>,
    pub(crate) event_loop: EventLoop,
    pub(crate) async_depth: usize,
    pub(crate) generator_depth: usize,
    pub(crate) generator_yields: Vec<crate::runtime::value::JsValue>,
    pub(crate) module_cache: ModuleCache,
    pub(crate) module_stack: Vec<PathBuf>,
    pub(crate) call_stack: CallStack,
    pub(crate) source_maps: HashMap<String, SourceMap>,
    pub(crate) start_time: Instant,
    pub(crate) symbol_registry: SymbolRegistry,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
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
            generator_depth: 0,
            generator_yields: Vec::new(),
            module_cache: ModuleCache::default(),
            module_stack: Vec::new(),
            call_stack: CallStack::default(),
            source_maps: HashMap::new(),
            start_time: Instant::now(),
            symbol_registry: SymbolRegistry::new(),
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
        let file = path.display().to_string();
        self.ensure_source_map_for_path(&path);
        self.module_stack.push(path);
        self.call_stack
            .push_frame(crate::diagnostics::stack_trace::CallFrame {
                function_name: "<global>".to_string(),
                file,
                line: 1,
                col: 1,
            });
        let out = self.run(program).map_err(|err| {
            let trace = self.call_stack.format_trace();
            self.attach_stack_to_error(err, &trace)
        });
        self.call_stack.pop_frame();
        self.module_stack.pop();
        out
    }

    /// Get captured output lines (from console.log calls).
    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn current_stack_trace(&self) -> String {
        self.call_stack.format_trace()
    }

    pub(crate) fn register_source_map(&mut self, path: &std::path::Path, source: &str) {
        self.source_maps
            .insert(path.display().to_string(), SourceMap::from_source(source));
    }

    pub(crate) fn source_pos_for(&self, path: &str, offset: usize) -> SourcePos {
        self.source_maps
            .get(path)
            .map(|m| m.byte_to_pos(offset))
            .unwrap_or(SourcePos { line: 1, col: 1 })
    }

    pub(crate) fn ensure_source_map_for_path(&mut self, path: &std::path::Path) {
        let file = path.display().to_string();
        if self.source_maps.contains_key(&file) {
            return;
        }
        if let Ok(source) = std::fs::read_to_string(path) {
            self.register_source_map(path, &source);
        }
    }
}

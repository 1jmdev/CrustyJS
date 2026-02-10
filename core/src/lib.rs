pub mod context;
pub mod diagnostics;
pub mod embedding;
pub mod engine;
pub mod errors;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod vm;

pub use context::Context;
pub use embedding::{ClassBuilder, EventTarget, NativeClassDef};
pub use engine::Engine;
pub use runtime::value::JsValue as Value;

use errors::CrustyError;
use runtime::interpreter::Interpreter;

/// Convenience function to run JavaScript source code end-to-end.
pub fn run(source: &str) -> Result<Interpreter, CrustyError> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    let mut interp = Interpreter::new_with_realtime_timers(true);
    interp.run(&program)?;
    Ok(interp)
}

/// Execute source through the VM path.
pub fn run_vm(source: &str) -> Result<(), CrustyError> {
    vm::run_vm(source)
}

pub fn run_vm_with_path(source: &str, path: Option<std::path::PathBuf>) -> Result<(), CrustyError> {
    vm::run_vm_with_path(source, path)
}

pub mod bytecode;
pub mod compiler;
pub mod machine;

use crate::errors::CrustyError;
use crate::lexer;
use crate::parser;
use crate::runtime::interpreter::Interpreter;
use std::path::PathBuf;

pub fn run_vm(source: &str) -> Result<(), CrustyError> {
    run_vm_with_path(source, None)
}

pub fn run_vm_with_path(source: &str, path: Option<PathBuf>) -> Result<(), CrustyError> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    let mut compiler = compiler::Compiler::new();
    let chunk = compiler.compile(program.clone());
    if compiler.requires_tree_walk {
        let mut interp = Interpreter::new_with_realtime_timers(true);
        let exec_path = path.unwrap_or_else(|| PathBuf::from("."));
        interp.run_with_path(&program, exec_path)?;
        return Ok(());
    }
    let mut vm = machine::VM::new();
    vm.run(chunk, Some(source.to_string()), path)?;
    Ok(())
}

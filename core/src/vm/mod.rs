pub mod bytecode;
pub mod compiler;
pub mod machine;

use crate::errors::CrustyError;
use crate::lexer;
use crate::parser;

pub fn run_vm(source: &str) -> Result<(), CrustyError> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    let mut compiler = compiler::Compiler::new();
    let chunk = compiler.compile(program);
    let mut vm = machine::VM::new();
    vm.run(chunk, Some(source.to_string()))?;
    Ok(())
}

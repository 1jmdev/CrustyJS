pub mod errors;
pub mod lexer;
pub mod parser;
pub mod runtime;

use errors::CrustyError;
use runtime::interpreter::Interpreter;

/// Convenience function to run JavaScript source code end-to-end.
pub fn run(source: &str) -> Result<Interpreter, CrustyError> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    let mut interp = Interpreter::new();
    interp.run(&program)?;
    Ok(interp)
}

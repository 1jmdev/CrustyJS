pub mod cursor;
pub mod scanner;
mod string_scanner;
pub mod token;

use crate::errors::SyntaxError;
use token::Token;

/// Tokenize source code into a list of tokens.
pub fn lex(source: &str) -> Result<Vec<Token>, SyntaxError> {
    let mut scanner = scanner::Scanner::new(source);
    scanner.scan_tokens()
}

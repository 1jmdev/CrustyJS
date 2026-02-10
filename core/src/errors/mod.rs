mod runtime_error;
mod syntax_error;

pub use runtime_error::RuntimeError;
pub use syntax_error::SyntaxError;

use miette::Diagnostic;
use thiserror::Error;

/// Unified error type wrapping all CrustyJS errors.
#[derive(Debug, Error, Diagnostic)]
pub enum CrustyError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Syntax(#[from] SyntaxError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    Runtime(#[from] RuntimeError),
}

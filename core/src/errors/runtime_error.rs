use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum RuntimeError {
    #[error("'{name}' is not defined")]
    UndefinedVariable { name: String },

    #[error("'{name}' is not a function")]
    NotAFunction { name: String },

    #[error("expected {expected} arguments but got {got}")]
    ArityMismatch { expected: usize, got: usize },

    #[error("type error: {message}")]
    TypeError { message: String },
}

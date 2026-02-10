use miette::Diagnostic;
use thiserror::Error;

use crate::runtime::value::JsValue;

#[derive(Debug, Error, Diagnostic)]
pub enum RuntimeError {
    #[error("ReferenceError: '{name}' is not defined")]
    #[diagnostic(help("declare '{name}' with let or const before using it"))]
    UndefinedVariable { name: String },

    #[error("TypeError: '{name}' is not a function")]
    #[diagnostic(help("ensure '{name}' is declared as a function before calling it"))]
    NotAFunction { name: String },

    #[error("TypeError: expected {expected} arguments but got {got}")]
    ArityMismatch { expected: usize, got: usize },

    #[error("TypeError: {message}")]
    TypeError { message: String },

    #[error("TypeError: Assignment to constant variable '{name}'")]
    ConstReassignment { name: String },

    #[error("Uncaught {value}")]
    Thrown { value: JsValue },
}

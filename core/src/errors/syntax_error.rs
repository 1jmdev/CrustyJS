use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[error("SyntaxError: {message}")]
#[diagnostic(help("check the syntax around this location"))]
pub struct SyntaxError {
    pub message: String,

    #[label("here")]
    pub span: miette::SourceSpan,
}

impl SyntaxError {
    pub fn new(message: impl Into<String>, offset: usize, length: usize) -> Self {
        Self {
            message: message.into(),
            span: (offset, length).into(),
        }
    }
}

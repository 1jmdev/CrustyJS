use crate::context::Context;

/// Top-level engine entry point for embedders.
#[derive(Debug, Default, Clone, Copy)]
pub struct Engine;

impl Engine {
    pub fn new() -> Self {
        Self
    }

    pub fn new_context(&self) -> Context {
        Context::new()
    }
}

use crate::vm::bytecode::Chunk;

#[derive(Clone)]
pub struct CallFrame {
    pub chunk: Chunk,
    pub ip: usize,
    pub slot: usize,
}

impl CallFrame {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            slot: 0,
        }
    }
}

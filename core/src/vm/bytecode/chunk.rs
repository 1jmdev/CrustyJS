use super::{Opcode, VmValue};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Opcode>,
    pub constants: Vec<VmValue>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, op: Opcode, line: usize) {
        self.instructions.push(op);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: VmValue) -> u16 {
        self.constants.push(value);
        (self.constants.len() - 1) as u16
    }
}

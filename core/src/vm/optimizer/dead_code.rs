use std::collections::HashSet;

use crate::vm::bytecode::{Chunk, Opcode};

pub fn eliminate_dead_code(chunk: &mut Chunk) {
    let jump_targets = collect_jump_targets(&chunk.instructions);
    let len = chunk.instructions.len();
    let mut i = 0;
    while i < len {
        let is_terminator = matches!(chunk.instructions[i], Opcode::Return | Opcode::Jump(_));
        if !is_terminator {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut j = start;
        while j < len {
            if jump_targets.contains(&j) {
                break;
            }
            chunk.instructions[j] = Opcode::Nop;
            j += 1;
        }
        i = j;
    }
}

fn collect_jump_targets(instructions: &[Opcode]) -> HashSet<usize> {
    let mut targets = HashSet::new();
    for op in instructions {
        match op {
            Opcode::Jump(t) | Opcode::JumpIfFalse(t) | Opcode::Loop(t) => {
                targets.insert(*t as usize);
            }
            _ => {}
        }
    }
    targets
}

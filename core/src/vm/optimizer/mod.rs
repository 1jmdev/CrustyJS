mod constant_fold;
mod dead_code;

use crate::vm::bytecode::Chunk;

pub fn optimize(chunk: &mut Chunk) {
    constant_fold::constant_fold(chunk);
    dead_code::eliminate_dead_code(chunk);
}

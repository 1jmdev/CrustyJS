use crate::vm::bytecode::{Chunk, Opcode, VmValue};

pub fn constant_fold(chunk: &mut Chunk) {
    let len = chunk.instructions.len();
    if len < 3 {
        return;
    }
    let mut i = 0;
    while i + 2 < len {
        let folded = try_fold(chunk, i);
        if let Some((result_value, op_offset)) = folded {
            let const_idx = chunk.add_constant(result_value);
            chunk.instructions[i] = Opcode::Constant(const_idx);
            chunk.instructions[i + 1] = Opcode::Nop;
            chunk.instructions[i + op_offset] = Opcode::Nop;
            i += op_offset + 1;
        } else {
            i += 1;
        }
    }
}

fn try_fold(chunk: &Chunk, i: usize) -> Option<(VmValue, usize)> {
    let (a_idx, b_idx) = match (&chunk.instructions[i], &chunk.instructions[i + 1]) {
        (Opcode::Constant(a), Opcode::Constant(b)) => (*a, *b),
        _ => return None,
    };
    let op = &chunk.instructions[i + 2];
    let a = extract_number(&chunk.constants, a_idx)?;
    let b = extract_number(&chunk.constants, b_idx)?;
    let result = match op {
        Opcode::Add => a + b,
        Opcode::Sub => a - b,
        Opcode::Mul => a * b,
        Opcode::Div => {
            if b == 0.0 {
                return None;
            }
            a / b
        }
        _ => return None,
    };
    Some((VmValue::Number(result), 2))
}

fn extract_number(constants: &[VmValue], idx: u16) -> Option<f64> {
    match constants.get(idx as usize) {
        Some(VmValue::Number(n)) => Some(*n),
        _ => None,
    }
}

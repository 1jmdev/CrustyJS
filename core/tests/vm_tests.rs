use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::vm::bytecode::Opcode;
use crustyjs::vm::compiler::Compiler;

fn compile_source(source: &str) -> Vec<Opcode> {
    let tokens = lex(source).expect("lex failed");
    let program = parse(tokens).expect("parse failed");
    let mut compiler = Compiler::new();
    let chunk = compiler.compile(program);
    chunk.instructions
}

#[test]
fn compile_simple_expression_emits_arithmetic_opcode() {
    let ops = compile_source("1 + 2;");
    assert!(ops.contains(&Opcode::Add));
}

#[test]
fn compile_if_else_emits_jump_opcodes() {
    let ops = compile_source("if (1 < 2) { 1; } else { 2; }");
    assert!(ops.iter().any(|op| matches!(op, Opcode::JumpIfFalse(_))));
    assert!(ops.iter().any(|op| matches!(op, Opcode::Jump(_))));
}

#[test]
fn compile_while_emits_loop_opcode() {
    let ops = compile_source("let i = 0; while (i < 3) { i = i + 1; }");
    assert!(ops.iter().any(|op| matches!(op, Opcode::Loop(_))));
}

#[test]
fn vm_path_runs_fib_example() {
    let source = std::fs::read_to_string("examples/fib.js").expect("read fib example");
    crustyjs::run_vm(&source).expect("vm run should succeed");
}

#[test]
fn vm_path_runs_classes_example() {
    let source = std::fs::read_to_string("examples/classes.js").expect("read classes example");
    crustyjs::run_vm(&source).expect("vm run should succeed");
}

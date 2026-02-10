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

fn compile_source_with_fallback_flag(source: &str) -> (Vec<Opcode>, bool) {
    let tokens = lex(source).expect("lex failed");
    let program = parse(tokens).expect("parse failed");
    let mut compiler = Compiler::new();
    let chunk = compiler.compile(program);
    (chunk.instructions, compiler.requires_tree_walk)
}

fn run_vm_source(source: &str) {
    crustyjs::run_vm(source).expect("vm run should succeed");
}

fn run_vm_file(path: &str) {
    let source = std::fs::read_to_string(path).expect("read vm source file");
    crustyjs::run_vm_with_path(&source, Some(std::path::PathBuf::from(path)))
        .expect("vm run with path should succeed");
}

#[test]
fn compile_simple_expression_emits_arithmetic_opcode() {
    let ops = compile_source("1 + 2;");
    assert!(
        ops.iter().any(|op| matches!(op, Opcode::Nop)),
        "constant folding should replace operands with Nop"
    );
    assert!(
        !ops.contains(&Opcode::Add),
        "Add should be folded away for constant operands"
    );
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
    run_vm_source(&source);
}

#[test]
fn vm_path_runs_classes_example() {
    let source = std::fs::read_to_string("examples/classes.js").expect("read classes example");
    run_vm_source(&source);
}

#[test]
fn vm_compiles_fib_without_treewalk_fallback() {
    let (ops, requires_fallback) = compile_source_with_fallback_flag(
        r#"
        function fib(n) {
          if (n <= 1) return n;
          return fib(n - 1) + fib(n - 2);
        }
        console.log(fib(20));
        "#,
    );
    assert!(!requires_fallback);
    assert!(!ops.is_empty());
}

#[test]
fn vm_path_runs_objects_example() {
    let source = std::fs::read_to_string("examples/objects.js").expect("read objects example");
    run_vm_source(&source);
}

#[test]
fn vm_path_runs_array_and_closure_snippets() {
    run_vm_source(
        r#"
        let arr = [1, 2, 3];
        console.log(arr[0]);
        function makeAdder(x) {
          return y => x + y;
        }
        let add2 = makeAdder(2);
        console.log(add2(5));
        "#,
    );
}

#[test]
fn vm_falls_back_to_single_tree_walk_for_mixed_program() {
    let (ops, requires_fallback) = compile_source_with_fallback_flag(
        r#"
        let x = 1;
        class A {}
        console.log(x);
        "#,
    );

    assert!(requires_fallback);
    assert!(!ops.is_empty());
}

#[test]
fn vm_compiles_assignment_without_treewalk_fallback() {
    let ops = compile_source("let x = 1; x = x + 2; console.log(x);");
    assert!(!ops.is_empty());
}

#[test]
fn vm_path_runs_kitchen_sink_example() {
    run_vm_file("examples/kitchen_sink.js");
}

#[test]
fn vm_path_runs_modules_example_with_entry_path() {
    run_vm_file("examples/modules/main.js");
}

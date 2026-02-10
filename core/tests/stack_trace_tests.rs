use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;

#[test]
fn nested_runtime_error_includes_stack_frames() {
    let source = r#"
function inner() {
  missingVar;
}

function outer() {
  inner();
}

outer();
"#;

    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    let err = interp
        .run_with_path(&program, std::path::PathBuf::from("stack_test.js"))
        .expect_err("program should fail");
    let msg = err.to_string();

    assert!(msg.contains("inner"));
    assert!(msg.contains("outer"));
    assert!(msg.contains("at"));
}

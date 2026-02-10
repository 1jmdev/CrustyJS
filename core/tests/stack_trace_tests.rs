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

    let path =
        std::env::temp_dir().join(format!("crustyjs_stack_{}_nested.js", std::process::id()));
    std::fs::write(&path, source).expect("write source");

    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    let err = interp
        .run_with_path(&program, path)
        .expect_err("program should fail");
    let msg = err.to_string();

    assert!(msg.contains("inner"));
    assert!(msg.contains("outer"));
    assert!(msg.contains(":2:1"));
    assert!(msg.contains(":6:1"));
    assert!(msg.contains(":1:1"));
    assert!(msg.contains("at"));
}

#[test]
fn async_runtime_error_preserves_stack_trace() {
    let source = r#"
async function boom() {
  await Promise.resolve(1);
  missingAfterAwait;
}

boom().catch(e => console.log(e));
"#;

    let path = std::env::temp_dir().join(format!("crustyjs_stack_{}_async.js", std::process::id()));
    std::fs::write(&path, source).expect("write source");

    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    interp
        .run_with_path(&program, path)
        .expect("program should execute");
    let msg = interp.output().join("\n");

    assert!(msg.contains("boom"));
    assert!(msg.contains(":2:"));
}

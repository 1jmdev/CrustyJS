use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;

fn run_and_capture(source: &str) -> Vec<String> {
    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    interp.run(&program).expect("execution should succeed");
    interp.output().to_vec()
}

#[test]
fn math_constants() {
    let output = run_and_capture(
        r#"
        console.log(Math.PI);
        console.log(Math.E);
        "#,
    );

    assert_eq!(output.len(), 2);
    assert!(output[0].starts_with("3.14159"));
    assert!(output[1].starts_with("2.71828"));
}

#[test]
fn math_methods_basic() {
    let output = run_and_capture(
        r#"
        console.log(Math.floor(4.7));
        console.log(Math.max(1, 5, 3));
        console.log(Math.abs(-5));
        console.log(Math.sqrt(16));
        console.log(Math.pow(2, 10));
        console.log(Math.round(4.5));
        "#,
    );

    assert_eq!(output, vec!["4", "5", "5", "4", "1024", "5"]);
}

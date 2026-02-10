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
fn string_plus_string() {
    let output = run_and_capture(r#"console.log("a" + "b");"#);
    assert_eq!(output, vec!["ab"]);
}

#[test]
fn string_plus_number() {
    let output = run_and_capture(r#"console.log("x" + 5);"#);
    assert_eq!(output, vec!["x5"]);
}

#[test]
fn number_plus_string() {
    let output = run_and_capture(r#"console.log(5 + "x");"#);
    assert_eq!(output, vec!["5x"]);
}

#[test]
fn empty_string_plus_true() {
    let output = run_and_capture(r#"console.log("" + true);"#);
    assert_eq!(output, vec!["true"]);
}

#[test]
fn string_plus_null() {
    let output = run_and_capture(r#"console.log("val:" + null);"#);
    assert_eq!(output, vec!["val:null"]);
}

#[test]
fn string_plus_undefined() {
    let output = run_and_capture(r#"console.log("val:" + undefined);"#);
    assert_eq!(output, vec!["val:undefined"]);
}

#[test]
fn multi_concat_with_coercion() {
    let output = run_and_capture(
        r#"
        let greeting = "Hello" + " " + "World";
        console.log(greeting);
        "#,
    );
    assert_eq!(output, vec!["Hello World"]);
}

#[test]
fn number_concat_in_expression() {
    let output = run_and_capture(r#"console.log("count: " + 42);"#);
    assert_eq!(output, vec!["count: 42"]);
}

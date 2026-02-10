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

// --- String properties ---

#[test]
fn string_length() {
    let output = run_and_capture(
        r#"
        let s = "hello";
        console.log(s.length);
        "#,
    );
    assert_eq!(output, vec!["5"]);
}

// --- String methods ---

#[test]
fn string_to_upper_case() {
    let output = run_and_capture(
        r#"
        let s = "hello";
        console.log(s.toUpperCase());
        "#,
    );
    assert_eq!(output, vec!["HELLO"]);
}

#[test]
fn string_to_lower_case() {
    let output = run_and_capture(
        r#"
        let s = "HELLO";
        console.log(s.toLowerCase());
        "#,
    );
    assert_eq!(output, vec!["hello"]);
}

#[test]
fn string_includes() {
    let output = run_and_capture(
        r#"
        let s = "hello world";
        console.log(s.includes("world"));
        console.log(s.includes("xyz"));
        "#,
    );
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn string_index_of() {
    let output = run_and_capture(
        r#"
        let s = "hello";
        console.log(s.indexOf("ell"));
        console.log(s.indexOf("xyz"));
        "#,
    );
    assert_eq!(output, vec!["1", "-1"]);
}

#[test]
fn string_slice() {
    let output = run_and_capture(
        r#"
        let s = "hello world";
        console.log(s.slice(0, 5));
        console.log(s.slice(6, 11));
        "#,
    );
    assert_eq!(output, vec!["hello", "world"]);
}

#[test]
fn string_trim() {
    let output = run_and_capture(
        r#"
        let s = "  hello  ";
        console.log(s.trim());
        "#,
    );
    assert_eq!(output, vec!["hello"]);
}

// --- Template literals ---

#[test]
fn template_literal_no_interpolation() {
    let output = run_and_capture("console.log(`hello world`);");
    assert_eq!(output, vec!["hello world"]);
}

#[test]
fn template_literal_with_variable() {
    let output = run_and_capture(
        r#"
        let name = "world";
        console.log(`hello ${name}`);
        "#,
    );
    assert_eq!(output, vec!["hello world"]);
}

#[test]
fn template_literal_with_expression() {
    let output = run_and_capture("console.log(`2+2=${2+2}`);");
    assert_eq!(output, vec!["2+2=4"]);
}

#[test]
fn template_literal_multiple_interpolations() {
    let output = run_and_capture(
        r#"
        let a = "foo";
        let b = "bar";
        console.log(`${a} and ${b}`);
        "#,
    );
    assert_eq!(output, vec!["foo and bar"]);
}

use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;

fn run_and_capture(source: &str) -> Vec<String> {
    let tokens = lex(source).expect("lex failed");
    let program = parse(tokens).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.run(&program).expect("runtime error");
    interp.output().to_vec()
}

#[test]
fn array_literal_display() {
    let out = run_and_capture("let arr = [1, 2, 3]; console.log(arr);");
    assert_eq!(out, vec!["[1, 2, 3]"]);
}

#[test]
fn array_index_access() {
    let out = run_and_capture("let arr = [10, 20, 30]; console.log(arr[0]); console.log(arr[2]);");
    assert_eq!(out, vec!["10", "30"]);
}

#[test]
fn array_length() {
    let out = run_and_capture("let arr = [1, 2, 3, 4]; console.log(arr.length);");
    assert_eq!(out, vec!["4"]);
}

#[test]
fn array_out_of_bounds() {
    let out = run_and_capture("let arr = [1, 2]; console.log(arr[5]);");
    assert_eq!(out, vec!["undefined"]);
}

#[test]
fn array_index_assignment() {
    let out = run_and_capture("let arr = [1, 2, 3]; arr[1] = 99; console.log(arr[1]);");
    assert_eq!(out, vec!["99"]);
}

#[test]
fn empty_array() {
    let out = run_and_capture("let arr = []; console.log(arr.length); console.log(arr);");
    assert_eq!(out, vec!["0", "[]"]);
}

#[test]
fn array_of_strings() {
    let out = run_and_capture(r#"let arr = ["a", "b", "c"]; console.log(arr);"#);
    assert_eq!(out, vec!["[a, b, c]"]);
}

#[test]
fn array_mixed_types() {
    let out = run_and_capture(r#"let arr = [1, "two", true, null]; console.log(arr);"#);
    assert_eq!(out, vec!["[1, two, true, null]"]);
}

#[test]
fn array_dynamic_index() {
    let src = r#"
        let arr = [10, 20, 30];
        let i = 1;
        console.log(arr[i]);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["20"]);
}

#[test]
fn array_grow_via_assignment() {
    let src = r#"
        let arr = [1];
        arr[3] = 99;
        console.log(arr.length);
        console.log(arr[1]);
        console.log(arr[3]);
    "#;
    let out = run_and_capture(src);
    assert_eq!(out, vec!["4", "undefined", "99"]);
}

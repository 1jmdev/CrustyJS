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
fn switch_basic_case_match() {
    let output = run_and_capture(
        r#"
        let val = 2;
        switch (val) {
            case 1: console.log("one"); break;
            case 2: console.log("two"); break;
            default: console.log("other");
        }
        "#,
    );

    assert_eq!(output, vec!["two"]);
}

#[test]
fn switch_fallthrough_until_break() {
    let output = run_and_capture(
        r#"
        let val = 1;
        switch (val) {
            case 1:
                console.log("one");
            case 2:
                console.log("two");
                break;
            default:
                console.log("other");
        }
        "#,
    );

    assert_eq!(output, vec!["one", "two"]);
}

#[test]
fn switch_default_when_no_match() {
    let output = run_and_capture(
        r#"
        switch (42) {
            case 1: console.log("one"); break;
            default: console.log("other");
        }
        "#,
    );

    assert_eq!(output, vec!["other"]);
}

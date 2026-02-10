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
fn promise_constructor_and_then() {
    let output = run_and_capture(
        r#"
        const p = new Promise((resolve, reject) => {
          resolve(42);
        });
        p.then(v => console.log(v));
        "#,
    );

    assert_eq!(output, vec!["42"]);
}

#[test]
fn promise_chaining_and_catch() {
    let output = run_and_capture(
        r#"
        Promise.resolve(1)
          .then(v => v + 1)
          .then(v => v * 2)
          .then(v => console.log(v));

        Promise.reject("fail")
          .catch(e => console.log(e));
        "#,
    );

    assert_eq!(output, vec!["fail", "4"]);
}

#[test]
fn promise_finally_runs_and_passes_value() {
    let output = run_and_capture(
        r#"
        Promise.resolve(3)
          .finally(() => console.log("done"))
          .then(v => console.log(v));
        "#,
    );

    assert_eq!(output, vec!["done", "3"]);
}

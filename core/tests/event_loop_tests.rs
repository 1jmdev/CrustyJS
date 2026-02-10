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
fn microtasks_run_before_macrotasks() {
    let output = run_and_capture(
        r#"
        Promise.resolve("micro").then(v => console.log(v));
        setTimeout(() => console.log("macro"), 0);
        "#,
    );

    assert_eq!(output, vec!["micro", "macro"]);
}

#[test]
fn timeout_order_uses_delay() {
    let output = run_and_capture(
        r#"
        setTimeout(() => console.log("later"), 20);
        setTimeout(() => console.log("soon"), 5);
        "#,
    );

    assert_eq!(output, vec!["soon", "later"]);
}

#[test]
fn interval_can_be_cleared() {
    let output = run_and_capture(
        r#"
        let i = 0;
        const id = setInterval(() => {
          i = i + 1;
          console.log(i);
          if (i === 3) {
            clearInterval(id);
          }
        }, 1);
        "#,
    );

    assert_eq!(output, vec!["1", "2", "3"]);
}

#[test]
fn queue_microtask_runs_before_timeout() {
    let output = run_and_capture(
        r#"
        setTimeout(() => console.log("timeout"), 0);
        queueMicrotask(() => console.log("microtask"));
        "#,
    );

    assert_eq!(output, vec!["microtask", "timeout"]);
}

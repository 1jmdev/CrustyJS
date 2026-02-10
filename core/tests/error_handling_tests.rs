use crustyjs::errors::RuntimeError;
use crustyjs::lexer::lex;
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;
use crustyjs::runtime::value::JsValue;

fn run_and_capture(source: &str) -> Vec<String> {
    let tokens = lex(source).expect("lex failed");
    let program = parse(tokens).expect("parse failed");
    let mut interp = Interpreter::new();
    interp.run(&program).expect("runtime error");
    interp.output().to_vec()
}

fn run_and_error(source: &str) -> RuntimeError {
    let tokens = lex(source).expect("lex failed");
    let program = parse(tokens).expect("parse failed");
    let mut interp = Interpreter::new();
    interp
        .run(&program)
        .expect_err("runtime should fail with an uncaught exception")
}

#[test]
fn catches_thrown_error_object() {
    let output = run_and_capture(
        r#"
        try {
          throw new Error("oops");
        } catch (e) {
          console.log(e.message);
        }
        "#,
    );
    assert_eq!(output, vec!["oops"]);
}

#[test]
fn finally_runs_on_success_and_throw() {
    let output = run_and_capture(
        r#"
        try {
          console.log("work");
        } finally {
          console.log("cleanup-1");
        }

        try {
          throw "bad";
        } catch (e) {
          console.log(e);
        } finally {
          console.log("cleanup-2");
        }
        "#,
    );
    assert_eq!(output, vec!["work", "cleanup-1", "bad", "cleanup-2"]);
}

#[test]
fn nested_try_catch_works() {
    let output = run_and_capture(
        r#"
        try {
          try {
            throw new Error("inner");
          } catch (e) {
            console.log(e.message);
            throw new Error("outer");
          }
        } catch (e) {
          console.log(e.message);
        }
        "#,
    );
    assert_eq!(output, vec!["inner", "outer"]);
}

#[test]
fn uncaught_exception_bubbles_out() {
    let err = run_and_error(
        r#"
        throw new Error("boom");
        "#,
    );

    match err {
        RuntimeError::Thrown { value } => {
            let JsValue::Object(obj) = value else {
                panic!("expected thrown object");
            };
            let message = obj.borrow().get("message").expect("message should exist");
            assert_eq!(message, JsValue::String("boom".to_string()));
        }
        other => panic!("expected thrown error, got {other:?}"),
    }
}

#[test]
fn error_constructor_exists_globally() {
    let output = run_and_capture("console.log(typeof Error);");
    assert_eq!(output, vec!["function"]);
}

#[test]
fn catch_body_allows_missing_semicolon_before_brace() {
    let output = run_and_capture(
        r#"
        try {
          throw new Error("boom");
        } catch (e) {
          console.log(e.message)
        }
        "#,
    );
    assert_eq!(output, vec!["boom"]);
}

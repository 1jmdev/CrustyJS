use crustyjs::errors::RuntimeError;
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

fn run_and_error(source: &str) -> RuntimeError {
    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    interp.run(&program).expect_err("execution should fail")
}

#[test]
fn console_log_arithmetic() {
    let output = run_and_capture("console.log(2 + 3);");
    assert_eq!(output, vec!["5"]);
}

#[test]
fn variable_declaration_and_use() {
    let output = run_and_capture("let x = 10; console.log(x + 5);");
    assert_eq!(output, vec!["15"]);
}

#[test]
fn if_else_branch() {
    let output = run_and_capture(
        r#"
        let x = 5;
        if (x > 3) {
            console.log("big");
        } else {
            console.log("small");
        }
        "#,
    );
    assert_eq!(output, vec!["big"]);
}

#[test]
fn function_call() {
    let output = run_and_capture(
        r#"
        function double(n) {
            return n * 2;
        }
        console.log(double(21));
        "#,
    );
    assert_eq!(output, vec!["42"]);
}

#[test]
fn while_loop() {
    let output = run_and_capture(
        r#"
        let i = 0;
        let sum = 0;
        while (i < 5) {
            sum = sum + i;
            i = i + 1;
        }
        console.log(sum);
        "#,
    );
    assert_eq!(output, vec!["10"]);
}

#[test]
fn recursive_fibonacci() {
    let output = run_and_capture(
        r#"
        function fib(n) {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }
        console.log(fib(10));
        "#,
    );
    assert_eq!(output, vec!["55"]);
}

#[test]
fn string_concatenation() {
    let output = run_and_capture(r#"console.log("hello" + " " + "world");"#);
    assert_eq!(output, vec!["hello world"]);
}

#[test]
fn boolean_logic() {
    let output = run_and_capture(
        r#"
        console.log(!false);
        console.log(!true);
        "#,
    );
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn strict_equality() {
    let output = run_and_capture(
        r#"
        console.log(1 === 1);
        console.log(1 !== 2);
        console.log(1 === 2);
        "#,
    );
    assert_eq!(output, vec!["true", "true", "false"]);
}

#[test]
fn unary_negation() {
    let output = run_and_capture("console.log(-5 + 3);");
    assert_eq!(output, vec!["-2"]);
}

#[test]
fn logical_and_or_nullish() {
    let output = run_and_capture(
        r#"
        console.log(true && "yes");
        console.log(false && "no");
        console.log("left" || "right");
        console.log("" || "fallback");
        console.log(null ?? "n");
        console.log(0 ?? 42);
        "#,
    );
    assert_eq!(output, vec!["yes", "false", "left", "fallback", "n", "0"]);
}

#[test]
fn ternary_expression() {
    let output = run_and_capture(
        r#"
        let x = 3;
        console.log(x > 2 ? "big" : "small");
        console.log(x < 2 ? 1 : 0);
        "#,
    );
    assert_eq!(output, vec!["big", "0"]);
}

#[test]
fn compound_assignment_and_modulo() {
    let output = run_and_capture(
        r#"
        let x = 10;
        x += 5;
        x -= 2;
        x *= 3;
        x /= 2;
        console.log(x);
        console.log(x % 4);
        "#,
    );
    assert_eq!(output, vec!["19.5", "3.5"]);
}

#[test]
fn prefix_and_postfix_updates() {
    let output = run_and_capture(
        r#"
        let x = 10;
        console.log(++x);
        console.log(x++);
        console.log(x);
        console.log(--x);
        console.log(x--);
        console.log(x);
        "#,
    );
    assert_eq!(output, vec!["11", "11", "12", "11", "11", "10"]);
}

#[test]
fn typeof_operator() {
    let output = run_and_capture(
        r#"
        console.log(typeof 42);
        console.log(typeof "hello");
        console.log(typeof true);
        console.log(typeof undefined);
        console.log(typeof null);
        console.log(typeof {});
        console.log(typeof (x => x));
        "#,
    );
    assert_eq!(
        output,
        vec![
            "number",
            "string",
            "boolean",
            "undefined",
            "object",
            "object",
            "function"
        ]
    );
}

#[test]
fn loose_equality_with_coercion() {
    let output = run_and_capture(
        r#"
        console.log(1 == "1");
        console.log(0 == false);
        console.log(null == undefined);
        console.log("" == false);
        console.log(1 != "1");
        "#,
    );
    assert_eq!(output, vec!["true", "true", "true", "true", "false"]);
}

#[test]
fn computed_property_name() {
    let output = run_and_capture(
        r#"
        let prop = "name";
        let obj = { [prop]: "Rex" };
        console.log(obj.name);
        "#,
    );
    assert_eq!(output, vec!["Rex"]);
}

#[test]
fn for_in_loop_over_object_keys() {
    let output = run_and_capture(
        r#"
        let obj = { a: 1, b: 2 };
        for (let key in obj) {
            console.log(key);
        }
        "#,
    );
    assert_eq!(output.len(), 2);
    assert!(output.contains(&"a".to_string()));
    assert!(output.contains(&"b".to_string()));
}

#[test]
fn const_reassignment_throws() {
    let err = run_and_error("const x = 10; x = 20;");
    assert!(matches!(err, RuntimeError::ConstReassignment { .. }));
}

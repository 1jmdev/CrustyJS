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

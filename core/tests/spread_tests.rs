use crustyjs::lexer::lex;
use crustyjs::parser::ast::{Expr, Stmt};
use crustyjs::parser::parse;
use crustyjs::runtime::interpreter::Interpreter;

fn parse_source(source: &str) -> Vec<Stmt> {
    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    program.body
}

fn run_and_capture(source: &str) -> Vec<String> {
    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    let mut interp = Interpreter::new();
    interp.run(&program).expect("execution should succeed");
    interp.output().to_vec()
}

#[test]
fn parse_spread_in_array_and_call() {
    let stmts = parse_source("let a = [1, 2]; let b = [...a, 3]; console.log(Math.max(...a));");
    assert_eq!(stmts.len(), 3);

    match &stmts[1] {
        Stmt::VarDecl {
            init: Some(Expr::ArrayLiteral { elements }),
            ..
        } => {
            assert!(matches!(elements[0], Expr::Spread(_)));
        }
        other => panic!("expected array literal var declaration, got {other:?}"),
    }

    match &stmts[2] {
        Stmt::ExprStmt(Expr::Call { args, .. }) => {
            assert!(!args.is_empty());
        }
        other => panic!("expected call expression, got {other:?}"),
    }
}

#[test]
fn eval_array_spread_and_call_spread() {
    let output = run_and_capture(
        r#"
        const base = [1, 2, 3];
        const copy = [...base, 4, ...[5]];
        console.log(copy.length);
        console.log(copy[0]);
        console.log(copy[4]);

        function sum(a, b, c) { return a + b + c; }
        console.log(sum(...base));
        "#,
    );

    assert_eq!(output, vec!["5", "1", "5", "6"]);
}

#[test]
fn eval_multiple_spreads_in_call() {
    let output = run_and_capture(
        r#"
        function join(a, b, c, d) { return a + b + c + d; }
        const left = ["A", "B"];
        const right = ["C", "D"];
        console.log(join(...left, ...right));
        "#,
    );

    assert_eq!(output, vec!["ABCD"]);
}

#[test]
fn eval_object_spread_and_rest_destructuring() {
    let output = run_and_capture(
        r#"
        const obj1 = { a: 1, b: 2 };
        const obj2 = { ...obj1, c: 3 };
        const { a, ...rest } = obj2;
        console.log(a);
        console.log(rest.b);
        console.log(rest.c);
        "#,
    );

    assert_eq!(output, vec!["1", "2", "3"]);
}

#[test]
fn eval_object_spread_overwrite_order() {
    let output = run_and_capture(
        r#"
        const base = { a: 1, b: 2 };
        const merged = { ...base, b: 9, ...{ c: 3 } };
        console.log(merged.a);
        console.log(merged.b);
        console.log(merged.c);
        "#,
    );

    assert_eq!(output, vec!["1", "9", "3"]);
}

use crustyjs::lexer::lex;
use crustyjs::parser::ast::{Expr, Literal, Pattern, Stmt};
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
fn parse_object_destructuring_with_alias_and_default() {
    let stmts = parse_source("const { name, age: years = 0 } = person;");
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::VarDecl {
            pattern: Pattern::ObjectPattern { properties },
            init: Some(Expr::Identifier(init_name)),
        } => {
            assert_eq!(init_name, "person");
            assert_eq!(properties.len(), 2);

            assert_eq!(properties[0].key, "name");
            assert!(properties[0].alias.is_none());
            assert!(properties[0].default.is_none());

            assert_eq!(properties[1].key, "age");
            assert_eq!(
                properties[1].alias,
                Some(Pattern::Identifier("years".to_string()))
            );
            assert_eq!(
                properties[1].default,
                Some(Expr::Literal(Literal::Number(0.0)))
            );
        }
        other => panic!("expected object pattern declaration, got {other:?}"),
    }
}

#[test]
fn parse_nested_array_destructuring_with_rest() {
    let stmts = parse_source("let [first, , [nested], ...rest] = items;");
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::VarDecl {
            pattern: Pattern::ArrayPattern { elements },
            init: Some(Expr::Identifier(init_name)),
        } => {
            assert_eq!(init_name, "items");
            assert_eq!(elements.len(), 4);

            assert_eq!(elements[0], Some(Pattern::Identifier("first".to_string())));
            assert_eq!(elements[1], None);

            assert_eq!(
                elements[2],
                Some(Pattern::ArrayPattern {
                    elements: vec![Some(Pattern::Identifier("nested".to_string()))],
                })
            );

            assert_eq!(
                elements[3],
                Some(Pattern::Rest(Box::new(Pattern::Identifier(
                    "rest".to_string()
                ))))
            );
        }
        other => panic!("expected array pattern declaration, got {other:?}"),
    }
}

#[test]
fn parse_nested_object_destructuring() {
    let stmts = parse_source("const { user: { name: displayName = \"anon\" } } = payload;");
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::VarDecl {
            pattern: Pattern::ObjectPattern { properties },
            ..
        } => {
            assert_eq!(properties.len(), 1);
            assert_eq!(properties[0].key, "user");
            match &properties[0].alias {
                Some(Pattern::ObjectPattern {
                    properties: nested_props,
                }) => {
                    assert_eq!(nested_props.len(), 1);
                    assert_eq!(nested_props[0].key, "name");
                    assert_eq!(
                        nested_props[0].alias,
                        Some(Pattern::Identifier("displayName".to_string()))
                    );
                    assert_eq!(
                        nested_props[0].default,
                        Some(Expr::Literal(Literal::String("anon".to_string())))
                    );
                }
                other => panic!("expected nested object alias pattern, got {other:?}"),
            }
        }
        other => panic!("expected object pattern declaration, got {other:?}"),
    }
}

#[test]
fn eval_object_destructuring_with_defaults_and_alias() {
    let output = run_and_capture(
        r#"
        let person = { name: "Alice" };
        const { name, age: years = 30 } = person;
        console.log(name);
        console.log(years);
        "#,
    );

    assert_eq!(output, vec!["Alice", "30"]);
}

#[test]
fn eval_array_destructuring_with_holes_and_rest() {
    let output = run_and_capture(
        r#"
        const values = [1, 2, 3, 4];
        let [first, , third, ...rest] = values;
        console.log(first);
        console.log(third);
        console.log(rest.length);
        console.log(rest[0]);
        "#,
    );

    assert_eq!(output, vec!["1", "3", "1", "4"]);
}

#[test]
fn eval_nested_destructuring_with_object_rest() {
    let output = run_and_capture(
        r#"
        const data = { id: 7, user: { name: "Rex" }, role: "admin", active: true };
        const { user: { name }, ...rest } = data;
        console.log(name);
        console.log(rest.id);
        console.log(rest.role);
        console.log(rest.active);
        "#,
    );

    assert_eq!(output, vec!["Rex", "7", "admin", "true"]);
}

#[test]
fn parse_function_param_destructuring() {
    let stmts = parse_source("function greet({ name, age = 0 }) { return name; }");
    assert_eq!(stmts.len(), 1);

    match &stmts[0] {
        Stmt::FunctionDecl { params, .. } => {
            assert_eq!(params.len(), 1);
            match &params[0].pattern {
                Pattern::ObjectPattern { properties } => {
                    assert_eq!(properties.len(), 2);
                    assert_eq!(properties[0].key, "name");
                    assert_eq!(properties[1].key, "age");
                    assert_eq!(
                        properties[1].default,
                        Some(Expr::Literal(Literal::Number(0.0)))
                    );
                }
                other => panic!("expected object pattern parameter, got {other:?}"),
            }
        }
        other => panic!("expected function declaration, got {other:?}"),
    }
}

#[test]
fn eval_function_param_destructuring_with_defaults() {
    let output = run_and_capture(
        r#"
        function greet({ name, age = 0 }) {
            console.log(name + " is " + age);
        }

        greet({ name: "Alice", age: 30 });
        greet({ name: "Bob" });
        "#,
    );

    assert_eq!(output, vec!["Alice is 30", "Bob is 0"]);
}

#[test]
fn eval_arrow_param_destructuring() {
    let output = run_and_capture(
        r#"
        const read = ({ name }) => name;
        console.log(read({ name: "Rex" }));
        "#,
    );

    assert_eq!(output, vec!["Rex"]);
}

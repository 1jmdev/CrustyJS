use crustyjs::lexer::lex;
use crustyjs::parser::ast::{BinOp, Expr, Literal, ObjectProperty, Param, Pattern, Stmt};
use crustyjs::parser::parse;

fn parse_source(source: &str) -> Vec<Stmt> {
    let tokens = lex(source).expect("lexing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    program.body
}

#[test]
fn parse_variable_declaration() {
    let stmts = parse_source("let x = 42;");
    assert_eq!(stmts.len(), 1);
    assert_eq!(
        stmts[0],
        Stmt::VarDecl {
            pattern: Pattern::Identifier("x".into()),
            init: Some(Expr::Literal(Literal::Number(42.0))),
        }
    );
}

#[test]
fn parse_if_else() {
    let stmts = parse_source("if (x <= 1) return x; else return 0;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => {
            assert!(matches!(
                condition,
                Expr::Binary {
                    op: BinOp::LessEq,
                    ..
                }
            ));
            assert!(matches!(**then_branch, Stmt::Return(Some(_))));
            assert!(else_branch.is_some());
        }
        other => panic!("expected If statement, got {:?}", other),
    }
}

#[test]
fn parse_function_declaration() {
    let stmts = parse_source("function fib(n) { return n; }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::FunctionDecl { name, params, body } => {
            assert_eq!(name, "fib");
            assert_eq!(
                params,
                &[Param {
                    pattern: Pattern::Identifier("n".to_string()),
                    default: None,
                }]
            );
            assert_eq!(body.len(), 1);
        }
        other => panic!("expected FunctionDecl, got {:?}", other),
    }
}

#[test]
fn parse_call_expression() {
    let stmts = parse_source("fib(10);");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::ExprStmt(Expr::Call { callee, args }) => {
            assert_eq!(**callee, Expr::Identifier("fib".into()));
            assert_eq!(args.len(), 1);
            assert_eq!(args[0], Expr::Literal(Literal::Number(10.0)));
        }
        other => panic!("expected call ExprStmt, got {:?}", other),
    }
}

#[test]
fn parse_binary_precedence() {
    // 1 + 2 * 3 should parse as 1 + (2 * 3)
    let stmts = parse_source("1 + 2 * 3;");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::ExprStmt(Expr::Binary {
            left,
            op: BinOp::Add,
            right,
        }) => {
            assert_eq!(**left, Expr::Literal(Literal::Number(1.0)));
            assert!(matches!(**right, Expr::Binary { op: BinOp::Mul, .. }));
        }
        other => panic!("expected binary Add, got {:?}", other),
    }
}

#[test]
fn parse_member_access_call() {
    let stmts = parse_source("console.log(42);");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::ExprStmt(Expr::Call { callee, args }) => {
            assert!(matches!(
                **callee,
                Expr::MemberAccess {
                    ref property,
                    ..
                } if property == "log"
            ));
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected member call, got {:?}", other),
    }
}

#[test]
fn parse_full_fib_program() {
    let source = r#"
        function fib(n) {
            if (n <= 1) return n;
            return fib(n - 1) + fib(n - 2);
        }
        console.log(fib(10));
    "#;
    let stmts = parse_source(source);
    assert_eq!(stmts.len(), 2);
    assert!(matches!(stmts[0], Stmt::FunctionDecl { .. }));
    assert!(matches!(stmts[1], Stmt::ExprStmt(Expr::Call { .. })));
}

#[test]
fn parse_try_catch_finally_statement() {
    let stmts =
        parse_source("try { throw 1; } catch (e) { console.log(e); } finally { console.log(2); }");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::TryCatch {
            catch_param,
            catch_block,
            finally_block,
            ..
        } => {
            assert_eq!(catch_param.as_deref(), Some("e"));
            assert!(catch_block.is_some());
            assert!(finally_block.is_some());
        }
        other => panic!("expected TryCatch, got {other:?}"),
    }
}

#[test]
fn parse_new_expression() {
    let stmts = parse_source("new Error(\"oops\");");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::ExprStmt(Expr::New { callee, args }) => {
            assert_eq!(**callee, Expr::Identifier("Error".into()));
            assert_eq!(args.len(), 1);
        }
        other => panic!("expected New expression, got {other:?}"),
    }
}

#[test]
fn parse_expression_without_semicolon_at_eof() {
    let stmts = parse_source("console.log(1)");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(stmts[0], Stmt::ExprStmt(Expr::Call { .. })));
}

#[test]
fn parse_class_with_extends_and_constructor() {
    let src = r#"
        class Dog extends Animal {
            constructor(name) {
                super(name);
            }
            speak() {
                return "woof";
            }
        }
    "#;
    let stmts = parse_source(src);
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::Class(class_decl) => {
            assert_eq!(class_decl.name, "Dog");
            assert_eq!(class_decl.parent.as_deref(), Some("Animal"));
            assert!(class_decl.constructor.is_some());
            assert_eq!(class_decl.methods.len(), 1);
        }
        other => panic!("expected class declaration, got {other:?}"),
    }
}

#[test]
fn parse_object_method_shorthand() {
    let stmts = parse_source("let obj = { speak() { return 1; } }; ");
    assert_eq!(stmts.len(), 1);
    match &stmts[0] {
        Stmt::VarDecl {
            init: Some(Expr::ObjectLiteral { properties }),
            ..
        } => {
            assert_eq!(properties.len(), 1);
            assert!(matches!(
                properties[0],
                ObjectProperty::KeyValue(ref key, _) if key == "speak"
            ));
        }
        other => panic!("expected object literal var decl, got {other:?}"),
    }
}

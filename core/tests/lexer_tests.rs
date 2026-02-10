use crustyjs::lexer::{lex, token::TokenKind};

fn token_kinds(source: &str) -> Vec<TokenKind> {
    lex(source)
        .expect("lexing should succeed")
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

#[test]
fn lex_variable_declaration() {
    let kinds = token_kinds("let x = 42;");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Ident("x".into()),
            TokenKind::Assign,
            TokenKind::Number(42.0),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_function_header() {
    let kinds = token_kinds("function fib(n) {");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Function,
            TokenKind::Ident("fib".into()),
            TokenKind::LeftParen,
            TokenKind::Ident("n".into()),
            TokenKind::RightParen,
            TokenKind::LeftBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_operators() {
    let kinds = token_kinds("a + b - c * d / e");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::Plus,
            TokenKind::Ident("b".into()),
            TokenKind::Minus,
            TokenKind::Ident("c".into()),
            TokenKind::Star,
            TokenKind::Ident("d".into()),
            TokenKind::Slash,
            TokenKind::Ident("e".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_comparison_operators() {
    let kinds = token_kinds("a <= b >= c === d !== e");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::LessEq,
            TokenKind::Ident("b".into()),
            TokenKind::GreaterEq,
            TokenKind::Ident("c".into()),
            TokenKind::EqEqEq,
            TokenKind::Ident("d".into()),
            TokenKind::NotEqEq,
            TokenKind::Ident("e".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_string_literal() {
    let kinds = token_kinds(r#""hello world""#);
    assert_eq!(
        kinds,
        vec![TokenKind::String("hello world".into()), TokenKind::Eof,]
    );
}

#[test]
fn lex_boolean_and_null() {
    let kinds = token_kinds("true false null undefined");
    assert_eq!(
        kinds,
        vec![
            TokenKind::True,
            TokenKind::False,
            TokenKind::Null,
            TokenKind::Undefined,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_fib_function_body() {
    let source = "if (n <= 1) return n; return fib(n - 1) + fib(n - 2);";
    let kinds = token_kinds(source);
    assert_eq!(
        kinds,
        vec![
            TokenKind::If,
            TokenKind::LeftParen,
            TokenKind::Ident("n".into()),
            TokenKind::LessEq,
            TokenKind::Number(1.0),
            TokenKind::RightParen,
            TokenKind::Return,
            TokenKind::Ident("n".into()),
            TokenKind::Semicolon,
            TokenKind::Return,
            TokenKind::Ident("fib".into()),
            TokenKind::LeftParen,
            TokenKind::Ident("n".into()),
            TokenKind::Minus,
            TokenKind::Number(1.0),
            TokenKind::RightParen,
            TokenKind::Plus,
            TokenKind::Ident("fib".into()),
            TokenKind::LeftParen,
            TokenKind::Ident("n".into()),
            TokenKind::Minus,
            TokenKind::Number(2.0),
            TokenKind::RightParen,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_skips_line_comment() {
    let kinds = token_kinds("let x = 1; // this is a comment\nlet y = 2;");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Let,
            TokenKind::Ident("x".into()),
            TokenKind::Assign,
            TokenKind::Number(1.0),
            TokenKind::Semicolon,
            TokenKind::Let,
            TokenKind::Ident("y".into()),
            TokenKind::Assign,
            TokenKind::Number(2.0),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_decimal_number() {
    let kinds = token_kinds("3.14");
    assert_eq!(kinds, vec![TokenKind::Number(3.14), TokenKind::Eof,]);
}

#[test]
fn lex_member_access() {
    let kinds = token_kinds("console.log");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("console".into()),
            TokenKind::Dot,
            TokenKind::Ident("log".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_arrow_function_tokens() {
    let kinds = token_kinds("x => x + 1");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("x".into()),
            TokenKind::Arrow,
            TokenKind::Ident("x".into()),
            TokenKind::Plus,
            TokenKind::Number(1.0),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_logical_and_update_tokens() {
    let kinds = token_kinds("a && b || c ?? d; x += 1; y++; z %= 2;");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::AmpAmp,
            TokenKind::Ident("b".into()),
            TokenKind::PipePipe,
            TokenKind::Ident("c".into()),
            TokenKind::NullishCoalescing,
            TokenKind::Ident("d".into()),
            TokenKind::Semicolon,
            TokenKind::Ident("x".into()),
            TokenKind::PlusEquals,
            TokenKind::Number(1.0),
            TokenKind::Semicolon,
            TokenKind::Ident("y".into()),
            TokenKind::PlusPlus,
            TokenKind::Semicolon,
            TokenKind::Ident("z".into()),
            TokenKind::PercentEquals,
            TokenKind::Number(2.0),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_typeof_and_loose_equality_tokens() {
    let kinds = token_kinds("typeof x == \"1\" != false");
    assert_eq!(
        kinds,
        vec![
            TokenKind::Typeof,
            TokenKind::Ident("x".into()),
            TokenKind::EqEq,
            TokenKind::String("1".into()),
            TokenKind::NotEq,
            TokenKind::False,
            TokenKind::Eof,
        ]
    );
}

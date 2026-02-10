use crustyjs::lexer::{lex, token::TokenKind};

#[test]
fn lex_error_handling_keywords() {
    let kinds: Vec<TokenKind> = lex("try catch finally throw new Error")
        .expect("lexing should succeed")
        .into_iter()
        .map(|t| t.kind)
        .collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Try,
            TokenKind::Catch,
            TokenKind::Finally,
            TokenKind::Throw,
            TokenKind::New,
            TokenKind::Ident("Error".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lex_class_keywords() {
    let kinds: Vec<TokenKind> = lex("class Dog extends Animal { super instanceof }")
        .expect("lexing should succeed")
        .into_iter()
        .map(|t| t.kind)
        .collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Class,
            TokenKind::Ident("Dog".into()),
            TokenKind::Extends,
            TokenKind::Ident("Animal".into()),
            TokenKind::LeftBrace,
            TokenKind::Super,
            TokenKind::Instanceof,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ]
    );
}

/// A single token with its kind and source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// Byte offset span in the source string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number(f64),
    String(String),
    True,
    False,
    Null,
    Undefined,

    // Template literals
    TemplateHead(String),
    TemplateMiddle(String),
    TemplateTail(String),
    NoSubTemplate(String),

    // Identifier
    Ident(String),

    // Keywords
    Let,
    Const,
    Function,
    If,
    Else,
    Return,
    While,
    For,
    Of,
    Typeof,
    Try,
    Catch,
    Finally,
    Throw,
    New,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    Arrow,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,
    PercentEquals,
    PlusPlus,
    MinusMinus,
    AmpAmp,
    PipePipe,
    NullishCoalescing,
    Question,
    EqEqEq,
    NotEqEq,
    EqEq,
    NotEq,
    LessEq,
    GreaterEq,
    Less,
    Greater,
    Bang,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Dot,
    Colon,

    // Special
    Eof,
}

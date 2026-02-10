use super::cursor::Cursor;
use super::number_ident_scanner::is_ident_start;
use super::token::{Span, Token, TokenKind};
use crate::errors::SyntaxError;

/// Scans source code into a sequence of tokens.
pub struct Scanner<'src> {
    pub(super) cursor: Cursor<'src>,
    pub(super) pending: Vec<Token>,
    pub(super) template_depth: usize,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            cursor: Cursor::new(source),
            pending: Vec::new(),
            template_depth: 0,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, SyntaxError> {
        let mut tokens = Vec::new();

        loop {
            if let Some(tok) = self.pending.pop() {
                tokens.push(tok);
                continue;
            }
            self.skip_whitespace_and_comments();
            if self.cursor.is_at_end() {
                break;
            }
            let prev_kind = tokens.last().map(|t: &Token| &t.kind).cloned();
            let token = self.scan_token_with_context(prev_kind.as_ref())?;
            tokens.push(token);
        }

        let eof_pos = self.cursor.pos();
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(eof_pos, eof_pos),
        });

        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.cursor.peek() {
                Some(b' ' | b'\t' | b'\r' | b'\n') => {
                    self.cursor.advance();
                }
                Some(b'/') if self.cursor.peek_next() == Some(b'/') => {
                    while let Some(ch) = self.cursor.peek() {
                        if ch == b'\n' {
                            break;
                        }
                        self.cursor.advance();
                    }
                }
                Some(b'/') if self.cursor.peek_next() == Some(b'*') => {
                    self.cursor.advance();
                    self.cursor.advance();
                    loop {
                        match self.cursor.advance() {
                            Some(b'*') if self.cursor.peek() == Some(b'/') => {
                                self.cursor.advance();
                                break;
                            }
                            None => break,
                            _ => {}
                        }
                    }
                }
                _ => break,
            }
        }
    }

    fn scan_token_with_context(&mut self, prev: Option<&TokenKind>) -> Result<Token, SyntaxError> {
        let start = self.cursor.pos();

        // Regex literal disambiguation: if we see '/' and the previous
        // token is NOT a value-producing token, treat as regex.
        if self.cursor.peek() == Some(b'/') && !is_division_context(prev) {
            return self.scan_regex_literal(start);
        }

        self.scan_token()
    }

    fn scan_regex_literal(&mut self, start: usize) -> Result<Token, SyntaxError> {
        self.cursor.advance(); // consume opening '/'

        let mut pattern = String::new();
        let mut in_char_class = false;

        loop {
            match self.cursor.peek() {
                None | Some(b'\n') => {
                    return Err(SyntaxError::new(
                        "unterminated regex literal",
                        start,
                        self.cursor.pos() - start,
                    ));
                }
                Some(b'\\') => {
                    self.cursor.advance();
                    pattern.push('\\');
                    if let Some(escaped) = self.cursor.advance() {
                        pattern.push(escaped as char);
                    }
                }
                Some(b'[') => {
                    in_char_class = true;
                    self.cursor.advance();
                    pattern.push('[');
                }
                Some(b']') if in_char_class => {
                    in_char_class = false;
                    self.cursor.advance();
                    pattern.push(']');
                }
                Some(b'/') if !in_char_class => {
                    self.cursor.advance(); // consume closing '/'
                    break;
                }
                Some(ch) => {
                    self.cursor.advance();
                    pattern.push(ch as char);
                }
            }
        }

        // Scan flags
        let mut flags = String::new();
        while let Some(ch) = self.cursor.peek() {
            if ch.is_ascii_alphabetic() {
                flags.push(ch as char);
                self.cursor.advance();
            } else {
                break;
            }
        }

        let end = self.cursor.pos();
        Ok(Token {
            kind: TokenKind::RegexLiteral { pattern, flags },
            span: Span::new(start, end),
        })
    }

    pub(super) fn scan_token(&mut self) -> Result<Token, SyntaxError> {
        let start = self.cursor.pos();
        let ch = self.cursor.advance().unwrap();

        let kind = match ch {
            b'(' => TokenKind::LeftParen,
            b')' => TokenKind::RightParen,
            b'{' => TokenKind::LeftBrace,
            b'}' => {
                if self.template_depth > 0 {
                    return self.scan_template_continue(start);
                }
                TokenKind::RightBrace
            }
            b',' => TokenKind::Comma,
            b';' => TokenKind::Semicolon,
            b'.' => {
                if self.cursor.match_char(b'.') && self.cursor.match_char(b'.') {
                    TokenKind::DotDotDot
                } else {
                    TokenKind::Dot
                }
            }
            b':' => TokenKind::Colon,
            b'[' => TokenKind::LeftBracket,
            b']' => TokenKind::RightBracket,
            b'+' => {
                if self.cursor.match_char(b'+') {
                    TokenKind::PlusPlus
                } else if self.cursor.match_char(b'=') {
                    TokenKind::PlusEquals
                } else {
                    TokenKind::Plus
                }
            }
            b'-' => {
                if self.cursor.match_char(b'-') {
                    TokenKind::MinusMinus
                } else if self.cursor.match_char(b'=') {
                    TokenKind::MinusEquals
                } else {
                    TokenKind::Minus
                }
            }
            b'*' => {
                if self.cursor.match_char(b'=') {
                    TokenKind::StarEquals
                } else {
                    TokenKind::Star
                }
            }
            b'/' => {
                if self.cursor.match_char(b'=') {
                    TokenKind::SlashEquals
                } else {
                    TokenKind::Slash
                }
            }
            b'%' => {
                if self.cursor.match_char(b'=') {
                    TokenKind::PercentEquals
                } else {
                    TokenKind::Percent
                }
            }
            b'&' => {
                if self.cursor.match_char(b'&') {
                    TokenKind::AmpAmp
                } else {
                    return Err(SyntaxError::new("unexpected '&'", start, 1));
                }
            }
            b'|' => {
                if self.cursor.match_char(b'|') {
                    TokenKind::PipePipe
                } else {
                    return Err(SyntaxError::new("unexpected '|'", start, 1));
                }
            }
            b'?' => {
                if self.cursor.match_char(b'?') {
                    TokenKind::NullishCoalescing
                } else if self.cursor.match_char(b'.') {
                    TokenKind::QuestionDot
                } else {
                    TokenKind::Question
                }
            }
            b'<' => {
                if self.cursor.match_char(b'=') {
                    TokenKind::LessEq
                } else {
                    TokenKind::Less
                }
            }
            b'>' => {
                if self.cursor.match_char(b'=') {
                    TokenKind::GreaterEq
                } else {
                    TokenKind::Greater
                }
            }
            b'=' => {
                if self.cursor.match_char(b'>') {
                    TokenKind::Arrow
                } else if self.cursor.match_char(b'=') {
                    if self.cursor.match_char(b'=') {
                        TokenKind::EqEqEq
                    } else {
                        TokenKind::EqEq
                    }
                } else {
                    TokenKind::Assign
                }
            }
            b'!' => {
                if self.cursor.match_char(b'=') {
                    if self.cursor.match_char(b'=') {
                        TokenKind::NotEqEq
                    } else {
                        TokenKind::NotEq
                    }
                } else {
                    TokenKind::Bang
                }
            }
            b'"' | b'\'' => self.scan_string(ch, start)?,
            b'`' => return self.scan_template(start),
            c if c.is_ascii_digit() => self.scan_number(start),
            c if is_ident_start(c) => self.scan_identifier(start),
            _ => {
                return Err(SyntaxError::new(
                    format!("unexpected character '{}'", ch as char),
                    start,
                    1,
                ));
            }
        };

        let end = self.cursor.pos();
        Ok(Token {
            kind,
            span: Span::new(start, end),
        })
    }
}

/// Returns true when the previous token indicates that `/` should be
/// parsed as division rather than the start of a regex literal.
fn is_division_context(prev: Option<&TokenKind>) -> bool {
    matches!(
        prev,
        Some(
            TokenKind::Number(_)
                | TokenKind::String(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Undefined
                | TokenKind::Ident(_)
                | TokenKind::RightParen
                | TokenKind::RightBracket
                | TokenKind::RightBrace
                | TokenKind::PlusPlus
                | TokenKind::MinusMinus
                | TokenKind::NoSubTemplate(_)
                | TokenKind::TemplateTail(_)
                | TokenKind::RegexLiteral { .. }
        )
    )
}

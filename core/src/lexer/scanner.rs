use super::cursor::Cursor;
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
            let token = self.scan_token()?;
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
                    // Line comment: skip until newline
                    while let Some(ch) = self.cursor.peek() {
                        if ch == b'\n' {
                            break;
                        }
                        self.cursor.advance();
                    }
                }
                Some(b'/') if self.cursor.peek_next() == Some(b'*') => {
                    // Block comment: skip until */
                    self.cursor.advance(); // skip /
                    self.cursor.advance(); // skip *
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

    fn scan_token(&mut self) -> Result<Token, SyntaxError> {
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
            b'.' => TokenKind::Dot,
            b':' => TokenKind::Colon,
            b'[' => TokenKind::LeftBracket,
            b']' => TokenKind::RightBracket,
            b'+' => TokenKind::Plus,
            b'-' => TokenKind::Minus,
            b'*' => TokenKind::Star,
            b'/' => TokenKind::Slash,
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
                } else if self.cursor.match_char(b'=') && self.cursor.match_char(b'=') {
                    TokenKind::EqEqEq
                } else {
                    TokenKind::Assign
                }
            }
            b'!' => {
                if self.cursor.match_char(b'=') && self.cursor.match_char(b'=') {
                    TokenKind::NotEqEq
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

    fn scan_number(&mut self, start: usize) -> TokenKind {
        while let Some(c) = self.cursor.peek() {
            if c.is_ascii_digit() {
                self.cursor.advance();
            } else {
                break;
            }
        }

        if self.cursor.peek() == Some(b'.')
            && self.cursor.peek_next().is_some_and(|c| c.is_ascii_digit())
        {
            self.cursor.advance(); // consume '.'
            while let Some(c) = self.cursor.peek() {
                if c.is_ascii_digit() {
                    self.cursor.advance();
                } else {
                    break;
                }
            }
        }

        let text = self.cursor.slice_from(start);
        let value: f64 = text.parse().expect("scanned digits should parse as f64");
        TokenKind::Number(value)
    }

    fn scan_identifier(&mut self, start: usize) -> TokenKind {
        while let Some(c) = self.cursor.peek() {
            if is_ident_continue(c) {
                self.cursor.advance();
            } else {
                break;
            }
        }

        let text = self.cursor.slice_from(start);
        match text {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "function" => TokenKind::Function,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "return" => TokenKind::Return,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "of" => TokenKind::Of,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "undefined" => TokenKind::Undefined,
            _ => TokenKind::Ident(text.to_owned()),
        }
    }
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}

fn is_ident_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

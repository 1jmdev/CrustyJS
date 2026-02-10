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
}

use super::scanner::Scanner;
use super::token::TokenKind;
use crate::errors::SyntaxError;

impl<'src> Scanner<'src> {
    pub(super) fn scan_number(&mut self, start: usize) -> TokenKind {
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
            self.cursor.advance();
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

    pub(super) fn scan_identifier(&mut self, start: usize) -> Result<TokenKind, SyntaxError> {
        while let Some(c) = self.cursor.peek() {
            if is_ident_continue(c) {
                self.cursor.advance();
            } else if c == b'\\' && self.cursor.peek_next() == Some(b'u') {
                self.consume_unicode_escape()?;
            } else {
                break;
            }
        }

        let text = self.cursor.slice_from(start);
        keyword_or_ident(&decode_identifier(text, start)?)
    }

    pub(super) fn scan_identifier_after_escape_start(
        &mut self,
        start: usize,
    ) -> Result<TokenKind, SyntaxError> {
        self.consume_unicode_escape_after_backslash(start)?;

        while let Some(c) = self.cursor.peek() {
            if is_ident_continue(c) {
                self.cursor.advance();
            } else if c == b'\\' && self.cursor.peek_next() == Some(b'u') {
                self.consume_unicode_escape()?;
            } else {
                break;
            }
        }

        let text = self.cursor.slice_from(start);
        keyword_or_ident(&decode_identifier(text, start)?)
    }

    fn consume_unicode_escape(&mut self) -> Result<(), SyntaxError> {
        let backslash_offset = self.cursor.pos();
        self.cursor.advance(); // '\\'
        self.consume_unicode_escape_after_backslash_with_offset(backslash_offset)
    }

    fn consume_unicode_escape_after_backslash(
        &mut self,
        ident_start: usize,
    ) -> Result<(), SyntaxError> {
        let backslash_offset = ident_start;
        self.consume_unicode_escape_after_backslash_with_offset(backslash_offset)
    }

    fn consume_unicode_escape_after_backslash_with_offset(
        &mut self,
        backslash_offset: usize,
    ) -> Result<(), SyntaxError> {
        if self.cursor.peek() != Some(b'u') {
            return Err(SyntaxError::new(
                "invalid unicode escape sequence in identifier",
                backslash_offset,
                1,
            ));
        }
        self.cursor.advance(); // 'u'

        if self.cursor.peek() == Some(b'{') {
            self.cursor.advance();
            let mut has_digit = false;
            while let Some(c) = self.cursor.peek() {
                if c.is_ascii_hexdigit() {
                    has_digit = true;
                    self.cursor.advance();
                } else {
                    break;
                }
            }
            if !has_digit || self.cursor.peek() != Some(b'}') {
                return Err(SyntaxError::new(
                    "invalid unicode escape sequence in identifier",
                    backslash_offset,
                    self.cursor.pos().saturating_sub(backslash_offset).max(1),
                ));
            }
            self.cursor.advance();
            return Ok(());
        }

        for _ in 0..4 {
            match self.cursor.peek() {
                Some(c) if c.is_ascii_hexdigit() => {
                    self.cursor.advance();
                }
                _ => {
                    return Err(SyntaxError::new(
                        "invalid unicode escape sequence in identifier",
                        backslash_offset,
                        self.cursor.pos().saturating_sub(backslash_offset).max(1),
                    ));
                }
            }
        }

        Ok(())
    }
}

fn keyword_or_ident(text: &str) -> Result<TokenKind, SyntaxError> {
    Ok(match text {
        "let" => TokenKind::Let,
        "const" => TokenKind::Const,
        "function" => TokenKind::Function,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "return" => TokenKind::Return,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "of" => TokenKind::Of,
        "in" => TokenKind::In,
        "typeof" => TokenKind::Typeof,
        "delete" => TokenKind::Delete,
        "try" => TokenKind::Try,
        "catch" => TokenKind::Catch,
        "finally" => TokenKind::Finally,
        "throw" => TokenKind::Throw,
        "switch" => TokenKind::Switch,
        "case" => TokenKind::Case,
        "default" => TokenKind::Default,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "async" => TokenKind::Async,
        "await" => TokenKind::Await,
        "import" => TokenKind::Import,
        "export" => TokenKind::Export,
        "from" => TokenKind::From,
        "as" => TokenKind::As,
        "new" => TokenKind::New,
        "class" => TokenKind::Class,
        "extends" => TokenKind::Extends,
        "super" => TokenKind::Super,
        "instanceof" => TokenKind::Instanceof,
        "var" => TokenKind::Var,
        "void" => TokenKind::Void,
        "do" => TokenKind::Do,
        "yield" => TokenKind::Yield,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "null" => TokenKind::Null,
        "undefined" => TokenKind::Undefined,
        _ => TokenKind::Ident(text.to_owned()),
    })
}

fn decode_identifier(text: &str, start_offset: usize) -> Result<String, SyntaxError> {
    if !text.as_bytes().contains(&b'\\') {
        return Ok(text.to_owned());
    }

    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'\\' {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }

        let esc_start = i;
        i += 1;
        if i >= bytes.len() || bytes[i] != b'u' {
            return Err(SyntaxError::new(
                "invalid unicode escape sequence in identifier",
                start_offset + esc_start,
                1,
            ));
        }
        i += 1;

        let code_point = if i < bytes.len() && bytes[i] == b'{' {
            i += 1;
            let hex_start = i;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            if i == hex_start || i >= bytes.len() || bytes[i] != b'}' {
                return Err(SyntaxError::new(
                    "invalid unicode escape sequence in identifier",
                    start_offset + esc_start,
                    i.saturating_sub(esc_start).max(1),
                ));
            }
            let hex = &text[hex_start..i];
            i += 1;
            u32::from_str_radix(hex, 16).map_err(|_| {
                SyntaxError::new(
                    "invalid unicode escape sequence in identifier",
                    start_offset + esc_start,
                    i.saturating_sub(esc_start).max(1),
                )
            })?
        } else {
            if i + 4 > bytes.len() {
                return Err(SyntaxError::new(
                    "invalid unicode escape sequence in identifier",
                    start_offset + esc_start,
                    bytes.len().saturating_sub(esc_start).max(1),
                ));
            }
            let hex = &text[i..i + 4];
            i += 4;
            u32::from_str_radix(hex, 16).map_err(|_| {
                SyntaxError::new(
                    "invalid unicode escape sequence in identifier",
                    start_offset + esc_start,
                    i.saturating_sub(esc_start).max(1),
                )
            })?
        };

        let ch = char::from_u32(code_point).ok_or_else(|| {
            SyntaxError::new(
                "invalid unicode escape sequence in identifier",
                start_offset + esc_start,
                i.saturating_sub(esc_start).max(1),
            )
        })?;
        out.push(ch);
    }

    Ok(out)
}

pub(super) fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}

fn is_ident_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

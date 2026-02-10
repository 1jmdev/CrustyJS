use super::scanner::Scanner;
use super::token::TokenKind;

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

    pub(super) fn scan_identifier(&mut self, start: usize) -> TokenKind {
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
            "typeof" => TokenKind::Typeof,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            "throw" => TokenKind::Throw,
            "new" => TokenKind::New,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "undefined" => TokenKind::Undefined,
            _ => TokenKind::Ident(text.to_owned()),
        }
    }
}

pub(super) fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}

fn is_ident_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

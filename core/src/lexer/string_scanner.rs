use super::scanner::Scanner;
use super::token::{Span, Token, TokenKind};
use crate::errors::SyntaxError;

impl Scanner<'_> {
    pub(super) fn scan_string(
        &mut self,
        quote: u8,
        start: usize,
    ) -> Result<TokenKind, SyntaxError> {
        let mut value = String::new();
        loop {
            match self.cursor.advance() {
                Some(c) if c == quote => break,
                Some(b'\\') => match self.cursor.advance() {
                    Some(b'n') => value.push('\n'),
                    Some(b't') => value.push('\t'),
                    Some(b'\\') => value.push('\\'),
                    Some(c) if c == quote => value.push(c as char),
                    Some(c) => {
                        value.push('\\');
                        value.push(c as char);
                    }
                    None => {
                        return Err(SyntaxError::new(
                            "unterminated string literal",
                            start,
                            self.cursor.pos() - start,
                        ));
                    }
                },
                Some(c) => value.push(c as char),
                None => {
                    return Err(SyntaxError::new(
                        "unterminated string literal",
                        start,
                        self.cursor.pos() - start,
                    ));
                }
            }
        }
        Ok(TokenKind::String(value))
    }

    pub(super) fn scan_template(&mut self, start: usize) -> Result<Token, SyntaxError> {
        let (text, ended) = self.scan_template_text(start)?;
        let end = self.cursor.pos();
        let kind = if ended {
            TokenKind::NoSubTemplate(text)
        } else {
            self.template_depth += 1;
            TokenKind::TemplateHead(text)
        };
        Ok(Token {
            kind,
            span: Span::new(start, end),
        })
    }

    pub(super) fn scan_template_continue(&mut self, start: usize) -> Result<Token, SyntaxError> {
        let (text, ended) = self.scan_template_text(start)?;
        let end = self.cursor.pos();
        let kind = if ended {
            self.template_depth -= 1;
            TokenKind::TemplateTail(text)
        } else {
            TokenKind::TemplateMiddle(text)
        };
        Ok(Token {
            kind,
            span: Span::new(start, end),
        })
    }

    fn scan_template_text(&mut self, start: usize) -> Result<(String, bool), SyntaxError> {
        let mut value = String::new();
        loop {
            match self.cursor.advance() {
                Some(b'`') => return Ok((value, true)),
                Some(b'$') if self.cursor.peek() == Some(b'{') => {
                    self.cursor.advance();
                    return Ok((value, false));
                }
                Some(b'\\') => match self.cursor.advance() {
                    Some(b'n') => value.push('\n'),
                    Some(b't') => value.push('\t'),
                    Some(c) => value.push(c as char),
                    None => break,
                },
                Some(c) => value.push(c as char),
                None => break,
            }
        }
        Err(SyntaxError::new(
            "unterminated template literal",
            start,
            self.cursor.pos() - start,
        ))
    }
}

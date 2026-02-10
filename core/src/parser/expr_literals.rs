use super::ast::{Expr, TemplatePart};
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_object_literal(&mut self) -> Result<Expr, SyntaxError> {
        let mut properties = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let key = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expr(0)?;
            properties.push((key, value));
            if !self.check(&TokenKind::RightBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(Expr::ObjectLiteral { properties })
    }

    pub(crate) fn parse_array_literal(&mut self) -> Result<Expr, SyntaxError> {
        let mut elements = Vec::new();
        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            elements.push(self.parse_expr(0)?);
            if !self.check(&TokenKind::RightBracket) {
                self.expect(&TokenKind::Comma)?;
            }
        }
        self.expect(&TokenKind::RightBracket)?;
        Ok(Expr::ArrayLiteral { elements })
    }

    pub(crate) fn parse_template_parts(&mut self, head: String) -> Result<Expr, SyntaxError> {
        let mut parts = Vec::new();
        if !head.is_empty() {
            parts.push(TemplatePart::Str(head));
        }
        loop {
            let expr = self.parse_expr(0)?;
            parts.push(TemplatePart::Expression(expr));
            let tok = self.advance().clone();
            match tok.kind {
                TokenKind::TemplateTail(ref s) => {
                    if !s.is_empty() {
                        parts.push(TemplatePart::Str(s.clone()));
                    }
                    break;
                }
                TokenKind::TemplateMiddle(ref s) => {
                    if !s.is_empty() {
                        parts.push(TemplatePart::Str(s.clone()));
                    }
                }
                _ => {
                    return Err(SyntaxError::new(
                        "expected template continuation",
                        tok.span.start,
                        tok.span.len().max(1),
                    ));
                }
            }
        }
        Ok(Expr::TemplateLiteral { parts })
    }
}

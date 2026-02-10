use super::ast::{ArrowBody, Expr, TemplatePart};
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_ident_or_arrow(&mut self, name: String) -> Result<Expr, SyntaxError> {
        if self.check(&TokenKind::Arrow) {
            self.advance();
            let body = self.parse_arrow_body()?;
            Ok(Expr::ArrowFunction {
                params: vec![name],
                body,
            })
        } else if self.check(&TokenKind::Assign) {
            self.advance();
            let value = self.parse_expr(0)?;
            Ok(Expr::Assign {
                name,
                value: Box::new(value),
            })
        } else {
            Ok(Expr::Identifier(name))
        }
    }

    pub(crate) fn parse_paren_or_arrow(&mut self) -> Result<Expr, SyntaxError> {
        let after_lparen = self.pos;
        if let Some((params, next_pos)) = self.scan_arrow_params(after_lparen) {
            self.pos = next_pos;
            let body = self.parse_arrow_body()?;
            return Ok(Expr::ArrowFunction { params, body });
        }

        self.pos = after_lparen;
        let expr = self.parse_expr(0)?;
        self.expect(&TokenKind::RightParen)?;
        Ok(expr)
    }

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

    fn parse_arrow_body(&mut self) -> Result<ArrowBody, SyntaxError> {
        if self.check(&TokenKind::LeftBrace) {
            Ok(ArrowBody::Block(self.parse_block()?))
        } else {
            Ok(ArrowBody::Expr(Box::new(self.parse_expr(0)?)))
        }
    }

    fn scan_arrow_params(&self, start_pos: usize) -> Option<(Vec<String>, usize)> {
        let mut params = Vec::new();
        let mut i = start_pos;

        if matches!(&self.tokens[i].kind, TokenKind::RightParen) {
            i += 1;
        } else {
            loop {
                match &self.tokens[i].kind {
                    TokenKind::Ident(name) => params.push(name.clone()),
                    _ => return None,
                }
                i += 1;

                if matches!(&self.tokens[i].kind, TokenKind::Comma) {
                    i += 1;
                    continue;
                }
                break;
            }

            if !matches!(&self.tokens[i].kind, TokenKind::RightParen) {
                return None;
            }
            i += 1;
        }

        if !matches!(&self.tokens[i].kind, TokenKind::Arrow) {
            return None;
        }
        i += 1;

        Some((params, i))
    }
}

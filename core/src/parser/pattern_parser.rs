use super::Parser;
use super::ast::{ObjectPatternProp, Param, Pattern};
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_params_list(&mut self) -> Result<Vec<Param>, SyntaxError> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RightParen) {
            return Ok(params);
        }

        loop {
            let pattern = self.parse_pattern()?;
            let default = if self.check(&TokenKind::Assign) {
                self.advance();
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            params.push(Param { pattern, default });

            if self.check(&TokenKind::Comma) {
                self.advance();
                continue;
            }
            break;
        }

        Ok(params)
    }

    pub(crate) fn scan_arrow_signature(&self, start_pos: usize) -> bool {
        let mut i = start_pos;
        let mut paren_depth = 0usize;

        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LeftParen => paren_depth += 1,
                TokenKind::RightParen => {
                    if paren_depth == 0 {
                        return i + 1 < self.tokens.len()
                            && matches!(self.tokens[i + 1].kind, TokenKind::Arrow);
                    }
                    paren_depth -= 1;
                }
                TokenKind::Eof => return false,
                _ => {}
            }
            i += 1;
        }

        false
    }

    pub(crate) fn parse_pattern(&mut self) -> Result<Pattern, SyntaxError> {
        match self.peek() {
            TokenKind::Ident(_) => Ok(Pattern::Identifier(self.expect_ident()?)),
            TokenKind::LeftBrace => self.parse_object_pattern(),
            TokenKind::LeftBracket => self.parse_array_pattern(),
            _ => {
                let token = self.tokens[self.pos].clone();
                Err(SyntaxError::new(
                    format!("expected binding pattern, found {:?}", token.kind),
                    token.span.start,
                    token.span.len().max(1),
                ))
            }
        }
    }

    fn parse_object_pattern(&mut self) -> Result<Pattern, SyntaxError> {
        self.expect(&TokenKind::LeftBrace)?;
        let mut properties = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::DotDotDot) {
                self.advance();
                let rest = Pattern::Rest(Box::new(self.parse_pattern()?));
                properties.push(ObjectPatternProp {
                    key: String::new(),
                    alias: Some(rest),
                    default: None,
                    is_rest: true,
                });
            } else {
                let key = self.expect_ident()?;
                let alias = if self.check(&TokenKind::Colon) {
                    self.advance();
                    Some(self.parse_pattern()?)
                } else {
                    None
                };

                let default = if self.check(&TokenKind::Assign) {
                    self.advance();
                    Some(self.parse_expr(0)?)
                } else {
                    None
                };

                properties.push(ObjectPatternProp {
                    key,
                    alias,
                    default,
                    is_rest: false,
                });
            }

            if !self.check(&TokenKind::RightBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }

        self.expect(&TokenKind::RightBrace)?;
        Ok(Pattern::ObjectPattern { properties })
    }

    fn parse_array_pattern(&mut self) -> Result<Pattern, SyntaxError> {
        self.expect(&TokenKind::LeftBracket)?;
        let mut elements = Vec::new();

        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            if self.check(&TokenKind::Comma) {
                self.advance();
                elements.push(None);
                continue;
            }

            let element = if self.check(&TokenKind::DotDotDot) {
                self.advance();
                Pattern::Rest(Box::new(self.parse_pattern()?))
            } else {
                self.parse_pattern()?
            };
            elements.push(Some(element));

            if !self.check(&TokenKind::RightBracket) {
                self.expect(&TokenKind::Comma)?;
            }
        }

        self.expect(&TokenKind::RightBracket)?;
        Ok(Pattern::ArrayPattern { elements })
    }
}

use super::Parser;
use super::ast::{Stmt, SwitchCase};
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_switch(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'switch'
        self.expect(&TokenKind::LeftParen)?;
        let discriminant = self.parse_expr(0)?;
        self.expect(&TokenKind::RightParen)?;
        self.expect(&TokenKind::LeftBrace)?;

        let mut cases = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Case) {
                self.advance();
                let test = self.parse_expr(0)?;
                self.expect(&TokenKind::Colon)?;
                let body = self.parse_switch_case_body()?;
                cases.push(SwitchCase {
                    test: Some(test),
                    body,
                });
            } else if self.check(&TokenKind::Default) {
                self.advance();
                self.expect(&TokenKind::Colon)?;
                let body = self.parse_switch_case_body()?;
                cases.push(SwitchCase { test: None, body });
            } else {
                let token = self.tokens[self.pos].clone();
                return Err(SyntaxError::new(
                    format!("expected case/default in switch, found {:?}", token.kind),
                    token.span.start,
                    token.span.len().max(1),
                ));
            }
        }

        self.expect(&TokenKind::RightBrace)?;
        Ok(Stmt::Switch {
            discriminant,
            cases,
        })
    }

    fn parse_switch_case_body(&mut self) -> Result<Vec<Stmt>, SyntaxError> {
        let mut body = Vec::new();
        while !self.check(&TokenKind::Case)
            && !self.check(&TokenKind::Default)
            && !self.check(&TokenKind::RightBrace)
            && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
        }
        Ok(body)
    }
}

use super::Parser;
use super::ast::Stmt;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_for(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'for'
        self.expect(&TokenKind::LeftParen)?;

        if matches!(self.peek(), TokenKind::Let | TokenKind::Const) {
            let saved_pos = self.pos;
            self.advance();
            if let TokenKind::Ident(_) = self.peek() {
                let name = self.expect_ident()?;
                if self.check(&TokenKind::Of) || self.check(&TokenKind::In) {
                    let is_for_in = self.check(&TokenKind::In);
                    self.advance();
                    let iterable_or_object = self.parse_expr(0)?;
                    self.expect(&TokenKind::RightParen)?;
                    let body = Box::new(self.parse_statement()?);
                    return if is_for_in {
                        Ok(Stmt::ForIn {
                            variable: name,
                            object: iterable_or_object,
                            body,
                        })
                    } else {
                        Ok(Stmt::ForOf {
                            variable: name,
                            iterable: iterable_or_object,
                            body,
                        })
                    };
                }
                self.pos = saved_pos;
            } else {
                self.pos = saved_pos;
            }
        }

        let init = if self.check(&TokenKind::Semicolon) {
            self.advance();
            None
        } else {
            let stmt = if matches!(self.peek(), TokenKind::Let | TokenKind::Const) {
                self.parse_var_decl()?
            } else {
                self.parse_expr_stmt()?
            };
            Some(Box::new(stmt))
        };

        let condition = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr(0)?)
        };
        self.expect(&TokenKind::Semicolon)?;

        let update = if self.check(&TokenKind::RightParen) {
            None
        } else {
            Some(self.parse_expr(0)?)
        };
        self.expect(&TokenKind::RightParen)?;

        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::ForLoop {
            init,
            condition,
            update,
            body,
        })
    }
}

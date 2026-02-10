use super::ast::Stmt;
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_statement(&mut self) -> Result<Stmt, SyntaxError> {
        match self.peek() {
            TokenKind::Let | TokenKind::Const => self.parse_var_decl(),
            TokenKind::Function => self.parse_function_decl(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Try => self.parse_try_catch(),
            TokenKind::Throw => self.parse_throw(),
            TokenKind::Class => self.parse_class_decl(),
            TokenKind::LeftBrace => self.parse_block_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_var_decl(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'let' or 'const'
        let name = self.expect_ident()?;
        let init = if self.check(&TokenKind::Assign) {
            self.advance(); // consume '='
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        self.consume_stmt_terminator()?;
        Ok(Stmt::VarDecl { name, init })
    }

    fn parse_function_decl(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'function'
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LeftParen)?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            params.push(self.expect_ident()?);
            while self.check(&TokenKind::Comma) {
                self.advance(); // consume ','
                params.push(self.expect_ident()?);
            }
        }
        self.expect(&TokenKind::RightParen)?;

        let body = self.parse_block()?;
        Ok(Stmt::FunctionDecl { name, params, body })
    }

    fn parse_if(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'if'
        self.expect(&TokenKind::LeftParen)?;
        let condition = self.parse_expr(0)?;
        self.expect(&TokenKind::RightParen)?;

        let then_branch = Box::new(self.parse_statement()?);

        let else_branch = if self.check(&TokenKind::Else) {
            self.advance(); // consume 'else'
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'while'
        self.expect(&TokenKind::LeftParen)?;
        let condition = self.parse_expr(0)?;
        self.expect(&TokenKind::RightParen)?;
        let body = Box::new(self.parse_statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn parse_return(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'return'
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            return Ok(Stmt::Return(None));
        }
        let value = self.parse_expr(0)?;
        self.consume_stmt_terminator()?;
        Ok(Stmt::Return(Some(value)))
    }

    fn parse_block_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        Ok(Stmt::Block(self.parse_block()?))
    }

    pub(crate) fn parse_block(&mut self) -> Result<Vec<Stmt>, SyntaxError> {
        self.expect(&TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(stmts)
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, SyntaxError> {
        let expr = self.parse_expr(0)?;
        self.consume_stmt_terminator()?;
        Ok(Stmt::ExprStmt(expr))
    }

    fn parse_for(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'for'
        self.expect(&TokenKind::LeftParen)?;

        // Check for `for (let x of iterable)`
        if matches!(self.peek(), TokenKind::Let | TokenKind::Const) {
            let saved_pos = self.pos;
            self.advance(); // consume let/const
            if let TokenKind::Ident(_) = self.peek() {
                let name = self.expect_ident()?;
                if self.check(&TokenKind::Of) {
                    self.advance(); // consume 'of'
                    let iterable = self.parse_expr(0)?;
                    self.expect(&TokenKind::RightParen)?;
                    let body = Box::new(self.parse_statement()?);
                    return Ok(Stmt::ForOf {
                        variable: name,
                        iterable,
                        body,
                    });
                }
                // Not for-of, rewind and parse as regular for
                self.pos = saved_pos;
            } else {
                self.pos = saved_pos;
            }
        }

        // Regular for loop: for (init; cond; update)
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

    fn parse_try_catch(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'try'
        let try_block = self.parse_block()?;

        let mut catch_param = None;
        let mut catch_block = None;
        let mut finally_block = None;

        if self.check(&TokenKind::Catch) {
            self.advance(); // consume 'catch'
            if self.check(&TokenKind::LeftParen) {
                self.advance();
                catch_param = Some(self.expect_ident()?);
                self.expect(&TokenKind::RightParen)?;
            }
            catch_block = Some(self.parse_block()?);
        }

        if self.check(&TokenKind::Finally) {
            self.advance(); // consume 'finally'
            finally_block = Some(self.parse_block()?);
        }

        if catch_block.is_none() && finally_block.is_none() {
            let token = self.tokens[self.pos].clone();
            return Err(SyntaxError::new(
                "try requires catch and/or finally",
                token.span.start,
                token.span.len().max(1),
            ));
        }

        Ok(Stmt::TryCatch {
            try_block,
            catch_param,
            catch_block,
            finally_block,
        })
    }

    fn parse_throw(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'throw'
        let expr = self.parse_expr(0)?;
        self.consume_stmt_terminator()?;
        Ok(Stmt::Throw(expr))
    }

    fn consume_stmt_terminator(&mut self) -> Result<(), SyntaxError> {
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            return Ok(());
        }
        if self.check(&TokenKind::RightBrace) || self.check(&TokenKind::Eof) {
            return Ok(());
        }
        let token = self.tokens[self.pos].clone();
        Err(SyntaxError::new(
            format!("expected Semicolon, found {:?}", token.kind),
            token.span.start,
            token.span.len().max(1),
        ))
    }
}

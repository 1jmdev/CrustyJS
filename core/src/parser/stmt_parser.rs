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
            TokenKind::Return => self.parse_return(),
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
        self.expect(&TokenKind::Semicolon)?;
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
        self.expect(&TokenKind::Semicolon)?;
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
        self.expect(&TokenKind::Semicolon)?;
        Ok(Stmt::ExprStmt(expr))
    }
}

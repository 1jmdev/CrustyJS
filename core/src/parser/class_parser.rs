use super::ast::{ClassDecl, ClassMethod, Stmt};
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_class_decl(&mut self) -> Result<Stmt, SyntaxError> {
        self.advance(); // consume 'class'
        let name = self.expect_ident()?;

        let parent = if self.check(&TokenKind::Extends) {
            self.advance(); // consume 'extends'
            Some(self.expect_ident()?)
        } else {
            None
        };

        self.expect(&TokenKind::LeftBrace)?;
        let mut constructor = None;
        let mut methods = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let method_name = self.expect_ident()?;
            self.expect(&TokenKind::LeftParen)?;

            let mut params = Vec::new();
            if !self.check(&TokenKind::RightParen) {
                params.push(self.expect_ident()?);
                while self.check(&TokenKind::Comma) {
                    self.advance();
                    params.push(self.expect_ident()?);
                }
            }
            self.expect(&TokenKind::RightParen)?;
            let body = self.parse_block()?;

            let method = ClassMethod {
                name: method_name.clone(),
                params,
                body,
                is_static: false,
            };

            if method_name == "constructor" {
                constructor = Some(method);
            } else {
                methods.push(method);
            }
        }

        self.expect(&TokenKind::RightBrace)?;
        Ok(Stmt::Class(ClassDecl {
            name,
            parent,
            constructor,
            methods,
        }))
    }
}

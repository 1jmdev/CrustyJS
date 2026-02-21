use super::ast::{ClassDecl, ClassMethod, ClassMethodKind, Stmt};
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
            let mut method_kind = ClassMethodKind::Method;
            let mut method_name = self.expect_ident()?;

            if (method_name == "get" || method_name == "set") && !self.check(&TokenKind::LeftParen)
            {
                method_kind = if method_name == "get" {
                    ClassMethodKind::Getter
                } else {
                    ClassMethodKind::Setter
                };
                method_name = self.expect_ident()?;
            }

            self.expect(&TokenKind::LeftParen)?;

            let mut params = Vec::new();
            if !self.check(&TokenKind::RightParen) {
                params.push(self.expect_ident()?);
                while self.check(&TokenKind::Comma) {
                    self.advance();
                    if self.check(&TokenKind::RightParen) {
                        break;
                    }
                    params.push(self.expect_ident()?);
                }
            }
            self.expect(&TokenKind::RightParen)?;
            let body = self.parse_block()?;

            if method_kind == ClassMethodKind::Getter && !params.is_empty() {
                return Err(SyntaxError::new(
                    "getter must not declare parameters",
                    self.tokens[self.pos - 1].span.start,
                    self.tokens[self.pos - 1].span.len().max(1),
                ));
            }

            if method_kind == ClassMethodKind::Setter && params.len() != 1 {
                return Err(SyntaxError::new(
                    "setter must declare exactly one parameter",
                    self.tokens[self.pos - 1].span.start,
                    self.tokens[self.pos - 1].span.len().max(1),
                ));
            }

            let method = ClassMethod {
                name: method_name.clone(),
                params,
                body,
                is_static: false,
                kind: method_kind,
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

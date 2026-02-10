use super::ast::{ExportDecl, ExportSpecifier, ImportDecl, ImportSpecifier, Stmt};
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_import_decl(&mut self) -> Result<Stmt, SyntaxError> {
        self.expect(&TokenKind::Import)?;

        let mut specifiers = Vec::new();
        if let TokenKind::Ident(name) = self.peek().clone() {
            self.advance();
            specifiers.push(ImportSpecifier::Default(name));
            if self.check(&TokenKind::Comma) {
                self.advance();
            }
        }

        if self.check(&TokenKind::Star) {
            self.advance();
            self.expect(&TokenKind::As)?;
            let name = self.expect_ident()?;
            specifiers.push(ImportSpecifier::Namespace(name));
        } else if self.check(&TokenKind::LeftBrace) {
            self.advance();
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                let imported = self.expect_ident()?;
                let local = if self.check(&TokenKind::As) {
                    self.advance();
                    self.expect_ident()?
                } else {
                    imported.clone()
                };
                specifiers.push(ImportSpecifier::Named { imported, local });
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RightBrace)?;
        }

        self.expect(&TokenKind::From)?;
        let source = match self.advance().kind.clone() {
            TokenKind::String(s) => s,
            other => {
                return Err(SyntaxError::new(
                    format!("expected import source string, found {other:?}"),
                    self.tokens[self.pos - 1].span.start,
                    self.tokens[self.pos - 1].span.len().max(1),
                ))
            }
        };
        self.consume_stmt_terminator()?;
        Ok(Stmt::Import(ImportDecl { specifiers, source }))
    }

    pub(crate) fn parse_export_decl(&mut self) -> Result<Stmt, SyntaxError> {
        self.expect(&TokenKind::Export)?;
        if self.check(&TokenKind::Default) {
            self.advance();
            let expr = self.parse_expr(0)?;
            self.consume_stmt_terminator()?;
            return Ok(Stmt::Export(ExportDecl::Default(expr)));
        }

        if self.check(&TokenKind::Function) {
            let stmt = self.parse_function_decl()?;
            return Ok(Stmt::Export(ExportDecl::NamedStmt(Box::new(stmt))));
        }
        if self.check(&TokenKind::Async) {
            self.advance();
            if self.check(&TokenKind::Function) {
                let stmt = self.parse_function_decl_with_async(true)?;
                return Ok(Stmt::Export(ExportDecl::NamedStmt(Box::new(stmt))));
            }
            let token = self.tokens[self.pos].clone();
            return Err(SyntaxError::new(
                "expected function after export async",
                token.span.start,
                token.span.len().max(1),
            ));
        }
        if self.check(&TokenKind::Const) || self.check(&TokenKind::Let) {
            let stmt = self.parse_var_decl()?;
            return Ok(Stmt::Export(ExportDecl::NamedStmt(Box::new(stmt))));
        }
        if self.check(&TokenKind::LeftBrace) {
            self.advance();
            let mut specifiers = Vec::new();
            while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                let local = self.expect_ident()?;
                let exported = if self.check(&TokenKind::As) {
                    self.advance();
                    self.expect_ident()?
                } else {
                    local.clone()
                };
                specifiers.push(ExportSpecifier { local, exported });
                if self.check(&TokenKind::Comma) {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RightBrace)?;
            self.consume_stmt_terminator()?;
            return Ok(Stmt::Export(ExportDecl::NamedList(specifiers)));
        }

        let token = self.tokens[self.pos].clone();
        Err(SyntaxError::new(
            "unsupported export declaration",
            token.span.start,
            token.span.len().max(1),
        ))
    }
}

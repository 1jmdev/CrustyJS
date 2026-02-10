pub mod ast;
mod expr_literals;
mod expr_ops;
mod expr_parser;
mod stmt_parser;

use crate::errors::SyntaxError;
use crate::lexer::token::{Token, TokenKind};
use ast::Program;

/// Parse a token stream into a Program AST.
pub fn parse(tokens: Vec<Token>) -> Result<Program, SyntaxError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

/// Recursive-descent parser over a token stream.
pub(crate) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_program(&mut self) -> Result<Program, SyntaxError> {
        let mut body = Vec::new();
        while !self.is_at_end() {
            body.push(self.parse_statement()?);
        }
        Ok(Program { body })
    }

    pub(crate) fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    pub(crate) fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos];
        if !self.is_at_end() {
            self.pos += 1;
        }
        token
    }

    pub(crate) fn expect(&mut self, expected: &TokenKind) -> Result<&Token, SyntaxError> {
        if self.peek() == expected {
            Ok(self.advance())
        } else {
            let token = &self.tokens[self.pos];
            Err(SyntaxError::new(
                format!("expected {:?}, found {:?}", expected, token.kind),
                token.span.start,
                token.span.len().max(1),
            ))
        }
    }

    pub(crate) fn is_at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    pub(crate) fn check(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }
}

use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn consume_stmt_terminator(&mut self) -> Result<(), SyntaxError> {
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            return Ok(());
        }
        if self.check(&TokenKind::RightBrace) || self.check(&TokenKind::Eof) {
            return Ok(());
        }
        if self.has_line_terminator_before_current() {
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

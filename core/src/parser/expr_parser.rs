use super::ast::{BinOp, Expr, Literal, TemplatePart, UnaryOp};
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::EqEqEq | TokenKind::NotEqEq => Some((3, 4)),
        TokenKind::Less | TokenKind::LessEq | TokenKind::Greater | TokenKind::GreaterEq => {
            Some((5, 6))
        }
        TokenKind::Plus | TokenKind::Minus => Some((7, 8)),
        TokenKind::Star | TokenKind::Slash => Some((9, 10)),
        _ => None,
    }
}

fn prefix_binding_power(kind: &TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Minus | TokenKind::Bang => Some(11),
        _ => None,
    }
}

fn token_to_binop(kind: &TokenKind) -> BinOp {
    match kind {
        TokenKind::Plus => BinOp::Add,
        TokenKind::Minus => BinOp::Sub,
        TokenKind::Star => BinOp::Mul,
        TokenKind::Slash => BinOp::Div,
        TokenKind::EqEqEq => BinOp::EqEqEq,
        TokenKind::NotEqEq => BinOp::NotEqEq,
        TokenKind::Less => BinOp::Less,
        TokenKind::LessEq => BinOp::LessEq,
        TokenKind::Greater => BinOp::Greater,
        TokenKind::GreaterEq => BinOp::GreaterEq,
        _ => unreachable!("not a binary operator: {:?}", kind),
    }
}

impl Parser {
    pub(crate) fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, SyntaxError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            lhs = match self.peek() {
                TokenKind::LeftParen => {
                    self.advance();
                    let args = self.parse_call_args()?;
                    self.expect(&TokenKind::RightParen)?;
                    Expr::Call {
                        callee: Box::new(lhs),
                        args,
                    }
                }
                TokenKind::Dot => {
                    self.advance();
                    let property = self.expect_ident()?;
                    if self.check(&TokenKind::Assign) {
                        self.advance();
                        let value = self.parse_expr(0)?;
                        Expr::MemberAssign {
                            object: Box::new(lhs),
                            property: Box::new(Expr::Literal(Literal::String(property))),
                            value: Box::new(value),
                        }
                    } else {
                        Expr::MemberAccess {
                            object: Box::new(lhs),
                            property,
                        }
                    }
                }
                TokenKind::LeftBracket => {
                    self.advance();
                    let prop_expr = self.parse_expr(0)?;
                    self.expect(&TokenKind::RightBracket)?;
                    if self.check(&TokenKind::Assign) {
                        self.advance();
                        let value = self.parse_expr(0)?;
                        Expr::MemberAssign {
                            object: Box::new(lhs),
                            property: Box::new(prop_expr),
                            value: Box::new(value),
                        }
                    } else {
                        Expr::ComputedMemberAccess {
                            object: Box::new(lhs),
                            property: Box::new(prop_expr),
                        }
                    }
                }
                _ => break,
            };
        }

        loop {
            let Some((l_bp, r_bp)) = infix_binding_power(self.peek()) else {
                break;
            };
            if l_bp < min_bp {
                break;
            }

            let op_token = self.advance().kind.clone();
            let op = token_to_binop(&op_token);
            let rhs = self.parse_expr(r_bp)?;

            lhs = Expr::Binary {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, SyntaxError> {
        if let Some(rbp) = prefix_binding_power(self.peek()) {
            let op_kind = self.advance().kind.clone();
            let op = match op_kind {
                TokenKind::Minus => UnaryOp::Neg,
                TokenKind::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            let operand = self.parse_expr(rbp)?;
            return Ok(Expr::Unary {
                op,
                operand: Box::new(operand),
            });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, SyntaxError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Number(n) => Ok(Expr::Literal(Literal::Number(n))),
            TokenKind::String(ref s) => Ok(Expr::Literal(Literal::String(s.clone()))),
            TokenKind::True => Ok(Expr::Literal(Literal::Boolean(true))),
            TokenKind::False => Ok(Expr::Literal(Literal::Boolean(false))),
            TokenKind::Null => Ok(Expr::Literal(Literal::Null)),
            TokenKind::Undefined => Ok(Expr::Literal(Literal::Undefined)),
            TokenKind::Ident(ref name) => {
                let name = name.clone();
                if self.check(&TokenKind::Assign) {
                    self.advance();
                    let value = self.parse_expr(0)?;
                    Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            TokenKind::LeftParen => {
                let expr = self.parse_expr(0)?;
                self.expect(&TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::LeftBrace => self.parse_object_literal(),
            TokenKind::LeftBracket => self.parse_array_literal(),
            TokenKind::NoSubTemplate(ref s) => Ok(Expr::Literal(Literal::String(s.clone()))),
            TokenKind::TemplateHead(ref s) => {
                let head = s.clone();
                self.parse_template_parts(head)
            }
            _ => Err(SyntaxError::new(
                format!("unexpected token {:?} in expression", token.kind),
                token.span.start,
                token.span.len().max(1),
            )),
        }
    }

    fn parse_call_args(&mut self) -> Result<Vec<Expr>, SyntaxError> {
        let mut args = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            args.push(self.parse_expr(0)?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                args.push(self.parse_expr(0)?);
            }
        }
        Ok(args)
    }

    fn parse_object_literal(&mut self) -> Result<Expr, SyntaxError> {
        let mut properties = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let key = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let value = self.parse_expr(0)?;
            properties.push((key, value));
            if !self.check(&TokenKind::RightBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(Expr::ObjectLiteral { properties })
    }

    fn parse_array_literal(&mut self) -> Result<Expr, SyntaxError> {
        let mut elements = Vec::new();
        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            elements.push(self.parse_expr(0)?);
            if !self.check(&TokenKind::RightBracket) {
                self.expect(&TokenKind::Comma)?;
            }
        }
        self.expect(&TokenKind::RightBracket)?;
        Ok(Expr::ArrayLiteral { elements })
    }

    pub(crate) fn expect_ident(&mut self) -> Result<String, SyntaxError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Ident(name) => Ok(name),
            _ => Err(SyntaxError::new(
                format!("expected identifier, found {:?}", token.kind),
                token.span.start,
                token.span.len().max(1),
            )),
        }
    }

    fn parse_template_parts(&mut self, head: String) -> Result<Expr, SyntaxError> {
        let mut parts = Vec::new();
        if !head.is_empty() {
            parts.push(TemplatePart::Str(head));
        }
        loop {
            let expr = self.parse_expr(0)?;
            parts.push(TemplatePart::Expression(expr));
            let tok = self.advance().clone();
            match tok.kind {
                TokenKind::TemplateTail(ref s) => {
                    if !s.is_empty() {
                        parts.push(TemplatePart::Str(s.clone()));
                    }
                    break;
                }
                TokenKind::TemplateMiddle(ref s) => {
                    if !s.is_empty() {
                        parts.push(TemplatePart::Str(s.clone()));
                    }
                }
                _ => {
                    return Err(SyntaxError::new(
                        "expected template continuation",
                        tok.span.start,
                        tok.span.len().max(1),
                    ));
                }
            }
        }
        Ok(Expr::TemplateLiteral { parts })
    }
}

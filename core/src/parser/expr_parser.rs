use super::ast::{Expr, Literal, UnaryOp, UpdateOp};
use super::expr_ops::{
    infix_binding_power, prefix_binding_power, token_to_binop, token_to_logical_op,
};
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

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
                TokenKind::PlusPlus => {
                    self.advance();
                    match lhs {
                        Expr::Identifier(name) => Expr::UpdateExpr {
                            name,
                            op: UpdateOp::Inc,
                            prefix: false,
                        },
                        _ => {
                            return Err(SyntaxError::new(
                                "invalid postfix increment target",
                                self.tokens[self.pos - 1].span.start,
                                2,
                            ))
                        }
                    }
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    match lhs {
                        Expr::Identifier(name) => Expr::UpdateExpr {
                            name,
                            op: UpdateOp::Dec,
                            prefix: false,
                        },
                        _ => {
                            return Err(SyntaxError::new(
                                "invalid postfix decrement target",
                                self.tokens[self.pos - 1].span.start,
                                2,
                            ))
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
            let rhs = self.parse_expr(r_bp)?;

            lhs = match op_token {
                TokenKind::AmpAmp | TokenKind::PipePipe | TokenKind::NullishCoalescing => {
                    Expr::Logical {
                        left: Box::new(lhs),
                        op: token_to_logical_op(&op_token),
                        right: Box::new(rhs),
                    }
                }
                _ => Expr::Binary {
                    left: Box::new(lhs),
                    op: token_to_binop(&op_token),
                    right: Box::new(rhs),
                },
            };
        }

        if min_bp == 0 && self.check(&TokenKind::Question) {
            self.advance();
            let then_expr = self.parse_expr(0)?;
            self.expect(&TokenKind::Colon)?;
            let else_expr = self.parse_expr(0)?;
            lhs = Expr::Ternary {
                condition: Box::new(lhs),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, SyntaxError> {
        if self.check(&TokenKind::Typeof) {
            self.advance();
            let operand = self.parse_expr(12)?;
            return Ok(Expr::Typeof(Box::new(operand)));
        }

        if matches!(self.peek(), TokenKind::PlusPlus | TokenKind::MinusMinus) {
            let op_tok = self.advance().clone();
            let ident_tok = self.advance().clone();
            let name = match ident_tok.kind {
                TokenKind::Ident(name) => name,
                _ => {
                    return Err(SyntaxError::new(
                        "expected identifier after update operator",
                        ident_tok.span.start,
                        ident_tok.span.len().max(1),
                    ))
                }
            };
            let op = match op_tok.kind {
                TokenKind::PlusPlus => UpdateOp::Inc,
                TokenKind::MinusMinus => UpdateOp::Dec,
                _ => unreachable!(),
            };
            return Ok(Expr::UpdateExpr {
                name,
                op,
                prefix: true,
            });
        }

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
                self.parse_ident_or_arrow(name)
            }
            TokenKind::LeftParen => self.parse_paren_or_arrow(),
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
}

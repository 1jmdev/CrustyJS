use super::ast::{
    ArrowBody, AssignOp, Expr, ObjectProperty, Param, Pattern, PropertyKey, TemplatePart,
};
use super::Parser;
use crate::errors::SyntaxError;
use crate::lexer::token::TokenKind;

impl Parser {
    pub(crate) fn parse_super_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.advance(); // consume 'super'
        self.expect(&TokenKind::LeftParen)?;
        let args = self.parse_call_args()?;
        self.expect(&TokenKind::RightParen)?;
        Ok(Expr::SuperCall { args })
    }

    pub(crate) fn parse_new_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.advance(); // consume 'new'
        let callee_expr = self.parse_expr(12)?;
        match callee_expr {
            Expr::Call { callee, args } => Ok(Expr::New { callee, args }),
            callee => Ok(Expr::New {
                callee: Box::new(callee),
                args: Vec::new(),
            }),
        }
    }

    pub(crate) fn parse_ident_or_arrow(&mut self, name: String) -> Result<Expr, SyntaxError> {
        if self.check(&TokenKind::Arrow) {
            self.advance();
            let body = self.parse_arrow_body()?;
            Ok(Expr::ArrowFunction {
                params: vec![Param {
                    pattern: Pattern::Identifier(name),
                    default: None,
                }],
                body,
                is_async: false,
            })
        } else if self.check(&TokenKind::PlusEquals)
            || self.check(&TokenKind::MinusEquals)
            || self.check(&TokenKind::StarEquals)
            || self.check(&TokenKind::SlashEquals)
            || self.check(&TokenKind::PercentEquals)
        {
            let op_token = self.advance().kind.clone();
            let op = match op_token {
                TokenKind::PlusEquals => AssignOp::Add,
                TokenKind::MinusEquals => AssignOp::Sub,
                TokenKind::StarEquals => AssignOp::Mul,
                TokenKind::SlashEquals => AssignOp::Div,
                TokenKind::PercentEquals => AssignOp::Mod,
                _ => unreachable!(),
            };
            let value = self.parse_expr(0)?;
            Ok(Expr::CompoundAssign {
                name,
                op,
                value: Box::new(value),
            })
        } else if self.check(&TokenKind::Assign) {
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

    pub(crate) fn parse_paren_or_arrow(&mut self) -> Result<Expr, SyntaxError> {
        let after_lparen = self.pos;
        if self.scan_arrow_signature(after_lparen) {
            let params = self.parse_params_list()?;
            self.expect(&TokenKind::RightParen)?;
            self.expect(&TokenKind::Arrow)?;
            let body = self.parse_arrow_body()?;
            return Ok(Expr::ArrowFunction {
                params,
                body,
                is_async: false,
            });
        }

        self.pos = after_lparen;
        let expr = self.parse_expr(0)?;
        self.expect(&TokenKind::RightParen)?;
        Ok(expr)
    }

    pub(crate) fn parse_object_literal(&mut self) -> Result<Expr, SyntaxError> {
        let mut properties = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::DotDotDot) {
                self.advance();
                properties.push(ObjectProperty::Spread(self.parse_expr(0)?));
                if !self.check(&TokenKind::RightBrace) {
                    self.expect(&TokenKind::Comma)?;
                }
                continue;
            }

            let (key, value) = if self.check(&TokenKind::LeftBracket) {
                self.advance();
                let key_expr = self.parse_expr(0)?;
                self.expect(&TokenKind::RightBracket)?;
                self.expect(&TokenKind::Colon)?;
                (PropertyKey::Computed(key_expr), self.parse_expr(0)?)
            } else {
                let key_name = self.expect_ident()?;
                if (key_name == "get" || key_name == "set") && !self.check(&TokenKind::Colon) {
                    let accessor_key = PropertyKey::Identifier(self.expect_ident()?);
                    self.expect(&TokenKind::LeftParen)?;
                    let params = self.parse_method_params()?;
                    self.expect(&TokenKind::RightParen)?;
                    let body = self.parse_block()?;

                    let accessor = if key_name == "get" {
                        if !params.is_empty() {
                            let token = self.tokens[self.pos - 1].clone();
                            return Err(SyntaxError::new(
                                "getter must not declare parameters",
                                token.span.start,
                                token.span.len().max(1),
                            ));
                        }
                        ObjectProperty::Getter(accessor_key, body)
                    } else {
                        if params.len() != 1 {
                            let token = self.tokens[self.pos - 1].clone();
                            return Err(SyntaxError::new(
                                "setter must declare exactly one parameter",
                                token.span.start,
                                token.span.len().max(1),
                            ));
                        }
                        ObjectProperty::Setter(accessor_key, params[0].clone(), body)
                    };

                    properties.push(accessor);
                    if !self.check(&TokenKind::RightBrace) {
                        self.expect(&TokenKind::Comma)?;
                    }
                    continue;
                }

                if self.check(&TokenKind::Colon) {
                    self.advance();
                    (PropertyKey::Identifier(key_name), self.parse_expr(0)?)
                } else if self.check(&TokenKind::LeftParen) {
                    self.advance();
                    let params = self.parse_method_params()?;
                    self.expect(&TokenKind::RightParen)?;
                    let body = self.parse_block()?;
                    let params = params
                        .into_iter()
                        .map(|name| Param {
                            pattern: Pattern::Identifier(name),
                            default: None,
                        })
                        .collect();
                    (
                        PropertyKey::Identifier(key_name),
                        Expr::ArrowFunction {
                            params,
                            body: ArrowBody::Block(body),
                            is_async: false,
                        },
                    )
                } else {
                    (
                        PropertyKey::Identifier(key_name.clone()),
                        Expr::Identifier(key_name),
                    )
                }
            };
            properties.push(ObjectProperty::KeyValue(key, value));
            if !self.check(&TokenKind::RightBrace) {
                self.expect(&TokenKind::Comma)?;
            }
        }
        self.expect(&TokenKind::RightBrace)?;
        Ok(Expr::ObjectLiteral { properties })
    }

    pub(crate) fn parse_array_literal(&mut self) -> Result<Expr, SyntaxError> {
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

    pub(crate) fn parse_template_parts(&mut self, head: String) -> Result<Expr, SyntaxError> {
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

    fn parse_arrow_body(&mut self) -> Result<ArrowBody, SyntaxError> {
        if self.check(&TokenKind::LeftBrace) {
            Ok(ArrowBody::Block(self.parse_block()?))
        } else {
            Ok(ArrowBody::Expr(Box::new(self.parse_expr(0)?)))
        }
    }

    fn parse_method_params(&mut self) -> Result<Vec<String>, SyntaxError> {
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            params.push(self.expect_ident()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                params.push(self.expect_ident()?);
            }
        }
        Ok(params)
    }

    pub(crate) fn parse_async_expr(&mut self) -> Result<Expr, SyntaxError> {
        self.advance(); // consume async

        if self.check(&TokenKind::Function) {
            return self.parse_function_expr(true);
        }

        if matches!(self.peek(), TokenKind::Ident(_)) {
            let name = self.expect_ident()?;
            if self.check(&TokenKind::Arrow) {
                self.advance();
                let body = self.parse_arrow_body()?;
                return Ok(Expr::ArrowFunction {
                    params: vec![Param {
                        pattern: Pattern::Identifier(name),
                        default: None,
                    }],
                    body,
                    is_async: true,
                });
            }
            return Err(SyntaxError::new(
                "unexpected identifier after async",
                self.tokens[self.pos - 1].span.start,
                self.tokens[self.pos - 1].span.len().max(1),
            ));
        }

        if self.check(&TokenKind::LeftParen) {
            self.advance();
            let after_lparen = self.pos;
            if self.scan_arrow_signature(after_lparen) {
                let params = self.parse_params_list()?;
                self.expect(&TokenKind::RightParen)?;
                self.expect(&TokenKind::Arrow)?;
                let body = self.parse_arrow_body()?;
                return Ok(Expr::ArrowFunction {
                    params,
                    body,
                    is_async: true,
                });
            }
            return Err(SyntaxError::new(
                "async expression must be an async arrow function",
                self.tokens[self.pos - 1].span.start,
                self.tokens[self.pos - 1].span.len().max(1),
            ));
        }

        Err(SyntaxError::new(
            "unexpected token after async",
            self.tokens[self.pos - 1].span.start,
            self.tokens[self.pos - 1].span.len().max(1),
        ))
    }

    pub(crate) fn parse_function_expr(&mut self, is_async: bool) -> Result<Expr, SyntaxError> {
        self.advance(); // consume 'function'
        let is_generator = self.check(&TokenKind::Star);
        if is_generator {
            self.advance();
        }
        let name = if matches!(self.peek(), TokenKind::Ident(_)) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect(&TokenKind::LeftParen)?;
        let params = self.parse_params_list()?;
        self.expect(&TokenKind::RightParen)?;
        let body = self.parse_block()?;
        Ok(Expr::FunctionExpr {
            name,
            params,
            body,
            is_async,
            is_generator,
        })
    }
}

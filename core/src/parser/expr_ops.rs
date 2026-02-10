use super::ast::{BinOp, LogicalOp};
use crate::lexer::token::TokenKind;

pub(super) fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::PipePipe | TokenKind::NullishCoalescing => Some((1, 2)),
        TokenKind::AmpAmp => Some((2, 3)),
        TokenKind::EqEq | TokenKind::NotEq | TokenKind::EqEqEq | TokenKind::NotEqEq => Some((4, 5)),
        TokenKind::Less | TokenKind::LessEq | TokenKind::Greater | TokenKind::GreaterEq => {
            Some((6, 7))
        }
        TokenKind::Instanceof => Some((6, 7)),
        TokenKind::Plus | TokenKind::Minus => Some((8, 9)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((10, 11)),
        _ => None,
    }
}

pub(super) fn prefix_binding_power(kind: &TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Minus | TokenKind::Bang => Some(12),
        _ => None,
    }
}

pub(super) fn token_to_logical_op(kind: &TokenKind) -> LogicalOp {
    match kind {
        TokenKind::AmpAmp => LogicalOp::And,
        TokenKind::PipePipe => LogicalOp::Or,
        TokenKind::NullishCoalescing => LogicalOp::Nullish,
        _ => unreachable!("not a logical operator: {:?}", kind),
    }
}

pub(super) fn token_to_binop(kind: &TokenKind) -> BinOp {
    match kind {
        TokenKind::Plus => BinOp::Add,
        TokenKind::Minus => BinOp::Sub,
        TokenKind::Star => BinOp::Mul,
        TokenKind::Slash => BinOp::Div,
        TokenKind::Percent => BinOp::Mod,
        TokenKind::EqEqEq => BinOp::EqEqEq,
        TokenKind::NotEqEq => BinOp::NotEqEq,
        TokenKind::EqEq => BinOp::EqEq,
        TokenKind::NotEq => BinOp::NotEq,
        TokenKind::Less => BinOp::Less,
        TokenKind::LessEq => BinOp::LessEq,
        TokenKind::Greater => BinOp::Greater,
        TokenKind::GreaterEq => BinOp::GreaterEq,
        TokenKind::Instanceof => BinOp::Instanceof,
        _ => unreachable!("not a binary operator: {:?}", kind),
    }
}

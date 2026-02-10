mod expression;
mod literal;
mod statement;

pub use expression::{BinOp, Expr, UnaryOp};
pub use literal::Literal;
pub use statement::Stmt;

/// A complete JavaScript program — a list of top-level statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
}

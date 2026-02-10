mod class;
mod expression;
mod literal;
mod pattern;
mod statement;

pub use class::{ClassDecl, ClassMethod};
pub use expression::{
    ArrowBody, AssignOp, BinOp, Expr, LogicalOp, ObjectProperty, TemplatePart, UnaryOp, UpdateOp,
};
pub use literal::Literal;
pub use pattern::{ObjectPatternProp, Param, Pattern};
pub use statement::Stmt;

/// A complete JavaScript program — a list of top-level statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
}

mod class;
mod expression;
mod literal;
mod module;
mod pattern;
mod statement;

pub use class::{ClassDecl, ClassMethod, ClassMethodKind};
pub use expression::{
    ArrowBody, AssignOp, BinOp, Expr, LogicalOp, ObjectProperty, PropertyKey, TemplatePart,
    UnaryOp, UpdateOp,
};
pub use literal::Literal;
pub use module::{ExportDecl, ExportSpecifier, ImportDecl, ImportSpecifier};
pub use pattern::{ObjectPatternProp, Param, Pattern};
pub use statement::{Stmt, SwitchCase, VarDeclKind};

/// A complete JavaScript program — a list of top-level statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
}

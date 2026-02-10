use super::expression::Expr;
use super::statement::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub specifiers: Vec<ImportSpecifier>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportSpecifier {
    Named { imported: String, local: String },
    Default(String),
    Namespace(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportDecl {
    NamedStmt(Box<Stmt>),
    Default(Expr),
    DefaultStmt(Box<Stmt>),
    NamedList(Vec<ExportSpecifier>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportSpecifier {
    pub local: String,
    pub exported: String,
}

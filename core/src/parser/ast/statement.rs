use super::class::ClassDecl;
use super::expression::Expr;
use super::pattern::{Param, Pattern};

/// Statement AST nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    ExprStmt(Expr),
    VarDecl {
        pattern: Pattern,
        init: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    FunctionDecl {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    ForLoop {
        init: Option<Box<Stmt>>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
    },
    ForOf {
        variable: String,
        iterable: Expr,
        body: Box<Stmt>,
    },
    TryCatch {
        try_block: Vec<Stmt>,
        catch_param: Option<String>,
        catch_block: Option<Vec<Stmt>>,
        finally_block: Option<Vec<Stmt>>,
    },
    Throw(Expr),
    Switch {
        discriminant: Expr,
        cases: Vec<SwitchCase>,
    },
    Class(ClassDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub test: Option<Expr>,
    pub body: Vec<Stmt>,
}

use super::literal::Literal;
use super::statement::Stmt;

/// Binary operator kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    EqEqEq,
    NotEqEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

/// Unary operator kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Nullish,
}

/// Expression AST nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        property: String,
    },
    TemplateLiteral {
        parts: Vec<TemplatePart>,
    },
    ObjectLiteral {
        properties: Vec<(String, Expr)>,
    },
    ArrayLiteral {
        elements: Vec<Expr>,
    },
    ComputedMemberAccess {
        object: Box<Expr>,
        property: Box<Expr>,
    },
    MemberAssign {
        object: Box<Expr>,
        property: Box<Expr>,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        op: LogicalOp,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    ArrowFunction {
        params: Vec<String>,
        body: ArrowBody,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Str(String),
    Expression(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrowBody {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
}

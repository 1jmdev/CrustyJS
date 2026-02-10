use super::literal::Literal;

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Str(String),
    Expression(Expr),
}

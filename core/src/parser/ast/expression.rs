use super::literal::Literal;
use super::pattern::Param;
use super::statement::Stmt;

/// Binary operator kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqEqEq,
    NotEqEq,
    EqEq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    Instanceof,
    In,
}

/// Unary operator kinds.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    Void,
    Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Nullish,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateOp {
    Inc,
    Dec,
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
    CompoundAssign {
        name: String,
        op: AssignOp,
        value: Box<Expr>,
    },
    UpdateExpr {
        name: String,
        op: UpdateOp,
        prefix: bool,
    },
    MemberAccess {
        object: Box<Expr>,
        property: String,
    },
    TemplateLiteral {
        parts: Vec<TemplatePart>,
    },
    ObjectLiteral {
        properties: Vec<ObjectProperty>,
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
    Typeof(Box<Expr>),
    Spread(Box<Expr>),
    New {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Await(Box<Expr>),
    Yield {
        value: Option<Box<Expr>>,
        delegate: bool,
    },
    SuperCall {
        args: Vec<Expr>,
    },
    ArrowFunction {
        params: Vec<Param>,
        body: ArrowBody,
        is_async: bool,
    },
    OptionalChain {
        base: Box<Expr>,
        chain: Vec<OptionalOp>,
    },
    RegexLiteral {
        pattern: String,
        flags: String,
    },
    Delete(Box<Expr>),
    FunctionExpr {
        name: Option<String>,
        params: Vec<Param>,
        body: Vec<Stmt>,
        is_async: bool,
        is_generator: bool,
    },
    TaggedTemplate {
        tag: Box<Expr>,
        parts: Vec<TemplatePart>,
    },
    Sequence(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OptionalOp {
    PropertyAccess(String),
    ComputedAccess(Expr),
    Call(Vec<Expr>),
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

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectProperty {
    KeyValue(PropertyKey, Expr),
    Getter(PropertyKey, Vec<Stmt>),
    Setter(PropertyKey, String, Vec<Stmt>),
    Spread(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyKey {
    Identifier(String),
    Computed(Expr),
}

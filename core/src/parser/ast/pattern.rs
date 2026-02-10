use super::expression::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Identifier(String),
    ObjectPattern { properties: Vec<ObjectPatternProp> },
    ArrayPattern { elements: Vec<Option<Pattern>> },
    Rest(Box<Pattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub pattern: Pattern,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPatternProp {
    pub key: String,
    pub alias: Option<Pattern>,
    pub default: Option<Expr>,
    pub is_rest: bool,
}

impl Pattern {
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Pattern::Identifier(name) => Some(name),
            _ => None,
        }
    }
}

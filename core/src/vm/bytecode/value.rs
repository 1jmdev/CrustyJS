use super::Chunk;

#[derive(Debug, Clone)]
pub struct VmFunction {
    pub name: String,
    pub arity: usize,
    pub chunk: Box<Chunk>,
}

#[derive(Debug, Clone)]
pub enum VmValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Box<VmFunction>),
}

impl VmValue {
    pub fn to_number(&self) -> f64 {
        match self {
            VmValue::Number(n) => *n,
            VmValue::Boolean(true) => 1.0,
            VmValue::Boolean(false) | VmValue::Null => 0.0,
            VmValue::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            VmValue::Undefined | VmValue::Function(_) => f64::NAN,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            VmValue::Undefined | VmValue::Null => false,
            VmValue::Boolean(b) => *b,
            VmValue::Number(n) => *n != 0.0 && !n.is_nan(),
            VmValue::String(s) => !s.is_empty(),
            VmValue::Function(_) => true,
        }
    }

    pub fn to_output(&self) -> String {
        match self {
            VmValue::Undefined => "undefined".to_string(),
            VmValue::Null => "null".to_string(),
            VmValue::Boolean(b) => b.to_string(),
            VmValue::Number(n) => {
                if n.fract() == 0.0 {
                    (*n as i64).to_string()
                } else {
                    n.to_string()
                }
            }
            VmValue::String(s) => s.clone(),
            VmValue::Function(f) => format!("[Function: {}]", f.name),
        }
    }
}

mod coercion;
mod display;
pub mod object;
pub mod string_methods;

use std::cell::RefCell;
use std::rc::Rc;

use crate::parser::ast::Stmt;
use object::JsObject;

#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Object(Rc<RefCell<JsObject>>),
}

impl PartialEq for JsValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::Null, JsValue::Null) => true,
            (JsValue::Boolean(a), JsValue::Boolean(b)) => a == b,
            (JsValue::Number(a), JsValue::Number(b)) => a == b,
            (JsValue::String(a), JsValue::String(b)) => a == b,
            (JsValue::Object(a), JsValue::Object(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

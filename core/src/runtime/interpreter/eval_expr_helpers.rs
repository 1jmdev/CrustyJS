use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{AssignOp, BinOp, Literal, PropertyKey, UnaryOp};
use crate::runtime::value::JsValue;
use crate::runtime::value::abstract_equals;
use crate::runtime::value::iterator::get_property_simple;
use crate::runtime::value::symbol;

impl Interpreter {
    pub(crate) fn eval_call_args(
        &mut self,
        args: &[crate::parser::ast::Expr],
    ) -> Result<Vec<JsValue>, RuntimeError> {
        let mut values = Vec::new();
        for arg in args {
            match arg {
                crate::parser::ast::Expr::Spread(inner) => {
                    let spread_val = self.eval_expr(inner)?;
                    values.extend(self.collect_iterable(&spread_val)?);
                }
                other => values.push(self.eval_expr(other)?),
            }
        }
        Ok(values)
    }

    pub(crate) fn eval_property_key(&mut self, key: &PropertyKey) -> Result<String, RuntimeError> {
        match key {
            PropertyKey::Identifier(name) => Ok(name.clone()),
            PropertyKey::Computed(expr) => Ok(self.eval_expr(expr)?.to_js_string()),
        }
    }

    pub(crate) fn collect_iterable(
        &mut self,
        value: &JsValue,
    ) -> Result<Vec<JsValue>, RuntimeError> {
        match value {
            JsValue::Array(arr) => Ok(arr.borrow().elements.clone()),
            JsValue::String(s) => Ok(s
                .chars()
                .map(|ch| JsValue::String(ch.to_string()))
                .collect()),
            JsValue::Object(obj) => {
                let iter_sym = symbol::symbol_iterator();
                let method = obj.borrow().get_symbol(&iter_sym);
                let Some(iter_fn) = method else {
                    return Err(RuntimeError::TypeError {
                        message: "object is not iterable".to_string(),
                    });
                };
                let iterator = self.call_function(&iter_fn, &[])?;
                let mut results = Vec::new();
                loop {
                    let next_fn = get_property_simple(&iterator, "next").ok_or_else(|| {
                        RuntimeError::TypeError {
                            message: "iterator has no next method".to_string(),
                        }
                    })?;
                    let result =
                        self.call_function_with_this(&next_fn, &[], Some(iterator.clone()))?;
                    let done = get_property_simple(&result, "done")
                        .map(|v| v.to_boolean())
                        .unwrap_or(false);
                    if done {
                        break;
                    }
                    let val = get_property_simple(&result, "value").unwrap_or(JsValue::Undefined);
                    results.push(val);
                }
                Ok(results)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("{value} is not iterable"),
            }),
        }
    }
}

pub(crate) fn eval_literal(lit: &Literal) -> JsValue {
    match lit {
        Literal::Number(n) => JsValue::Number(*n),
        Literal::String(s) => JsValue::String(s.clone()),
        Literal::Boolean(b) => JsValue::Boolean(*b),
        Literal::Null => JsValue::Null,
        Literal::Undefined => JsValue::Undefined,
    }
}

pub(crate) fn eval_binary(lhs: JsValue, op: &BinOp, rhs: JsValue) -> Result<JsValue, RuntimeError> {
    if matches!(op, BinOp::Add)
        && (matches!(&lhs, JsValue::String(_)) || matches!(&rhs, JsValue::String(_)))
    {
        let a = lhs.to_js_string();
        let b = rhs.to_js_string();
        return Ok(JsValue::String(format!("{a}{b}")));
    }

    let ln = lhs.to_number();
    let rn = rhs.to_number();

    match op {
        BinOp::Add => Ok(JsValue::Number(ln + rn)),
        BinOp::Sub => Ok(JsValue::Number(ln - rn)),
        BinOp::Mul => Ok(JsValue::Number(ln * rn)),
        BinOp::Div => Ok(JsValue::Number(ln / rn)),
        BinOp::Mod => Ok(JsValue::Number(ln % rn)),
        BinOp::Less => Ok(JsValue::Boolean(ln < rn)),
        BinOp::LessEq => Ok(JsValue::Boolean(ln <= rn)),
        BinOp::Greater => Ok(JsValue::Boolean(ln > rn)),
        BinOp::GreaterEq => Ok(JsValue::Boolean(ln >= rn)),
        BinOp::EqEqEq => Ok(JsValue::Boolean(lhs == rhs)),
        BinOp::NotEqEq => Ok(JsValue::Boolean(lhs != rhs)),
        BinOp::EqEq => Ok(JsValue::Boolean(abstract_equals(&lhs, &rhs))),
        BinOp::NotEq => Ok(JsValue::Boolean(!abstract_equals(&lhs, &rhs))),
        BinOp::Instanceof => unreachable!("instanceof handled before eval_binary"),
    }
}

pub(crate) fn eval_compound(
    lhs: JsValue,
    op: &AssignOp,
    rhs: JsValue,
) -> Result<JsValue, RuntimeError> {
    let bin = match op {
        AssignOp::Add => BinOp::Add,
        AssignOp::Sub => BinOp::Sub,
        AssignOp::Mul => BinOp::Mul,
        AssignOp::Div => BinOp::Div,
        AssignOp::Mod => BinOp::Mod,
    };
    eval_binary(lhs, &bin, rhs)
}

pub(crate) fn eval_unary(op: &UnaryOp, val: JsValue) -> Result<JsValue, RuntimeError> {
    match op {
        UnaryOp::Neg => Ok(JsValue::Number(-val.to_number())),
        UnaryOp::Not => Ok(JsValue::Boolean(!val.to_boolean())),
    }
}

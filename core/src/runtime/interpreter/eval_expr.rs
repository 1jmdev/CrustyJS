use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{BinOp, Expr, Literal, UnaryOp};
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_expr(&mut self, expr: &Expr) -> Result<JsValue, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(eval_literal(lit)),
            Expr::Identifier(name) => self.env.get(name),
            Expr::Binary { left, op, right } => {
                let lhs = self.eval_expr(left)?;
                let rhs = self.eval_expr(right)?;
                eval_binary(lhs, op, rhs)
            }
            Expr::Unary { op, operand } => {
                let val = self.eval_expr(operand)?;
                eval_unary(op, val)
            }
            Expr::Call { callee, args } => self.eval_call(callee, args),
            Expr::Assign { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.set(name, val.clone())?;
                Ok(val)
            }
            Expr::MemberAccess { object, property } => {
                self.eval_member_call(object, property, &[], false)
            }
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        // Special case: member method call (e.g. console.log(...), str.toUpperCase())
        if let Expr::MemberAccess { object, property } = callee {
            return self.eval_member_call(object, property, args, true);
        }

        let func = self.eval_expr(callee)?;
        let arg_values: Vec<JsValue> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<_, _>>()?;

        match func {
            JsValue::Function {
                name: _,
                params,
                body,
            } => {
                if params.len() != arg_values.len() {
                    return Err(RuntimeError::ArityMismatch {
                        expected: params.len(),
                        got: arg_values.len(),
                    });
                }

                self.env.push_scope();
                for (param, value) in params.iter().zip(arg_values) {
                    self.env.define(param.clone(), value);
                }

                let mut result = JsValue::Undefined;
                for stmt in &body {
                    match self.eval_stmt(stmt)? {
                        super::ControlFlow::Return(val) => {
                            result = val;
                            break;
                        }
                        super::ControlFlow::None => {}
                    }
                }

                self.env.pop_scope();
                Ok(result)
            }
            other => Err(RuntimeError::NotAFunction {
                name: format!("{other}"),
            }),
        }
    }
}

fn eval_literal(lit: &Literal) -> JsValue {
    match lit {
        Literal::Number(n) => JsValue::Number(*n),
        Literal::String(s) => JsValue::String(s.clone()),
        Literal::Boolean(b) => JsValue::Boolean(*b),
        Literal::Null => JsValue::Null,
        Literal::Undefined => JsValue::Undefined,
    }
}

fn eval_binary(lhs: JsValue, op: &BinOp, rhs: JsValue) -> Result<JsValue, RuntimeError> {
    // String concatenation: if either side is a string, coerce the other
    if matches!(op, BinOp::Add) {
        if matches!(&lhs, JsValue::String(_)) || matches!(&rhs, JsValue::String(_)) {
            let a = lhs.to_js_string();
            let b = rhs.to_js_string();
            return Ok(JsValue::String(format!("{a}{b}")));
        }
    }

    let ln = lhs.to_number();
    let rn = rhs.to_number();

    match op {
        BinOp::Add => Ok(JsValue::Number(ln + rn)),
        BinOp::Sub => Ok(JsValue::Number(ln - rn)),
        BinOp::Mul => Ok(JsValue::Number(ln * rn)),
        BinOp::Div => Ok(JsValue::Number(ln / rn)),
        BinOp::Less => Ok(JsValue::Boolean(ln < rn)),
        BinOp::LessEq => Ok(JsValue::Boolean(ln <= rn)),
        BinOp::Greater => Ok(JsValue::Boolean(ln > rn)),
        BinOp::GreaterEq => Ok(JsValue::Boolean(ln >= rn)),
        BinOp::EqEqEq => Ok(JsValue::Boolean(lhs == rhs)),
        BinOp::NotEqEq => Ok(JsValue::Boolean(lhs != rhs)),
    }
}

fn eval_unary(op: &UnaryOp, val: JsValue) -> Result<JsValue, RuntimeError> {
    match op {
        UnaryOp::Neg => Ok(JsValue::Number(-val.to_number())),
        UnaryOp::Not => Ok(JsValue::Boolean(!val.to_boolean())),
    }
}

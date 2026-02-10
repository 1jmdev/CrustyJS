use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{
    ArrowBody, AssignOp, BinOp, Expr, Literal, LogicalOp, Stmt, TemplatePart, UnaryOp, UpdateOp,
};
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
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
            Expr::CompoundAssign { name, op, value } => {
                let current = self.env.get(name)?;
                let rhs = self.eval_expr(value)?;
                let next = eval_compound(current, op, rhs)?;
                self.env.set(name, next.clone())?;
                Ok(next)
            }
            Expr::UpdateExpr { name, op, prefix } => {
                let current = self.env.get(name)?;
                let num = current.to_number();
                let next = match op {
                    UpdateOp::Inc => JsValue::Number(num + 1.0),
                    UpdateOp::Dec => JsValue::Number(num - 1.0),
                };
                self.env.set(name, next.clone())?;
                if *prefix {
                    Ok(next)
                } else {
                    Ok(current)
                }
            }
            Expr::MemberAccess { object, property } => {
                self.eval_member_call(object, property, &[], false)
            }
            Expr::TemplateLiteral { parts } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        TemplatePart::Str(s) => result.push_str(s),
                        TemplatePart::Expression(expr) => {
                            let val = self.eval_expr(expr)?;
                            result.push_str(&val.to_js_string());
                        }
                    }
                }
                Ok(JsValue::String(result))
            }
            Expr::ObjectLiteral { properties } => {
                let mut obj = JsObject::new();
                for (key, val_expr) in properties {
                    let val = self.eval_expr(val_expr)?;
                    obj.set(key.clone(), val);
                }
                Ok(JsValue::Object(obj.wrapped()))
            }
            Expr::ArrayLiteral { elements } => {
                let vals: Vec<JsValue> = elements
                    .iter()
                    .map(|e| self.eval_expr(e))
                    .collect::<Result<_, _>>()?;
                Ok(JsValue::Array(JsArray::new(vals).wrapped()))
            }
            Expr::ComputedMemberAccess { object, property } => {
                let obj_val = self.eval_expr(object)?;
                let key = self.eval_expr(property)?.to_js_string();
                self.get_property(&obj_val, &key)
            }
            Expr::MemberAssign {
                object,
                property,
                value,
            } => {
                let obj_val = self.eval_expr(object)?;
                let key = self.eval_expr(property)?.to_js_string();
                let val = self.eval_expr(value)?;
                self.set_property(&obj_val, &key, val.clone())?;
                Ok(val)
            }
            Expr::Logical { left, op, right } => {
                let lhs = self.eval_expr(left)?;
                match op {
                    LogicalOp::And => {
                        if lhs.to_boolean() {
                            self.eval_expr(right)
                        } else {
                            Ok(lhs)
                        }
                    }
                    LogicalOp::Or => {
                        if lhs.to_boolean() {
                            Ok(lhs)
                        } else {
                            self.eval_expr(right)
                        }
                    }
                    LogicalOp::Nullish => {
                        if matches!(lhs, JsValue::Null | JsValue::Undefined) {
                            self.eval_expr(right)
                        } else {
                            Ok(lhs)
                        }
                    }
                }
            }
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                if self.eval_expr(condition)?.to_boolean() {
                    self.eval_expr(then_expr)
                } else {
                    self.eval_expr(else_expr)
                }
            }
            Expr::ArrowFunction { params, body } => {
                let body = match body {
                    ArrowBody::Block(stmts) => stmts.clone(),
                    ArrowBody::Expr(expr) => vec![Stmt::Return(Some(*expr.clone()))],
                };
                Ok(JsValue::Function {
                    name: "<arrow>".to_string(),
                    params: params.clone(),
                    body,
                    closure_env: self.env.capture(),
                })
            }
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        if let Expr::MemberAccess { object, property } = callee {
            return self.eval_member_call(object, property, args, true);
        }

        let func = self.eval_expr(callee)?;
        let arg_values: Vec<JsValue> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<_, _>>()?;

        self.call_function(&func, &arg_values)
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
        BinOp::Mod => Ok(JsValue::Number(ln % rn)),
        BinOp::Less => Ok(JsValue::Boolean(ln < rn)),
        BinOp::LessEq => Ok(JsValue::Boolean(ln <= rn)),
        BinOp::Greater => Ok(JsValue::Boolean(ln > rn)),
        BinOp::GreaterEq => Ok(JsValue::Boolean(ln >= rn)),
        BinOp::EqEqEq => Ok(JsValue::Boolean(lhs == rhs)),
        BinOp::NotEqEq => Ok(JsValue::Boolean(lhs != rhs)),
    }
}

fn eval_compound(lhs: JsValue, op: &AssignOp, rhs: JsValue) -> Result<JsValue, RuntimeError> {
    let bin = match op {
        AssignOp::Add => BinOp::Add,
        AssignOp::Sub => BinOp::Sub,
        AssignOp::Mul => BinOp::Mul,
        AssignOp::Div => BinOp::Div,
        AssignOp::Mod => BinOp::Mod,
    };
    eval_binary(lhs, &bin, rhs)
}
fn eval_unary(op: &UnaryOp, val: JsValue) -> Result<JsValue, RuntimeError> {
    match op {
        UnaryOp::Neg => Ok(JsValue::Number(-val.to_number())),
        UnaryOp::Not => Ok(JsValue::Boolean(!val.to_boolean())),
    }
}

use super::{ControlFlow, Interpreter};
use crate::errors::RuntimeError;
use crate::parser::ast::Stmt;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::VarDecl { name, init } => {
                let value = match init {
                    Some(expr) => self.eval_expr(expr)?,
                    None => JsValue::Undefined,
                };
                self.env.define(name.clone(), value);
                Ok(ControlFlow::None)
            }
            Stmt::Block(stmts) => {
                self.env.push_scope();
                let mut result = ControlFlow::None;
                for s in stmts {
                    result = self.eval_stmt(s)?;
                    if matches!(result, ControlFlow::Return(_)) {
                        break;
                    }
                }
                self.env.pop_scope();
                Ok(result)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.to_boolean() {
                    self.eval_stmt(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.eval_stmt(else_branch)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    let cond_val = self.eval_expr(condition)?;
                    if !cond_val.to_boolean() {
                        break;
                    }
                    if let ControlFlow::Return(v) = self.eval_stmt(body)? {
                        return Ok(ControlFlow::Return(v));
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::FunctionDecl { name, params, body } => {
                let func = JsValue::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                };
                self.env.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let value = match expr {
                    Some(e) => self.eval_expr(e)?,
                    None => JsValue::Undefined,
                };
                Ok(ControlFlow::Return(value))
            }
        }
    }
}

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
            Stmt::ForLoop {
                init,
                condition,
                update,
                body,
            } => {
                self.env.push_scope();
                if let Some(init_stmt) = init {
                    self.eval_stmt(init_stmt)?;
                }
                loop {
                    if let Some(cond) = condition {
                        if !self.eval_expr(cond)?.to_boolean() {
                            break;
                        }
                    }
                    if let ControlFlow::Return(v) = self.eval_stmt(body)? {
                        self.env.pop_scope();
                        return Ok(ControlFlow::Return(v));
                    }
                    if let Some(upd) = update {
                        self.eval_expr(upd)?;
                    }
                }
                self.env.pop_scope();
                Ok(ControlFlow::None)
            }
            Stmt::ForOf {
                variable,
                iterable,
                body,
            } => {
                let iter_val = self.eval_expr(iterable)?;
                let elements = match &iter_val {
                    JsValue::Array(arr) => arr.borrow().elements.clone(),
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "for-of requires an iterable".to_string(),
                        })
                    }
                };
                self.env.push_scope();
                self.env.define(variable.clone(), JsValue::Undefined);
                for elem in &elements {
                    self.env.set(variable, elem.clone())?;
                    if let ControlFlow::Return(v) = self.eval_stmt(body)? {
                        self.env.pop_scope();
                        return Ok(ControlFlow::Return(v));
                    }
                }
                self.env.pop_scope();
                Ok(ControlFlow::None)
            }
        }
    }
}

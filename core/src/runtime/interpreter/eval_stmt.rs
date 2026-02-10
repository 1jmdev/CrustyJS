use super::error_handling::JsException;
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
            Stmt::VarDecl { pattern, init } => {
                let value = match init {
                    Some(expr) => self.eval_expr(expr)?,
                    None => JsValue::Undefined,
                };
                self.eval_pattern_binding(pattern, value)?;
                Ok(ControlFlow::None)
            }
            Stmt::Block(stmts) => self.eval_block(stmts),
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
                    closure_env: self.env.capture(),
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
            Stmt::Throw(expr) => {
                let value = self.eval_expr(expr)?;
                Err(JsException::new(value).into_runtime_error())
            }
            Stmt::TryCatch {
                try_block,
                catch_param,
                catch_block,
                finally_block,
            } => self.eval_try_catch(try_block, catch_param, catch_block, finally_block),
            Stmt::Class(class_decl) => {
                self.eval_class_decl(class_decl)?;
                Ok(ControlFlow::None)
            }
        }
    }

    fn eval_block(&mut self, stmts: &[Stmt]) -> Result<ControlFlow, RuntimeError> {
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

    fn eval_try_catch(
        &mut self,
        try_block: &[Stmt],
        catch_param: &Option<String>,
        catch_block: &Option<Vec<Stmt>>,
        finally_block: &Option<Vec<Stmt>>,
    ) -> Result<ControlFlow, RuntimeError> {
        let mut flow = ControlFlow::None;
        let mut pending_error = None;

        match self.eval_block(try_block) {
            Ok(v) => flow = v,
            Err(err) => {
                if let RuntimeError::Thrown { value } = err {
                    if let Some(catch_stmts) = catch_block {
                        self.env.push_scope();
                        if let Some(name) = catch_param {
                            self.env.define(name.clone(), value);
                        }
                        let mut catch_flow = ControlFlow::None;
                        for stmt in catch_stmts {
                            catch_flow = self.eval_stmt(stmt)?;
                            if matches!(catch_flow, ControlFlow::Return(_)) {
                                break;
                            }
                        }
                        self.env.pop_scope();
                        flow = catch_flow;
                    } else {
                        pending_error = Some(RuntimeError::Thrown { value });
                    }
                } else {
                    pending_error = Some(err);
                }
            }
        }

        if let Some(finally_stmts) = finally_block {
            let finally_flow = self.eval_block(finally_stmts)?;
            if !matches!(finally_flow, ControlFlow::None) {
                return Ok(finally_flow);
            }
        }

        if let Some(err) = pending_error {
            return Err(err);
        }

        Ok(flow)
    }
}

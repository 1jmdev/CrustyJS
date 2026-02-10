use super::{ControlFlow, Interpreter};
use crate::errors::RuntimeError;
use crate::parser::ast::{Stmt, VarDeclKind};
use crate::runtime::environment::BindingKind;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::VarDecl {
                kind,
                pattern,
                init,
            } => {
                let value = match init {
                    Some(expr) => self.eval_expr(expr)?,
                    None => JsValue::Undefined,
                };
                let binding_kind = match kind {
                    VarDeclKind::Let => BindingKind::Let,
                    VarDeclKind::Const => BindingKind::Const,
                };
                self.eval_pattern_binding_with_kind(pattern, value, binding_kind)?;
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
                    match self.eval_stmt(body)? {
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::Break => break,
                        ControlFlow::None => {}
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::FunctionDecl {
                name,
                params,
                body,
                is_async,
                decl_offset,
            } => {
                let func = JsValue::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure_env: self.env.capture(),
                    is_async: *is_async,
                    source_path: self.module_stack.last().map(|p| p.display().to_string()),
                    source_offset: *decl_offset,
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
            Stmt::Break => Ok(ControlFlow::Break),
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
                    match self.eval_stmt(body)? {
                        ControlFlow::Return(v) => {
                            self.env.pop_scope();
                            return Ok(ControlFlow::Return(v));
                        }
                        ControlFlow::Break => break,
                        ControlFlow::None => {}
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
                    match self.eval_stmt(body)? {
                        ControlFlow::Return(v) => {
                            self.env.pop_scope();
                            return Ok(ControlFlow::Return(v));
                        }
                        ControlFlow::Break => break,
                        ControlFlow::None => {}
                    }
                }
                self.env.pop_scope();
                Ok(ControlFlow::None)
            }
            Stmt::ForIn {
                variable,
                object,
                body,
            } => {
                let source = self.eval_expr(object)?;
                let keys: Vec<String> = match source {
                    JsValue::Object(obj) => obj.borrow().properties.keys().cloned().collect(),
                    JsValue::Array(arr) => (0..arr.borrow().len()).map(|i| i.to_string()).collect(),
                    JsValue::String(s) => (0..s.chars().count()).map(|i| i.to_string()).collect(),
                    _ => Vec::new(),
                };

                self.env.push_scope();
                self.env
                    .define(variable.clone(), JsValue::String(String::new()));
                for key in keys {
                    self.env.set(variable, JsValue::String(key))?;
                    match self.eval_stmt(body)? {
                        ControlFlow::Return(v) => {
                            self.env.pop_scope();
                            return Ok(ControlFlow::Return(v));
                        }
                        ControlFlow::Break => break,
                        ControlFlow::None => {}
                    }
                }
                self.env.pop_scope();
                Ok(ControlFlow::None)
            }
            Stmt::Throw(expr) => self.eval_throw_expr(expr),
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
            Stmt::Switch {
                discriminant,
                cases,
            } => self.eval_switch(discriminant, cases),
            Stmt::Import(decl) => self.eval_import_stmt(decl),
            Stmt::Export(decl) => {
                let flow = self.eval_export_stmt(decl)?;
                if let crate::parser::ast::ExportDecl::NamedStmt(inner) = decl {
                    for name in Interpreter::export_names_from_stmt(inner) {
                        if let Ok(value) = self.env.get(&name) {
                            self.env.define(format!("__export_{name}"), value);
                        }
                    }
                }
                Ok(flow)
            }
        }
    }

    pub(crate) fn eval_block(&mut self, stmts: &[Stmt]) -> Result<ControlFlow, RuntimeError> {
        self.env.push_scope();
        let mut result = ControlFlow::None;
        for s in stmts {
            result = self.eval_stmt(s)?;
            if matches!(result, ControlFlow::Return(_) | ControlFlow::Break) {
                break;
            }
        }
        self.env.pop_scope();
        Ok(result)
    }
}

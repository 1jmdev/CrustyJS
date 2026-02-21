use super::{ControlFlow, Interpreter};
use crate::errors::RuntimeError;
use crate::parser::ast::{Stmt, VarDeclKind};
use crate::runtime::environment::BindingKind;
use crate::runtime::value::JsValue;

macro_rules! loop_body {
    ($flow:expr) => {
        match $flow {
            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
            ControlFlow::Break(None) => break,
            ControlFlow::Break(label) => return Ok(ControlFlow::Break(label)),
            ControlFlow::Continue(None) => {}
            ControlFlow::Continue(label) => return Ok(ControlFlow::Continue(label)),
            ControlFlow::None => {}
        }
    };
    ($flow:expr, scope: $self:expr) => {
        match $flow {
            ControlFlow::Return(v) => {
                $self.env.pop_scope();
                return Ok(ControlFlow::Return(v));
            }
            ControlFlow::Break(None) => break,
            ControlFlow::Break(label) => {
                $self.env.pop_scope();
                return Ok(ControlFlow::Break(label));
            }
            ControlFlow::Continue(None) => {}
            ControlFlow::Continue(label) => {
                $self.env.pop_scope();
                return Ok(ControlFlow::Continue(label));
            }
            ControlFlow::None => {}
        }
    };
}

impl Interpreter {
    pub(crate) fn eval_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        self.check_step_limit()?;
        match stmt {
            Stmt::Empty => Ok(ControlFlow::None),
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
                    Some(e) => self.eval_expr(e)?,
                    None => JsValue::Undefined,
                };
                self.eval_pattern_binding_with_kind(pattern, value, var_binding(kind))?;
                Ok(ControlFlow::None)
            }
            Stmt::Block(stmts) => self.eval_block(stmts),
            Stmt::VarDeclList { kind, declarations } => {
                for (pattern, init) in declarations {
                    let value = match init {
                        Some(e) => self.eval_expr(e)?,
                        None => JsValue::Undefined,
                    };
                    self.eval_pattern_binding_with_kind(pattern, value, var_binding(kind))?;
                }
                Ok(ControlFlow::None)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.eval_expr(condition)?.to_boolean() {
                    self.eval_stmt(then_branch)
                } else if let Some(b) = else_branch {
                    self.eval_stmt(b)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    if !self.eval_expr(condition)?.to_boolean() {
                        break;
                    }
                    loop_body!(self.eval_stmt(body)?);
                }
                Ok(ControlFlow::None)
            }
            Stmt::DoWhile { body, condition } => {
                loop {
                    loop_body!(self.eval_stmt(body)?);
                    if !self.eval_expr(condition)?.to_boolean() {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::ForLoop {
                init,
                condition,
                update,
                body,
            } => {
                self.env.push_scope(&mut self.heap);
                if let Some(s) = init {
                    self.eval_stmt(s)?;
                }
                loop {
                    if let Some(c) = condition {
                        if !self.eval_expr(c)?.to_boolean() {
                            break;
                        }
                    }
                    loop_body!(self.eval_stmt(body)?, scope: self);
                    if let Some(u) = update {
                        self.eval_expr(u)?;
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
                let elements = self.collect_iterable(&iter_val)?;
                self.env.push_scope(&mut self.heap);
                self.env.define(variable.clone(), JsValue::Undefined);
                for elem in &elements {
                    self.env.set(variable, elem.clone())?;
                    loop_body!(self.eval_stmt(body)?, scope: self);
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
                self.env.push_scope(&mut self.heap);
                self.env
                    .define(variable.clone(), JsValue::String(String::new()));
                for key in keys {
                    self.env.set(variable, JsValue::String(key))?;
                    loop_body!(self.eval_stmt(body)?, scope: self);
                }
                self.env.pop_scope();
                Ok(ControlFlow::None)
            }
            Stmt::FunctionDecl {
                name,
                params,
                body,
                is_async,
                is_generator,
                decl_offset,
            } => {
                let proto_gc = self
                    .heap
                    .alloc_cell(crate::runtime::value::object::JsObject::new());
                let mut fn_props = crate::runtime::value::object::JsObject::new();
                fn_props.set("prototype".into(), JsValue::Object(proto_gc));
                let func = JsValue::Function {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure_env: self.env.capture(),
                    is_async: *is_async,
                    is_generator: *is_generator,
                    source_path: self.module_stack.last().map(|p| p.display().to_string()),
                    source_offset: *decl_offset,
                    properties: Some(self.heap.alloc_cell(fn_props)),
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
            Stmt::Break { label } => Ok(ControlFlow::Break(label.clone())),
            Stmt::Continue { label } => Ok(ControlFlow::Continue(label.clone())),
            Stmt::Labeled { label, body } => match self.eval_stmt(body)? {
                ControlFlow::Break(Some(ref l)) if l == label => Ok(ControlFlow::None),
                other => Ok(other),
            },
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
        self.env.push_scope(&mut self.heap);
        let mut result = ControlFlow::None;
        for s in stmts {
            result = self.eval_stmt(s)?;
            if !matches!(result, ControlFlow::None) {
                break;
            }
        }
        self.env.pop_scope();
        Ok(result)
    }
}

fn var_binding(kind: &VarDeclKind) -> BindingKind {
    match kind {
        VarDeclKind::Let => BindingKind::Let,
        VarDeclKind::Const => BindingKind::Const,
        VarDeclKind::Var => BindingKind::Var,
    }
}

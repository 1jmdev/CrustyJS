use super::{ControlFlow, Interpreter};
use crate::errors::RuntimeError;
use crate::parser::ast::{ExportDecl, ImportSpecifier, Pattern, Stmt};
use crate::runtime::modules::resolver;
use crate::runtime::value::JsValue;
use std::collections::HashMap;
use std::path::PathBuf;

impl Interpreter {
    pub(crate) fn eval_import_stmt(
        &mut self,
        decl: &crate::parser::ast::ImportDecl,
    ) -> Result<ControlFlow, RuntimeError> {
        let current = self
            .module_stack
            .last()
            .cloned()
            .unwrap_or_else(|| PathBuf::from("."));
        let path = resolver::resolve(&decl.source, &current);
        let exports = self.load_module_exports(path)?;

        for spec in &decl.specifiers {
            match spec {
                ImportSpecifier::Default(local) => {
                    let value = exports
                        .get("default")
                        .cloned()
                        .unwrap_or(JsValue::Undefined);
                    self.env.define(local.clone(), value);
                }
                ImportSpecifier::Named { imported, local } => {
                    let value = exports.get(imported).cloned().unwrap_or(JsValue::Undefined);
                    self.env.define(local.clone(), value);
                }
                ImportSpecifier::Namespace(local) => {
                    let mut obj = crate::runtime::value::object::JsObject::new();
                    for (k, v) in &exports {
                        obj.set(k.clone(), v.clone());
                    }
                    self.env
                        .define(local.clone(), JsValue::Object(obj.wrapped()));
                }
            }
        }

        Ok(ControlFlow::None)
    }

    pub(crate) fn eval_export_stmt(
        &mut self,
        decl: &ExportDecl,
    ) -> Result<ControlFlow, RuntimeError> {
        match decl {
            ExportDecl::NamedStmt(stmt) => self.eval_stmt(stmt),
            ExportDecl::Default(expr) => {
                let value = self.eval_expr(expr)?;
                self.env.define("__default_export".to_string(), value);
                Ok(ControlFlow::None)
            }
            ExportDecl::NamedList(specs) => {
                for spec in specs {
                    let value = self.env.get(&spec.local)?;
                    self.env
                        .define(format!("__export_{}", spec.exported), value);
                }
                Ok(ControlFlow::None)
            }
        }
    }

    fn load_module_exports(
        &mut self,
        path: PathBuf,
    ) -> Result<HashMap<String, JsValue>, RuntimeError> {
        let key = path.to_string_lossy().to_string();
        if let Some(cached) = self.module_cache.get(&key) {
            return Ok(cached);
        }

        let source = std::fs::read_to_string(&path).map_err(|e| RuntimeError::TypeError {
            message: format!("failed to read module '{}': {e}", path.display()),
        })?;
        let tokens = crate::lexer::lex(&source).map_err(|e| RuntimeError::TypeError {
            message: format!("failed to lex module '{}': {e}", path.display()),
        })?;
        let program = crate::parser::parse(tokens).map_err(|e| RuntimeError::TypeError {
            message: format!("failed to parse module '{}': {e}", path.display()),
        })?;

        self.module_stack.push(path.clone());
        self.env.push_scope();
        for stmt in &program.body {
            self.eval_stmt(stmt)?;
        }

        let mut exports = HashMap::new();
        let scope_bindings = self.env.current_scope_bindings_snapshot();

        for (name, binding) in scope_bindings {
            if name == "__default_export" {
                exports.insert("default".to_string(), binding.value);
            } else if let Some(export_name) = name.strip_prefix("__export_") {
                exports.insert(export_name.to_string(), binding.value);
            } else {
                exports.insert(name, binding.value);
            }
        }

        self.env.pop_scope();
        self.module_stack.pop();
        self.module_cache.insert(key, exports.clone());
        Ok(exports)
    }

    pub(crate) fn export_names_from_stmt(stmt: &Stmt) -> Vec<String> {
        match stmt {
            Stmt::FunctionDecl { name, .. } => vec![name.clone()],
            Stmt::VarDecl { pattern, .. } => match pattern {
                Pattern::Identifier(name) => vec![name.clone()],
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }
}

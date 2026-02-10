use std::cell::RefCell;
use std::rc::Rc;

use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{ClassDecl, ClassMethod, Expr};
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

#[derive(Clone)]
pub(crate) struct RuntimeClass {
    pub constructor: JsValue,
    pub prototype: Rc<RefCell<JsObject>>,
    pub parent: Option<String>,
}

impl Interpreter {
    pub(crate) fn eval_class_decl(&mut self, class_decl: &ClassDecl) -> Result<(), RuntimeError> {
        let parent = class_decl
            .parent
            .as_ref()
            .map(|name| {
                self.classes
                    .get(name)
                    .cloned()
                    .ok_or_else(|| RuntimeError::TypeError {
                        message: format!("unknown parent class '{name}'"),
                    })
            })
            .transpose()?;

        let mut prototype = JsObject::new();
        if let Some(parent_class) = &parent {
            prototype.prototype = Some(parent_class.prototype.clone());
        }

        for method in &class_decl.methods {
            if method.is_static {
                continue;
            }
            let method_value = self.method_to_function(method, &class_decl.name);
            prototype.set(method.name.clone(), method_value);
        }

        let prototype = prototype.wrapped();
        let constructor = match &class_decl.constructor {
            Some(method) => self.method_to_function(method, &class_decl.name),
            None => JsValue::Function {
                name: format!("{}::constructor", class_decl.name),
                params: Vec::new(),
                body: Vec::new(),
                closure_env: self.env.capture(),
            },
        };

        self.classes.insert(
            class_decl.name.clone(),
            RuntimeClass {
                constructor: constructor.clone(),
                prototype,
                parent: class_decl.parent.clone(),
            },
        );

        self.env.define(class_decl.name.clone(), constructor);
        Ok(())
    }

    pub(crate) fn eval_new(
        &mut self,
        callee: &crate::parser::ast::Expr,
        args: &[crate::parser::ast::Expr],
    ) -> Result<JsValue, RuntimeError> {
        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Error") {
            let message = args
                .first()
                .map(|expr| self.eval_expr(expr))
                .transpose()?
                .unwrap_or(JsValue::Undefined);
            return Ok(super::error_handling::create_error_object(message));
        }

        let class_name = if let crate::parser::ast::Expr::Identifier(name) = callee {
            name
        } else {
            return Err(RuntimeError::TypeError {
                message: "new currently supports named classes and Error".to_string(),
            });
        };

        let class =
            self.classes
                .get(class_name)
                .cloned()
                .ok_or_else(|| RuntimeError::TypeError {
                    message: format!("'{class_name}' is not a class constructor"),
                })?;

        let arg_values: Vec<JsValue> = args
            .iter()
            .map(|arg| self.eval_expr(arg))
            .collect::<Result<_, _>>()?;

        let mut instance = JsObject::new();
        instance.prototype = Some(class.prototype.clone());
        let instance_value = JsValue::Object(instance.wrapped());

        self.super_stack.push(class.parent.clone());
        let ctor_result = self.call_function_with_this(
            &class.constructor,
            &arg_values,
            Some(instance_value.clone()),
        );
        self.super_stack.pop();
        ctor_result?;

        Ok(instance_value)
    }

    pub(crate) fn eval_super_call(
        &mut self,
        args: &[crate::parser::ast::Expr],
    ) -> Result<JsValue, RuntimeError> {
        let parent_name =
            self.super_stack
                .last()
                .cloned()
                .flatten()
                .ok_or_else(|| RuntimeError::TypeError {
                    message: "super() is only valid inside class constructors".to_string(),
                })?;

        let parent_class =
            self.classes
                .get(&parent_name)
                .cloned()
                .ok_or_else(|| RuntimeError::TypeError {
                    message: format!("unknown parent class '{parent_name}'"),
                })?;

        let this_value = self.env.get("this")?;
        let arg_values: Vec<JsValue> = args
            .iter()
            .map(|arg| self.eval_expr(arg))
            .collect::<Result<_, _>>()?;

        self.super_stack.push(parent_class.parent.clone());
        let result =
            self.call_function_with_this(&parent_class.constructor, &arg_values, Some(this_value));
        self.super_stack.pop();
        result?;
        Ok(JsValue::Undefined)
    }

    pub(crate) fn eval_instanceof_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
    ) -> Result<JsValue, RuntimeError> {
        let instance = self.eval_expr(left)?;
        let class_name = match right {
            Expr::Identifier(name) => name,
            _ => return Ok(JsValue::Boolean(false)),
        };

        let class = match self.classes.get(class_name) {
            Some(class) => class,
            None => return Ok(JsValue::Boolean(false)),
        };

        let JsValue::Object(object) = instance else {
            return Ok(JsValue::Boolean(false));
        };

        let mut current = object.borrow().prototype.clone();
        while let Some(proto) = current {
            if Rc::ptr_eq(&proto, &class.prototype) {
                return Ok(JsValue::Boolean(true));
            }
            current = proto.borrow().prototype.clone();
        }

        Ok(JsValue::Boolean(false))
    }

    fn method_to_function(&self, method: &ClassMethod, class_name: &str) -> JsValue {
        JsValue::Function {
            name: format!("{class_name}::{}", method.name),
            params: method.params.clone(),
            body: method.body.clone(),
            closure_env: self.env.capture(),
        }
    }
}

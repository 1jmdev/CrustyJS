use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{ClassDecl, ClassMethod, ClassMethodKind, Expr, Param, Pattern};
use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

#[derive(Clone)]
pub(crate) struct RuntimeClass {
    pub constructor: JsValue,
    pub prototype: Gc<GcCell<JsObject>>,
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
            prototype.prototype = Some(parent_class.prototype);
        }

        for method in &class_decl.methods {
            if method.is_static {
                continue;
            }
            let method_value = self.method_to_function(method, &class_decl.name);
            match method.kind {
                ClassMethodKind::Method => prototype.set(method.name.clone(), method_value),
                ClassMethodKind::Getter => prototype.set_getter(method.name.clone(), method_value),
                ClassMethodKind::Setter => prototype.set_setter(method.name.clone(), method_value),
            }
        }

        let prototype = self.heap.alloc_cell(prototype);
        let constructor = match &class_decl.constructor {
            Some(method) => self.method_to_function(method, &class_decl.name),
            None => JsValue::Function {
                name: format!("{}::constructor", class_decl.name),
                params: Vec::new(),
                body: Vec::new(),
                closure_env: self.env.capture(),
                is_async: false,
                is_generator: false,
                source_path: self.module_stack.last().map(|p| p.display().to_string()),
                source_offset: 0,
                properties: None,
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
        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Promise") {
            return self.eval_new_promise(args);
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Symbol") {
            return Err(RuntimeError::TypeError {
                message: "Symbol is not a constructor".to_string(),
            });
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Error") {
            let message = args
                .first()
                .map(|expr| self.eval_expr(expr))
                .transpose()?
                .unwrap_or(JsValue::Undefined);
            return Ok(super::error_handling::create_error_object(
                message,
                &mut self.heap,
            ));
        }

        if let crate::parser::ast::Expr::Identifier(name) = callee {
            match name.as_str() {
                "TypeError" | "ReferenceError" | "SyntaxError" | "RangeError" | "URIError"
                | "EvalError" => {
                    let message = args
                        .first()
                        .map(|expr| self.eval_expr(expr))
                        .transpose()?
                        .unwrap_or(JsValue::Undefined);
                    let mut obj = JsObject::new();
                    obj.set("name".to_string(), JsValue::String(name.clone()));
                    obj.set(
                        "message".to_string(),
                        JsValue::String(message.to_js_string()),
                    );
                    obj.set("[[ErrorType]]".to_string(), JsValue::String(name.clone()));
                    return Ok(JsValue::Object(self.heap.alloc_cell(obj)));
                }
                "Number" => {
                    let val = args
                        .first()
                        .map(|expr| self.eval_expr(expr))
                        .transpose()?
                        .unwrap_or(JsValue::Number(0.0));
                    let mut obj = JsObject::new();
                    obj.set(
                        "[[PrimitiveValue]]".to_string(),
                        JsValue::Number(val.to_number()),
                    );
                    return Ok(JsValue::Object(self.heap.alloc_cell(obj)));
                }
                "Boolean" => {
                    let val = args
                        .first()
                        .map(|expr| self.eval_expr(expr))
                        .transpose()?
                        .unwrap_or(JsValue::Boolean(false));
                    let mut obj = JsObject::new();
                    obj.set(
                        "[[PrimitiveValue]]".to_string(),
                        JsValue::Boolean(val.to_boolean()),
                    );
                    return Ok(JsValue::Object(self.heap.alloc_cell(obj)));
                }
                "String" => {
                    let val = args
                        .first()
                        .map(|expr| self.eval_expr(expr))
                        .transpose()?
                        .unwrap_or(JsValue::String(String::new()));
                    let mut obj = JsObject::new();
                    obj.set(
                        "[[PrimitiveValue]]".to_string(),
                        JsValue::String(val.to_js_string()),
                    );
                    return Ok(JsValue::Object(self.heap.alloc_cell(obj)));
                }
                "Object" => {
                    let val = args
                        .first()
                        .map(|expr| self.eval_expr(expr))
                        .transpose()?
                        .unwrap_or(JsValue::Undefined);
                    return match val {
                        JsValue::Object(_) => Ok(val),
                        JsValue::Null | JsValue::Undefined => {
                            Ok(JsValue::Object(self.heap.alloc_cell(JsObject::new())))
                        }
                        _ => Ok(JsValue::Object(self.heap.alloc_cell(JsObject::new()))),
                    };
                }
                _ => {}
            }
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Map") {
            return self.eval_new_map(args);
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Set") {
            return self.eval_new_set(args);
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "WeakMap") {
            return self.eval_new_weak_map(args);
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "WeakSet") {
            return self.eval_new_weak_set(args);
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "RegExp") {
            return self.eval_new_regexp(args);
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Date") {
            // new Date() returns a Date-like object with a timestamp
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let timestamp = if args.is_empty() {
                now.as_millis() as f64
            } else {
                let val = self.eval_expr(&args[0])?;
                val.to_number()
            };
            let mut obj = JsObject::new();
            obj.set("[[PrimitiveValue]]".to_string(), JsValue::Number(timestamp));
            obj.set("[[DateValue]]".to_string(), JsValue::Number(timestamp));
            return Ok(JsValue::Object(self.heap.alloc_cell(obj)));
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Array") {
            let arg_values = self.eval_call_args(args)?;
            let elements = if arg_values.len() == 1 {
                if let JsValue::Number(n) = &arg_values[0] {
                    let len = (*n as usize).min(1 << 20);
                    vec![JsValue::Undefined; len]
                } else {
                    arg_values
                }
            } else {
                arg_values
            };
            return Ok(JsValue::Array(
                self.heap
                    .alloc_cell(crate::runtime::value::array::JsArray::new(elements)),
            ));
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Function") {
            // new Function() - stub
            return Ok(JsValue::Function {
                name: "anonymous".to_string(),
                params: vec![],
                body: vec![],
                closure_env: self.env.capture(),
                is_async: false,
                is_generator: false,
                source_path: None,
                source_offset: 0,
                properties: None,
            });
        }

        if matches!(callee, crate::parser::ast::Expr::Identifier(name) if name == "Proxy") {
            return self.eval_new_proxy(args);
        }

        if let crate::parser::ast::Expr::Identifier(name) = callee {
            if let Ok(val) = self.env.get(name) {
                if let JsValue::Proxy(proxy) = &val {
                    let (trap, target) = {
                        let p = proxy.borrow();
                        p.check_revoked()
                            .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                        (p.get_trap("construct"), p.target.clone())
                    };
                    let arg_values: Vec<JsValue> = args
                        .iter()
                        .map(|arg| self.eval_expr(arg))
                        .collect::<Result<_, _>>()?;
                    if let Some(trap_fn) = trap {
                        let args_array = JsValue::Array(
                            self.heap
                                .alloc_cell(crate::runtime::value::array::JsArray::new(arg_values)),
                        );
                        return self.call_function(&trap_fn, &[target, args_array, val]);
                    }
                    if let JsValue::Function { name: fn_name, .. } = &target {
                        if let Some(class_name) = fn_name.strip_suffix("::constructor") {
                            if let Some(class) = self.classes.get(class_name).cloned() {
                                let mut instance = crate::runtime::value::object::JsObject::new();
                                instance.prototype = Some(class.prototype);
                                let instance_value =
                                    JsValue::Object(self.heap.alloc_cell(instance));
                                self.call_function_with_this(
                                    &class.constructor,
                                    &arg_values,
                                    Some(instance_value.clone()),
                                )?;
                                return Ok(instance_value);
                            }
                        }
                    }
                    return self.call_function(&target, &arg_values);
                }
            }
        }

        let class_name = if let crate::parser::ast::Expr::Identifier(name) = callee {
            name
        } else {
            return Err(RuntimeError::TypeError {
                message: "new currently supports named classes and Error".to_string(),
            });
        };

        // First try self.classes (class declarations)
        if let Some(class) = self.classes.get(class_name).cloned() {
            let arg_values: Vec<JsValue> = args
                .iter()
                .map(|arg| self.eval_expr(arg))
                .collect::<Result<_, _>>()?;

            let mut instance = JsObject::new();
            instance.prototype = Some(class.prototype);
            let instance_value = JsValue::Object(self.heap.alloc_cell(instance));

            self.super_stack.push(class.parent.clone());
            let ctor_result = self.call_function_with_this(
                &class.constructor,
                &arg_values,
                Some(instance_value.clone()),
            );
            self.super_stack.pop();
            ctor_result?;

            return Ok(instance_value);
        }

        // Try plain function constructor
        if let Ok(func_val) = self.env.get(class_name) {
            if let JsValue::Function { ref properties, .. } = func_val {
                let arg_values: Vec<JsValue> = args
                    .iter()
                    .map(|arg| self.eval_expr(arg))
                    .collect::<Result<_, _>>()?;

                let mut instance = JsObject::new();
                if let Some(props) = properties {
                    let borrowed = props.borrow();
                    if let Some(proto_prop) = borrowed.properties.get("prototype") {
                        if let JsValue::Object(proto_obj) = &proto_prop.value {
                            instance.prototype = Some(*proto_obj);
                        }
                    }
                }
                let instance_value = JsValue::Object(self.heap.alloc_cell(instance));

                let result = self.call_function_with_this(
                    &func_val,
                    &arg_values,
                    Some(instance_value.clone()),
                )?;

                // If the constructor returns an object, use that instead
                if matches!(result, JsValue::Object(_)) {
                    return Ok(result);
                }
                return Ok(instance_value);
            }
        }

        Err(RuntimeError::TypeError {
            message: format!("'{class_name}' is not a class constructor"),
        })
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

        // Check for error type objects created by the runtime
        if let JsValue::Object(obj) = &instance {
            let borrowed = obj.borrow();
            if let Some(prop) = borrowed.properties.get("[[ErrorType]]") {
                if let JsValue::String(error_type) = &prop.value {
                    return Ok(JsValue::Boolean(error_type == class_name));
                }
            }
        }

        let class = match self.classes.get(class_name) {
            Some(class) => class,
            None => return Ok(JsValue::Boolean(false)),
        };

        let JsValue::Object(object) = instance else {
            return Ok(JsValue::Boolean(false));
        };

        let mut current = object.borrow().prototype;
        while let Some(proto) = current {
            if Gc::ptr_eq(proto, class.prototype) {
                return Ok(JsValue::Boolean(true));
            }
            current = proto.borrow().prototype;
        }

        Ok(JsValue::Boolean(false))
    }

    pub(crate) fn eval_in_expr(
        &mut self,
        left: &Expr,
        right: &Expr,
    ) -> Result<JsValue, RuntimeError> {
        let key = self.eval_expr(left)?.to_js_string();
        let target = self.eval_expr(right)?;
        match &target {
            JsValue::Object(obj) => {
                let mut current = Some(*obj);
                while let Some(candidate) = current {
                    let borrowed = candidate.borrow();
                    if borrowed.properties.contains_key(&key) {
                        return Ok(JsValue::Boolean(true));
                    }
                    current = borrowed.prototype;
                }
                Ok(JsValue::Boolean(false))
            }
            JsValue::Proxy(proxy) => {
                let (trap, proxy_target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("has"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    let result =
                        self.call_function(&trap_fn, &[proxy_target, JsValue::String(key)])?;
                    Ok(JsValue::Boolean(result.to_boolean()))
                } else {
                    self.eval_in_value(&key, &proxy_target)
                }
            }
            JsValue::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    Ok(JsValue::Boolean(idx < arr.borrow().len()))
                } else {
                    Ok(JsValue::Boolean(false))
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: format!(
                    "Cannot use 'in' operator to search for '{}' in {}",
                    key,
                    target.to_js_string()
                ),
            }),
        }
    }

    pub(crate) fn eval_in_value(
        &mut self,
        key: &str,
        target: &JsValue,
    ) -> Result<JsValue, RuntimeError> {
        match target {
            JsValue::Object(obj) => {
                let mut current = Some(*obj);
                while let Some(candidate) = current {
                    let borrowed = candidate.borrow();
                    if borrowed.properties.contains_key(key) {
                        return Ok(JsValue::Boolean(true));
                    }
                    current = borrowed.prototype;
                }
                Ok(JsValue::Boolean(false))
            }
            JsValue::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    Ok(JsValue::Boolean(idx < arr.borrow().len()))
                } else {
                    Ok(JsValue::Boolean(false))
                }
            }
            JsValue::Proxy(proxy) => {
                let (trap, proxy_target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("has"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    let result = self.call_function(
                        &trap_fn,
                        &[proxy_target, JsValue::String(key.to_string())],
                    )?;
                    Ok(JsValue::Boolean(result.to_boolean()))
                } else {
                    self.eval_in_value(key, &proxy_target)
                }
            }
            _ => Ok(JsValue::Boolean(false)),
        }
    }

    fn method_to_function(&self, method: &ClassMethod, class_name: &str) -> JsValue {
        let params = method
            .params
            .iter()
            .map(|name| Param {
                pattern: Pattern::Identifier(name.clone()),
                default: None,
            })
            .collect();
        JsValue::Function {
            name: format!("{class_name}::{}", method.name),
            params,
            body: method.body.clone(),
            closure_env: self.env.capture(),
            is_async: false,
            is_generator: false,
            source_path: self.module_stack.last().map(|p| p.display().to_string()),
            source_offset: 0,
            properties: None,
        }
    }
}

use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{
    ArrowBody, BinOp, Expr, LogicalOp, ObjectProperty, OptionalOp, Stmt, TemplatePart, UpdateOp,
};
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::regexp::{JsRegExp, RegExpFlags};
use crate::runtime::value::JsValue;
use crate::runtime::value::{eval_literal, eval_unary};
impl Interpreter {
    pub(crate) fn eval_expr(&mut self, expr: &Expr) -> Result<JsValue, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(eval_literal(lit)),
            Expr::Identifier(name) => self.env.get(name),
            Expr::Binary { left, op, right } => {
                if matches!(op, BinOp::Instanceof) {
                    return self.eval_instanceof_expr(left, right);
                }
                if matches!(op, BinOp::In) {
                    return self.eval_in_expr(left, right);
                }
                let lhs = self.eval_expr(left)?;
                let rhs = self.eval_expr(right)?;
                self.eval_binary(lhs, op, rhs)
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
                let next = self.eval_compound(current, op, rhs)?;
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
                for property in properties {
                    match property {
                        ObjectProperty::KeyValue(key, val_expr) => {
                            let val = self.eval_expr(val_expr)?;
                            let key = self.eval_property_key(key)?;
                            obj.set(key, val);
                        }
                        ObjectProperty::Getter(key, body) => {
                            let key = self.eval_property_key(key)?;
                            let getter = JsValue::Function {
                                name: format!("get {key}"),
                                params: Vec::new(),
                                body: body.clone(),
                                closure_env: self.env.capture(),
                                is_async: false,
                                is_generator: false,
                                source_path: self
                                    .module_stack
                                    .last()
                                    .map(|p| p.display().to_string()),
                                source_offset: 0,
                                properties: None,
                            };
                            obj.set_getter(key, getter);
                        }
                        ObjectProperty::Setter(key, param, body) => {
                            let key = self.eval_property_key(key)?;
                            let setter = JsValue::Function {
                                name: format!("set {key}"),
                                params: vec![crate::parser::ast::Param {
                                    pattern: crate::parser::ast::Pattern::Identifier(param.clone()),
                                    default: None,
                                }],
                                body: body.clone(),
                                closure_env: self.env.capture(),
                                is_async: false,
                                is_generator: false,
                                source_path: self
                                    .module_stack
                                    .last()
                                    .map(|p| p.display().to_string()),
                                source_offset: 0,
                                properties: None,
                            };
                            obj.set_setter(key, setter);
                        }
                        ObjectProperty::Spread(expr) => {
                            let spread_val = self.eval_expr(expr)?;
                            match spread_val {
                                JsValue::Object(source) => {
                                    let borrowed = source.borrow();
                                    for (k, p) in &borrowed.properties {
                                        obj.set(k.clone(), p.value.clone());
                                    }
                                }
                                JsValue::Undefined | JsValue::Null => {}
                                other => {
                                    return Err(RuntimeError::TypeError {
                                        message: format!(
                                            "cannot spread non-object value {other} in object literal"
                                        ),
                                    });
                                }
                            }
                        }
                    }
                }
                Ok(JsValue::Object(self.heap.alloc_cell(obj)))
            }
            Expr::ArrayLiteral { elements } => {
                let mut vals: Vec<JsValue> = Vec::new();
                for element in elements {
                    match element {
                        Expr::Spread(inner) => {
                            let spread_val = self.eval_expr(inner)?;
                            vals.extend(self.collect_iterable(&spread_val)?);
                        }
                        other => vals.push(self.eval_expr(other)?),
                    }
                }
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(vals))))
            }
            Expr::ComputedMemberAccess { object, property } => {
                let obj_val = self.eval_expr(object)?;
                let key_val = self.eval_expr(property)?;
                if let JsValue::Symbol(ref sym) = key_val {
                    return self.get_symbol_property(&obj_val, sym);
                }
                let key = key_val.to_js_string();
                self.get_property(&obj_val, &key)
            }
            Expr::MemberAssign {
                object,
                property,
                value,
            } => {
                let obj_val = self.eval_expr(object)?;
                let key_val = self.eval_expr(property)?;
                let val = self.eval_expr(value)?;
                if let JsValue::Symbol(ref sym) = key_val {
                    self.set_symbol_property(&obj_val, sym, val.clone())?;
                    return Ok(val);
                }
                let key = key_val.to_js_string();
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
            Expr::Typeof(expr) => {
                let val = match expr.as_ref() {
                    Expr::Identifier(name) => self.env.get(name).unwrap_or(JsValue::Undefined),
                    other => self.eval_expr(other)?,
                };
                let t = match val {
                    JsValue::Undefined => "undefined",
                    JsValue::Null => "object",
                    JsValue::Boolean(_) => "boolean",
                    JsValue::Number(_) => "number",
                    JsValue::String(_) => "string",
                    JsValue::Function { .. } => "function",
                    JsValue::NativeFunction { .. } => "function",
                    JsValue::Symbol(_) => "symbol",
                    JsValue::Object(_)
                    | JsValue::Array(_)
                    | JsValue::Promise(_)
                    | JsValue::Map(_)
                    | JsValue::Set(_)
                    | JsValue::WeakMap(_)
                    | JsValue::WeakSet(_)
                    | JsValue::RegExp(_)
                    | JsValue::Proxy(_) => "object",
                };
                Ok(JsValue::String(t.to_string()))
            }
            Expr::Spread(_) => Err(RuntimeError::TypeError {
                message: "spread syntax is only valid in calls and array literals".to_string(),
            }),
            Expr::New { callee, args } => self.eval_new(callee, args),
            Expr::SuperCall { args } => self.eval_super_call(args),
            Expr::Await(expr) => self.eval_await_expr(expr),
            Expr::Yield { value, delegate } => {
                if self.generator_depth == 0 {
                    return Err(RuntimeError::TypeError {
                        message: "yield is only valid inside generator functions".to_string(),
                    });
                }
                if *delegate {
                    // yield* iterable — collect all values from the sub-iterable
                    let inner = match value {
                        Some(expr) => self.eval_expr(expr)?,
                        None => JsValue::Undefined,
                    };
                    let items = self.collect_iterable(&inner)?;
                    self.generator_yields.extend(items);
                } else {
                    let val = match value {
                        Some(expr) => self.eval_expr(expr)?,
                        None => JsValue::Undefined,
                    };
                    self.generator_yields.push(val);
                }
                Ok(JsValue::Undefined)
            }
            Expr::ArrowFunction {
                params,
                body,
                is_async,
            } => {
                let body = match body {
                    ArrowBody::Block(stmts) => stmts.clone(),
                    ArrowBody::Expr(expr) => vec![Stmt::Return(Some(*expr.clone()))],
                };
                Ok(JsValue::Function {
                    name: "<arrow>".to_string(),
                    params: params.clone(),
                    body,
                    closure_env: self.env.capture(),
                    is_async: *is_async,
                    is_generator: false,
                    source_path: self.module_stack.last().map(|p| p.display().to_string()),
                    source_offset: 0,
                    properties: None,
                })
            }
            Expr::OptionalChain { base, chain } => {
                let mut current = self.eval_expr(base)?;
                for op in chain {
                    if matches!(current, JsValue::Null | JsValue::Undefined) {
                        return Ok(JsValue::Undefined);
                    }

                    current = match op {
                        OptionalOp::PropertyAccess(name) => self.get_property(&current, name)?,
                        OptionalOp::ComputedAccess(expr) => {
                            let key = self.eval_expr(expr)?.to_js_string();
                            self.get_property(&current, &key)?
                        }
                        OptionalOp::Call(args) => {
                            let arg_values = args
                                .iter()
                                .map(|arg| self.eval_expr(arg))
                                .collect::<Result<Vec<_>, _>>()?;
                            self.call_function(&current, &arg_values)?
                        }
                    };
                }

                Ok(current)
            }
            Expr::RegexLiteral { pattern, flags } => {
                let fl = RegExpFlags::from_str(flags)
                    .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                let re = JsRegExp::new(pattern, fl)
                    .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                Ok(JsValue::RegExp(self.heap.alloc_cell(re)))
            }
            Expr::Delete(operand) => self.eval_delete_expr(operand),
            Expr::Sequence(exprs) => {
                let mut result = JsValue::Undefined;
                for e in exprs {
                    result = self.eval_expr(e)?;
                }
                Ok(result)
            }
            Expr::FunctionExpr {
                name,
                params,
                body,
                is_async,
                is_generator,
            } => {
                let proto = crate::runtime::value::object::JsObject::new();
                let proto_gc = self.heap.alloc_cell(proto);
                let mut fn_props = crate::runtime::value::object::JsObject::new();
                fn_props.set("prototype".to_string(), JsValue::Object(proto_gc));
                Ok(JsValue::Function {
                    name: name.clone().unwrap_or_else(|| "<anonymous>".to_string()),
                    params: params.clone(),
                    body: body.clone(),
                    closure_env: self.env.capture(),
                    is_async: *is_async,
                    is_generator: *is_generator,
                    source_path: self.module_stack.last().map(|p| p.display().to_string()),
                    source_offset: 0,
                    properties: Some(self.heap.alloc_cell(fn_props)),
                })
            }
            Expr::TaggedTemplate { tag, parts } => {
                let func = self.eval_expr(tag)?;
                let mut strings = Vec::new();
                let mut raw_strings = Vec::new();
                let mut exprs = Vec::new();
                for part in parts {
                    match part {
                        TemplatePart::Str(s) => {
                            raw_strings.push(JsValue::String(s.clone()));
                            let cooked = s
                                .replace("\\n", "\n")
                                .replace("\\t", "\t")
                                .replace("\\\\", "\\");
                            strings.push(JsValue::String(cooked));
                        }
                        TemplatePart::Expression(expr) => {
                            exprs.push(self.eval_expr(expr)?);
                        }
                    }
                }
                let raw_arr = JsValue::Array(self.heap.alloc_cell(JsArray::new(raw_strings)));
                let tmpl_arr_gc = self.heap.alloc_cell(JsArray::new(strings));
                let tmpl_obj = JsValue::Array(tmpl_arr_gc);
                self.set_property(&tmpl_obj, "raw", raw_arr)?;
                let mut call_args = vec![tmpl_obj];
                call_args.extend(exprs);
                self.call_function(&func, &call_args)
            }
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        if let Expr::MemberAccess { object, property } = callee {
            return self.eval_member_call(object, property, args, true);
        }

        let func = self.eval_expr(callee)?;
        let arg_values = self.eval_call_args(args)?;

        self.call_function(&func, &arg_values)
    }
}

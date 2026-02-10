use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::string_methods;
use crate::runtime::value::symbol::JsSymbol;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn get_property(
        &mut self,
        obj_val: &JsValue,
        key: &str,
    ) -> Result<JsValue, RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                let mut current = Some(*obj);
                while let Some(candidate) = current {
                    let (prop, next) = {
                        let borrowed = candidate.borrow();
                        (borrowed.properties.get(key).cloned(), borrowed.prototype)
                    };
                    if let Some(prop) = prop {
                        if let Some(getter) = prop.getter {
                            return self.call_function_with_this(
                                &getter,
                                &[],
                                Some(obj_val.clone()),
                            );
                        }
                        return Ok(prop.value.clone());
                    }
                    current = next;
                }
                Ok(JsValue::Undefined)
            }
            JsValue::Array(arr) => {
                let borrowed = arr.borrow();
                if key == "length" {
                    return Ok(JsValue::Number(borrowed.len() as f64));
                }
                if let Ok(idx) = key.parse::<usize>() {
                    return Ok(borrowed.get(idx));
                }
                Ok(JsValue::Undefined)
            }
            JsValue::String(s) => string_methods::resolve_string_property(s, key),
            JsValue::Map(map) => {
                if key == "size" {
                    Ok(JsValue::Number(map.borrow().size() as f64))
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::Set(set) => {
                if key == "size" {
                    Ok(JsValue::Number(set.borrow().size() as f64))
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            JsValue::Proxy(proxy) => {
                let (trap, target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("get"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    self.call_function(&trap_fn, &[target, JsValue::String(key.to_string())])
                } else {
                    self.get_property(&target, key)
                }
            }
            JsValue::Function {
                properties,
                name,
                params,
                ..
            } => {
                if key == "name" {
                    return Ok(JsValue::String(name.clone()));
                }
                if key == "length" {
                    return Ok(JsValue::Number(params.len() as f64));
                }
                if let Some(props) = properties {
                    let borrowed = props.borrow();
                    if let Some(prop) = borrowed.properties.get(key) {
                        return Ok(prop.value.clone());
                    }
                }
                Ok(JsValue::Undefined)
            }
            JsValue::NativeFunction { name, .. } => {
                if key == "name" {
                    return Ok(JsValue::String(name.clone()));
                }
                Ok(JsValue::Undefined)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("cannot access property '{key}' on {obj_val}"),
            }),
        }
    }

    pub(crate) fn set_property(
        &mut self,
        obj_val: &JsValue,
        key: &str,
        value: JsValue,
    ) -> Result<(), RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                let mut current = Some(*obj);
                while let Some(candidate) = current {
                    let (prop, next) = {
                        let borrowed = candidate.borrow();
                        (borrowed.properties.get(key).cloned(), borrowed.prototype)
                    };
                    if let Some(prop) = prop {
                        if let Some(setter) = prop.setter {
                            self.call_function_with_this(
                                &setter,
                                std::slice::from_ref(&value),
                                Some(obj_val.clone()),
                            )?;
                            return Ok(());
                        }
                        break;
                    }
                    current = next;
                }

                obj.borrow_mut().set(key.to_string(), value);
                Ok(())
            }
            JsValue::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    arr.borrow_mut().set(idx, value);
                    Ok(())
                } else {
                    Err(RuntimeError::TypeError {
                        message: format!("cannot set property '{key}' on array"),
                    })
                }
            }
            JsValue::Proxy(proxy) => {
                let (trap, target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("set"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    self.call_function(
                        &trap_fn,
                        &[target, JsValue::String(key.to_string()), value],
                    )?;
                    Ok(())
                } else {
                    self.set_property(&target, key, value)
                }
            }
            JsValue::Function { properties, .. } => {
                if let Some(props) = properties {
                    props.borrow_mut().set(key.to_string(), value);
                }
                Ok(())
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("cannot set property '{key}' on {obj_val}"),
            }),
        }
    }

    pub(crate) fn get_symbol_property(
        &mut self,
        obj_val: &JsValue,
        sym: &JsSymbol,
    ) -> Result<JsValue, RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                let mut current = Some(*obj);
                while let Some(candidate) = current {
                    let (val, next) = {
                        let borrowed = candidate.borrow();
                        (borrowed.get_symbol(sym), borrowed.prototype)
                    };
                    if let Some(v) = val {
                        return Ok(v);
                    }
                    current = next;
                }
                Ok(JsValue::Undefined)
            }
            _ => Ok(JsValue::Undefined),
        }
    }

    pub(crate) fn set_symbol_property(
        &mut self,
        obj_val: &JsValue,
        sym: &JsSymbol,
        value: JsValue,
    ) -> Result<(), RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                obj.borrow_mut().set_symbol(sym.clone(), value);
                Ok(())
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("cannot set symbol property on {obj_val}"),
            }),
        }
    }

    pub(crate) fn delete_property(
        &mut self,
        obj_val: &JsValue,
        key: &str,
    ) -> Result<JsValue, RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                let removed = obj.borrow_mut().properties.remove(key).is_some();
                Ok(JsValue::Boolean(removed))
            }
            JsValue::Proxy(proxy) => {
                let (trap, target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("deleteProperty"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    let result =
                        self.call_function(&trap_fn, &[target, JsValue::String(key.to_string())])?;
                    Ok(JsValue::Boolean(result.to_boolean()))
                } else {
                    self.delete_property(&target, key)
                }
            }
            _ => Ok(JsValue::Boolean(true)),
        }
    }

    pub(crate) fn eval_delete_expr(&mut self, operand: &Expr) -> Result<JsValue, RuntimeError> {
        match operand {
            Expr::MemberAccess { object, property } => {
                let obj_val = self.eval_expr(object)?;
                self.delete_property(&obj_val, property)
            }
            Expr::ComputedMemberAccess { object, property } => {
                let obj_val = self.eval_expr(object)?;
                let key = self.eval_expr(property)?.to_js_string();
                self.delete_property(&obj_val, &key)
            }
            _ => Ok(JsValue::Boolean(true)),
        }
    }
}

use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::value::string_methods;
use crate::runtime::value::symbol::JsSymbol;
use crate::runtime::value::JsValue;
use std::rc::Rc;

impl Interpreter {
    pub(crate) fn get_property(
        &mut self,
        obj_val: &JsValue,
        key: &str,
    ) -> Result<JsValue, RuntimeError> {
        match obj_val {
            JsValue::Object(obj) => {
                let mut current = Some(Rc::clone(obj));
                while let Some(candidate) = current {
                    let (prop, next) = {
                        let borrowed = candidate.borrow();
                        (
                            borrowed.properties.get(key).cloned(),
                            borrowed.prototype.clone(),
                        )
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
                let mut current = Some(Rc::clone(obj));
                while let Some(candidate) = current {
                    let (prop, next) = {
                        let borrowed = candidate.borrow();
                        (
                            borrowed.properties.get(key).cloned(),
                            borrowed.prototype.clone(),
                        )
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
                let mut current = Some(Rc::clone(obj));
                while let Some(candidate) = current {
                    let (val, next) = {
                        let borrowed = candidate.borrow();
                        (borrowed.get_symbol(sym), borrowed.prototype.clone())
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
}

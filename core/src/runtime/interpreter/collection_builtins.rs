use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::value::collections::map::JsMap;
use crate::runtime::value::collections::set::JsSet;
use crate::runtime::value::collections::weak_map::{is_valid_weak_key, JsWeakMap};
use crate::runtime::value::collections::weak_set::JsWeakSet;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_new_map(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut map = JsMap::new();
        if let Some(arg_expr) = args.first() {
            let iterable = self.eval_expr(arg_expr)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                let entries = self.collect_iterable(&iterable)?;
                for entry in entries {
                    let key = self.get_property(&entry, "0")?;
                    let value = self.get_property(&entry, "1")?;
                    map.set(key, value);
                }
            }
        }
        Ok(JsValue::Map(map.wrapped()))
    }

    pub(crate) fn eval_new_set(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut set = JsSet::new();
        if let Some(arg_expr) = args.first() {
            let iterable = self.eval_expr(arg_expr)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                let elements = self.collect_iterable(&iterable)?;
                for elem in elements {
                    set.add(elem);
                }
            }
        }
        Ok(JsValue::Set(set.wrapped()))
    }

    pub(crate) fn call_map_method(
        &mut self,
        map_rc: &std::rc::Rc<std::cell::RefCell<JsMap>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "set" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                let value = args.get(1).cloned().unwrap_or(JsValue::Undefined);
                map_rc.borrow_mut().set(key, value);
                Ok(JsValue::Map(map_rc.clone()))
            }
            "get" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(map_rc.borrow().get(&key))
            }
            "has" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(map_rc.borrow().has(&key)))
            }
            "delete" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(map_rc.borrow_mut().delete(&key)))
            }
            "clear" => {
                map_rc.borrow_mut().clear();
                Ok(JsValue::Undefined)
            }
            "keys" => {
                let keys: Vec<JsValue> = map_rc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect();
                Ok(self.create_array_iterator(keys))
            }
            "values" => {
                let values: Vec<JsValue> = map_rc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(_, v)| v.clone())
                    .collect();
                Ok(self.create_array_iterator(values))
            }
            "entries" => {
                let entries: Vec<JsValue> = map_rc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(k, v)| {
                        JsValue::Array(
                            crate::runtime::value::array::JsArray::new(vec![k.clone(), v.clone()])
                                .wrapped(),
                        )
                    })
                    .collect();
                Ok(self.create_array_iterator(entries))
            }
            "forEach" => {
                let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
                let entries: Vec<(JsValue, JsValue)> = map_rc.borrow().entries.clone();
                for (key, value) in entries {
                    self.call_function(&callback, &[value, key])?;
                }
                Ok(JsValue::Undefined)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("map.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn call_set_method(
        &mut self,
        set_rc: &std::rc::Rc<std::cell::RefCell<JsSet>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "add" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                set_rc.borrow_mut().add(value);
                Ok(JsValue::Set(set_rc.clone()))
            }
            "has" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(set_rc.borrow().has(&value)))
            }
            "delete" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(set_rc.borrow_mut().delete(&value)))
            }
            "clear" => {
                set_rc.borrow_mut().clear();
                Ok(JsValue::Undefined)
            }
            "keys" | "values" => {
                let values: Vec<JsValue> = set_rc.borrow().entries.clone();
                Ok(self.create_array_iterator(values))
            }
            "entries" => {
                let entries: Vec<JsValue> = set_rc
                    .borrow()
                    .entries
                    .iter()
                    .map(|v| {
                        JsValue::Array(
                            crate::runtime::value::array::JsArray::new(vec![v.clone(), v.clone()])
                                .wrapped(),
                        )
                    })
                    .collect();
                Ok(self.create_array_iterator(entries))
            }
            "forEach" => {
                let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
                let entries: Vec<JsValue> = set_rc.borrow().entries.clone();
                for value in entries {
                    self.call_function(&callback, &[value.clone(), value])?;
                }
                Ok(JsValue::Undefined)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("set.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn eval_new_weak_map(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut wm = JsWeakMap::new();
        if let Some(arg_expr) = args.first() {
            let iterable = self.eval_expr(arg_expr)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                let entries = self.collect_iterable(&iterable)?;
                for entry in entries {
                    let key = self.get_property(&entry, "0")?;
                    if !is_valid_weak_key(&key) {
                        return Err(RuntimeError::TypeError {
                            message: "Invalid value used as weak map key".to_string(),
                        });
                    }
                    let value = self.get_property(&entry, "1")?;
                    wm.set(key, value);
                }
            }
        }
        Ok(JsValue::WeakMap(wm.wrapped()))
    }

    pub(crate) fn eval_new_weak_set(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut ws = JsWeakSet::new();
        if let Some(arg_expr) = args.first() {
            let iterable = self.eval_expr(arg_expr)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                let elements = self.collect_iterable(&iterable)?;
                for elem in elements {
                    if !is_valid_weak_key(&elem) {
                        return Err(RuntimeError::TypeError {
                            message: "Invalid value used as weak set value".to_string(),
                        });
                    }
                    ws.add(elem);
                }
            }
        }
        Ok(JsValue::WeakSet(ws.wrapped()))
    }

    pub(crate) fn call_weak_map_method(
        &mut self,
        wm_rc: &std::rc::Rc<std::cell::RefCell<JsWeakMap>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "set" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                if !is_valid_weak_key(&key) {
                    return Err(RuntimeError::TypeError {
                        message: "Invalid value used as weak map key".to_string(),
                    });
                }
                let value = args.get(1).cloned().unwrap_or(JsValue::Undefined);
                wm_rc.borrow_mut().set(key, value);
                Ok(JsValue::WeakMap(wm_rc.clone()))
            }
            "get" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(wm_rc.borrow().get(&key))
            }
            "has" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(wm_rc.borrow().has(&key)))
            }
            "delete" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(wm_rc.borrow_mut().delete(&key)))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("weakMap.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn call_weak_set_method(
        &mut self,
        ws_rc: &std::rc::Rc<std::cell::RefCell<JsWeakSet>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "add" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                if !is_valid_weak_key(&value) {
                    return Err(RuntimeError::TypeError {
                        message: "Invalid value used as weak set value".to_string(),
                    });
                }
                ws_rc.borrow_mut().add(value);
                Ok(JsValue::WeakSet(ws_rc.clone()))
            }
            "has" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(ws_rc.borrow().has(&value)))
            }
            "delete" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(ws_rc.borrow_mut().delete(&value)))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("weakSet.{method} is not a function"),
            }),
        }
    }

    fn create_array_iterator(&mut self, items: Vec<JsValue>) -> JsValue {
        use crate::runtime::value::generator::JsGenerator;
        use crate::runtime::value::object::JsObject;
        use crate::runtime::value::symbol;

        let gen_state = JsGenerator::from_values(items.into());
        let gen_rc = gen_state.wrapped();

        let mut obj = JsObject::new();
        obj.set(
            "next".to_string(),
            JsValue::NativeFunction {
                name: "next".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorNext(gen_rc.clone()),
            },
        );
        obj.set(
            "return".to_string(),
            JsValue::NativeFunction {
                name: "return".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorReturn(gen_rc),
            },
        );

        let iter_sym = symbol::symbol_iterator();
        let obj_rc = obj.wrapped();
        obj_rc.borrow_mut().set_symbol(
            iter_sym,
            JsValue::NativeFunction {
                name: "[Symbol.iterator]".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorIterator,
            },
        );

        JsValue::Object(obj_rc)
    }
}

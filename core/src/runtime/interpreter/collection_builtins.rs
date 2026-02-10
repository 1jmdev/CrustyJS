use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::gc::{Gc, GcCell};
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
        Ok(JsValue::Map(self.heap.alloc_cell(map)))
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
        Ok(JsValue::Set(self.heap.alloc_cell(set)))
    }

    pub(crate) fn call_map_method(
        &mut self,
        map_gc: &Gc<GcCell<JsMap>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "set" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                let value = args.get(1).cloned().unwrap_or(JsValue::Undefined);
                map_gc.borrow_mut().set(key, value);
                Ok(JsValue::Map(*map_gc))
            }
            "get" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(map_gc.borrow().get(&key))
            }
            "has" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(map_gc.borrow().has(&key)))
            }
            "delete" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(map_gc.borrow_mut().delete(&key)))
            }
            "clear" => {
                map_gc.borrow_mut().clear();
                Ok(JsValue::Undefined)
            }
            "keys" => {
                let keys: Vec<JsValue> = map_gc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect();
                Ok(self.create_array_iterator(keys))
            }
            "values" => {
                let values: Vec<JsValue> = map_gc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(_, v)| v.clone())
                    .collect();
                Ok(self.create_array_iterator(values))
            }
            "entries" => {
                let pairs: Vec<Vec<JsValue>> = map_gc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(k, v)| vec![k.clone(), v.clone()])
                    .collect();
                let entries: Vec<JsValue> = pairs
                    .into_iter()
                    .map(|pair| {
                        JsValue::Array(
                            self.heap
                                .alloc_cell(crate::runtime::value::array::JsArray::new(pair)),
                        )
                    })
                    .collect();
                Ok(self.create_array_iterator(entries))
            }
            "forEach" => {
                let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
                let entries: Vec<(JsValue, JsValue)> = map_gc.borrow().entries.clone();
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
        set_gc: &Gc<GcCell<JsSet>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "add" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                set_gc.borrow_mut().add(value);
                Ok(JsValue::Set(*set_gc))
            }
            "has" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(set_gc.borrow().has(&value)))
            }
            "delete" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(set_gc.borrow_mut().delete(&value)))
            }
            "clear" => {
                set_gc.borrow_mut().clear();
                Ok(JsValue::Undefined)
            }
            "keys" | "values" => {
                let values: Vec<JsValue> = set_gc.borrow().entries.clone();
                Ok(self.create_array_iterator(values))
            }
            "entries" => {
                let raw: Vec<JsValue> = set_gc.borrow().entries.clone();
                let entries: Vec<JsValue> = raw
                    .into_iter()
                    .map(|v| {
                        JsValue::Array(self.heap.alloc_cell(
                            crate::runtime::value::array::JsArray::new(vec![v.clone(), v]),
                        ))
                    })
                    .collect();
                Ok(self.create_array_iterator(entries))
            }
            "forEach" => {
                let callback = args.first().cloned().unwrap_or(JsValue::Undefined);
                let entries: Vec<JsValue> = set_gc.borrow().entries.clone();
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
        Ok(JsValue::WeakMap(self.heap.alloc_cell(wm)))
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
        Ok(JsValue::WeakSet(self.heap.alloc_cell(ws)))
    }

    pub(crate) fn call_weak_map_method(
        &mut self,
        wm_gc: &Gc<GcCell<JsWeakMap>>,
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
                wm_gc.borrow_mut().set(key, value);
                Ok(JsValue::WeakMap(*wm_gc))
            }
            "get" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(wm_gc.borrow().get(&key))
            }
            "has" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(wm_gc.borrow().has(&key)))
            }
            "delete" => {
                let key = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(wm_gc.borrow_mut().delete(&key)))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("weakMap.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn call_weak_set_method(
        &mut self,
        ws_gc: &Gc<GcCell<JsWeakSet>>,
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
                ws_gc.borrow_mut().add(value);
                Ok(JsValue::WeakSet(*ws_gc))
            }
            "has" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(ws_gc.borrow().has(&value)))
            }
            "delete" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::Boolean(ws_gc.borrow_mut().delete(&value)))
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
        let gen_gc = self.heap.alloc_cell(gen_state);

        let mut obj = JsObject::new();
        obj.set(
            "next".to_string(),
            JsValue::NativeFunction {
                name: "next".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorNext(gen_gc),
            },
        );
        obj.set(
            "return".to_string(),
            JsValue::NativeFunction {
                name: "return".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorReturn(gen_gc),
            },
        );

        let iter_sym = symbol::symbol_iterator();
        let obj_gc = self.heap.alloc_cell(obj);
        obj_gc.borrow_mut().set_symbol(
            iter_sym,
            JsValue::NativeFunction {
                name: "[Symbol.iterator]".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorIterator,
            },
        );

        JsValue::Object(obj_gc)
    }
}

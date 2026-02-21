use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::collections::map::JsMap;
use crate::runtime::value::collections::set::JsSet;
use crate::runtime::value::collections::weak_map::{extract_weak_key, JsWeakMap};
use crate::runtime::value::collections::weak_set::JsWeakSet;
use crate::runtime::value::generator::JsGenerator;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::symbol;
use crate::runtime::value::{JsValue, NativeFunction};

impl Interpreter {
    pub(crate) fn eval_new_map(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut map = JsMap::new();
        if let Some(arg) = args.first() {
            let iterable = self.eval_expr(arg)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                for entry in self.collect_iterable(&iterable)? {
                    let k = self.get_property(&entry, "0")?;
                    let v = self.get_property(&entry, "1")?;
                    map.set(k, v);
                }
            }
        }
        Ok(JsValue::Map(self.heap.alloc_cell(map)))
    }

    pub(crate) fn eval_new_set(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut set = JsSet::new();
        if let Some(arg) = args.first() {
            let iterable = self.eval_expr(arg)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                for elem in self.collect_iterable(&iterable)? {
                    set.add(elem);
                }
            }
        }
        Ok(JsValue::Set(self.heap.alloc_cell(set)))
    }

    pub(crate) fn eval_new_weak_map(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut wm = JsWeakMap::new();
        if let Some(arg) = args.first() {
            let iterable = self.eval_expr(arg)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                for entry in self.collect_iterable(&iterable)? {
                    let key = self.get_property(&entry, "0")?;
                    let erased = extract_weak_key(&key).ok_or_else(|| RuntimeError::TypeError {
                        message: "Invalid value used as weak map key".into(),
                    })?;
                    wm.set(erased, self.get_property(&entry, "1")?);
                }
            }
        }
        Ok(JsValue::WeakMap(self.heap.alloc_cell(wm)))
    }

    pub(crate) fn eval_new_weak_set(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let mut ws = JsWeakSet::new();
        if let Some(arg) = args.first() {
            let iterable = self.eval_expr(arg)?;
            if !matches!(iterable, JsValue::Null | JsValue::Undefined) {
                for elem in self.collect_iterable(&iterable)? {
                    let erased =
                        extract_weak_key(&elem).ok_or_else(|| RuntimeError::TypeError {
                            message: "Invalid value used as weak set value".into(),
                        })?;
                    ws.add(erased);
                }
            }
        }
        Ok(JsValue::WeakSet(self.heap.alloc_cell(ws)))
    }

    pub(crate) fn call_map_method(
        &mut self,
        map_gc: &Gc<GcCell<JsMap>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let arg0 = || args.first().cloned().unwrap_or(JsValue::Undefined);
        let arg1 = || args.get(1).cloned().unwrap_or(JsValue::Undefined);
        match method {
            "set" => {
                map_gc.borrow_mut().set(arg0(), arg1());
                Ok(JsValue::Map(*map_gc))
            }
            "get" => Ok(map_gc.borrow().get(&arg0())),
            "has" => Ok(JsValue::Boolean(map_gc.borrow().has(&arg0()))),
            "delete" => Ok(JsValue::Boolean(map_gc.borrow_mut().delete(&arg0()))),
            "clear" => {
                map_gc.borrow_mut().clear();
                Ok(JsValue::Undefined)
            }
            "size" => Ok(JsValue::Number(map_gc.borrow().entries.len() as f64)),
            "keys" => {
                let keys: Vec<JsValue> = map_gc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(k, _)| k.clone())
                    .collect();
                Ok(self.make_iterator(keys))
            }
            "values" => {
                let vals: Vec<JsValue> = map_gc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(_, v)| v.clone())
                    .collect();
                Ok(self.make_iterator(vals))
            }
            "entries" => {
                let pairs: Vec<JsValue> = map_gc
                    .borrow()
                    .entries
                    .iter()
                    .map(|(k, v)| {
                        JsValue::Array(
                            self.heap
                                .alloc_cell(JsArray::new(vec![k.clone(), v.clone()])),
                        )
                    })
                    .collect();
                Ok(self.make_iterator(pairs))
            }
            "forEach" => {
                let cb = arg0();
                let entries: Vec<(JsValue, JsValue)> = map_gc.borrow().entries.clone();
                for (k, v) in entries {
                    self.call_function(&cb, &[v, k])?;
                }
                Ok(JsValue::Undefined)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Map.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn call_set_method(
        &mut self,
        set_gc: &Gc<GcCell<JsSet>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let arg0 = || args.first().cloned().unwrap_or(JsValue::Undefined);
        match method {
            "add" => {
                set_gc.borrow_mut().add(arg0());
                Ok(JsValue::Set(*set_gc))
            }
            "has" => Ok(JsValue::Boolean(set_gc.borrow().has(&arg0()))),
            "delete" => Ok(JsValue::Boolean(set_gc.borrow_mut().delete(&arg0()))),
            "clear" => {
                set_gc.borrow_mut().clear();
                Ok(JsValue::Undefined)
            }
            "size" => Ok(JsValue::Number(set_gc.borrow().entries.len() as f64)),
            "keys" | "values" => {
                let vals: Vec<JsValue> = set_gc.borrow().entries.clone();
                Ok(self.make_iterator(vals))
            }
            "entries" => {
                let raw: Vec<JsValue> = set_gc.borrow().entries.clone();
                let pairs: Vec<JsValue> = raw
                    .into_iter()
                    .map(|v| JsValue::Array(self.heap.alloc_cell(JsArray::new(vec![v.clone(), v]))))
                    .collect();
                Ok(self.make_iterator(pairs))
            }
            "forEach" => {
                let cb = arg0();
                let entries: Vec<JsValue> = set_gc.borrow().entries.clone();
                for v in entries {
                    self.call_function(&cb, &[v.clone(), v])?;
                }
                Ok(JsValue::Undefined)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Set.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn call_weak_map_method(
        &mut self,
        wm_gc: &Gc<GcCell<JsWeakMap>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let arg0 = || args.first().cloned().unwrap_or(JsValue::Undefined);
        match method {
            "set" => {
                let erased = extract_weak_key(&arg0()).ok_or_else(|| RuntimeError::TypeError {
                    message: "Invalid value used as weak map key".into(),
                })?;
                wm_gc
                    .borrow_mut()
                    .set(erased, args.get(1).cloned().unwrap_or(JsValue::Undefined));
                Ok(JsValue::WeakMap(*wm_gc))
            }
            "get" => Ok(extract_weak_key(&arg0())
                .map(|e| wm_gc.borrow().get(e))
                .unwrap_or(JsValue::Undefined)),
            "has" => Ok(JsValue::Boolean(
                extract_weak_key(&arg0())
                    .map(|e| wm_gc.borrow().has(e))
                    .unwrap_or(false),
            )),
            "delete" => Ok(JsValue::Boolean(
                extract_weak_key(&arg0())
                    .map(|e| wm_gc.borrow_mut().delete(e))
                    .unwrap_or(false),
            )),
            _ => Err(RuntimeError::TypeError {
                message: format!("WeakMap.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn call_weak_set_method(
        &mut self,
        ws_gc: &Gc<GcCell<JsWeakSet>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let arg0 = || args.first().cloned().unwrap_or(JsValue::Undefined);
        match method {
            "add" => {
                let erased = extract_weak_key(&arg0()).ok_or_else(|| RuntimeError::TypeError {
                    message: "Invalid value used as weak set value".into(),
                })?;
                ws_gc.borrow_mut().add(erased);
                Ok(JsValue::WeakSet(*ws_gc))
            }
            "has" => Ok(JsValue::Boolean(
                extract_weak_key(&arg0())
                    .map(|e| ws_gc.borrow().has(e))
                    .unwrap_or(false),
            )),
            "delete" => Ok(JsValue::Boolean(
                extract_weak_key(&arg0())
                    .map(|e| ws_gc.borrow_mut().delete(e))
                    .unwrap_or(false),
            )),
            _ => Err(RuntimeError::TypeError {
                message: format!("WeakSet.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn make_iterator(&mut self, items: Vec<JsValue>) -> JsValue {
        let gen_gc = self.heap.alloc_cell(JsGenerator::from_values(items.into()));
        let iter_sym = symbol::symbol_iterator();
        let mut obj = JsObject::new();
        obj.set(
            "next".into(),
            JsValue::NativeFunction {
                name: "next".into(),
                handler: NativeFunction::GeneratorNext(gen_gc),
            },
        );
        obj.set(
            "return".into(),
            JsValue::NativeFunction {
                name: "return".into(),
                handler: NativeFunction::GeneratorReturn(gen_gc),
            },
        );
        let obj_gc = self.heap.alloc_cell(obj);
        obj_gc.borrow_mut().set_symbol(
            iter_sym,
            JsValue::NativeFunction {
                name: "[Symbol.iterator]".into(),
                handler: NativeFunction::GeneratorIterator,
            },
        );
        JsValue::Object(obj_gc)
    }
}

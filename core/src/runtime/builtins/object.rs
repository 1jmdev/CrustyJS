use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn builtin_object_static(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "create" => {
                let proto = args.first().cloned().unwrap_or(JsValue::Null);
                let mut obj = JsObject::new();
                obj.prototype = match proto {
                    JsValue::Object(p) => Some(p),
                    JsValue::Null | JsValue::Undefined => None,
                    _ => {
                        return Err(RuntimeError::TypeError {
                            message: "Object.create: prototype must be object or null".into(),
                        })
                    }
                };
                Ok(JsValue::Object(self.heap.alloc_cell(obj)))
            }
            "keys" => {
                let keys =
                    self.object_own_keys(args.first().cloned().unwrap_or(JsValue::Undefined))?;
                let arr = JsArray::new(keys.into_iter().map(JsValue::String).collect());
                Ok(JsValue::Array(self.heap.alloc_cell(arr)))
            }
            "values" => {
                let obj = args.first().cloned().unwrap_or(JsValue::Undefined);
                let keys = self.object_own_keys(obj.clone())?;
                let mut vals = Vec::with_capacity(keys.len());
                for k in &keys {
                    vals.push(self.get_property(&obj, k)?);
                }
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(vals))))
            }
            "entries" => {
                let obj = args.first().cloned().unwrap_or(JsValue::Undefined);
                let keys = self.object_own_keys(obj.clone())?;
                let mut pairs = Vec::with_capacity(keys.len());
                for k in &keys {
                    let v = self.get_property(&obj, k)?;
                    let pair = JsArray::new(vec![JsValue::String(k.clone()), v]);
                    pairs.push(JsValue::Array(self.heap.alloc_cell(pair)));
                }
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(pairs))))
            }
            "assign" => {
                let target = args
                    .first()
                    .cloned()
                    .unwrap_or_else(|| JsValue::Object(self.heap.alloc_cell(JsObject::new())));
                let JsValue::Object(target_obj) = target.clone() else {
                    return Err(RuntimeError::TypeError {
                        message: "Object.assign: target must be an object".into(),
                    });
                };
                for source in args.iter().skip(1) {
                    if let JsValue::Object(src) = source {
                        let props: Vec<(String, JsValue)> = src
                            .borrow()
                            .properties
                            .iter()
                            .map(|(k, p)| (k.clone(), p.value.clone()))
                            .collect();
                        for (k, v) in props {
                            target_obj.borrow_mut().set(k, v);
                        }
                    }
                }
                Ok(target)
            }
            "getPrototypeOf" => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.object_get_prototype_of(&val)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Object.{method} is not a function"),
            }),
        }
    }

    /// Returns own enumerable string-keyed property names, respecting Proxy ownKeys trap.
    pub(crate) fn object_own_keys(&mut self, value: JsValue) -> Result<Vec<String>, RuntimeError> {
        match &value {
            JsValue::Object(obj) => Ok(obj.borrow().properties.keys().cloned().collect()),
            JsValue::Proxy(proxy) => {
                let (trap, target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("ownKeys"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    let result = self.call_function(&trap_fn, &[target])?;
                    if let JsValue::Array(arr) = result {
                        Ok(arr
                            .borrow()
                            .elements
                            .iter()
                            .map(|v| v.to_js_string())
                            .collect())
                    } else {
                        Err(RuntimeError::TypeError {
                            message: "Proxy ownKeys trap must return an array".into(),
                        })
                    }
                } else {
                    self.object_own_keys(target)
                }
            }
            _ => Ok(Vec::new()),
        }
    }

    pub(crate) fn object_get_prototype_of(
        &mut self,
        value: &JsValue,
    ) -> Result<JsValue, RuntimeError> {
        match value {
            JsValue::Object(obj) => Ok(match obj.borrow().prototype {
                Some(proto) => JsValue::Object(proto),
                None => JsValue::Null,
            }),
            JsValue::Proxy(proxy) => {
                let (trap, target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("getPrototypeOf"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    self.call_function(&trap_fn, &[target])
                } else {
                    self.object_get_prototype_of(&target)
                }
            }
            _ => Ok(JsValue::Null),
        }
    }
}

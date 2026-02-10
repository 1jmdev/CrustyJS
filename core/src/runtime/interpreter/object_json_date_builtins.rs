use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::gc::Gc;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

impl Interpreter {
    pub(crate) fn builtin_object_static_call(
        &mut self,
        property: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match property {
            "keys" => {
                let keys = self.object_keys(args.first().cloned().unwrap_or(JsValue::Undefined))?;
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(
                    keys.into_iter().map(JsValue::String).collect(),
                ))))
            }
            "values" => {
                let values =
                    self.object_values(args.first().cloned().unwrap_or(JsValue::Undefined))?;
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(values))))
            }
            "entries" => {
                let entries =
                    self.object_entries(args.first().cloned().unwrap_or(JsValue::Undefined))?;
                let pairs: Vec<(String, JsValue)> = entries;
                let pair_values: Vec<JsValue> = pairs
                    .into_iter()
                    .map(|(k, v)| {
                        JsValue::Array(
                            self.heap
                                .alloc_cell(JsArray::new(vec![JsValue::String(k), v])),
                        )
                    })
                    .collect();
                Ok(JsValue::Array(
                    self.heap.alloc_cell(JsArray::new(pair_values)),
                ))
            }
            "assign" => {
                let target = args
                    .first()
                    .cloned()
                    .unwrap_or_else(|| JsValue::Object(self.heap.alloc_cell(JsObject::new())));
                let JsValue::Object(target_obj) = target.clone() else {
                    return Err(RuntimeError::TypeError {
                        message: "Object.assign target must be object".to_string(),
                    });
                };
                for source in args.iter().skip(1) {
                    if let JsValue::Object(source_obj) = source {
                        let source_borrowed = source_obj.borrow();
                        for (k, p) in &source_borrowed.properties {
                            target_obj.borrow_mut().set(k.clone(), p.value.clone());
                        }
                    }
                }
                Ok(target)
            }
            "getPrototypeOf" => {
                let target = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.object_get_prototype_of(&target)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Object has no static method '{property}'"),
            }),
        }
    }

    pub(crate) fn builtin_json_call(
        &mut self,
        property: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match property {
            "stringify" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                let mut seen = HashSet::new();
                let json = self.to_json_value(&value, &mut seen)?;
                Ok(JsValue::String(json.to_string()))
            }
            "parse" => {
                let input = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Undefined)
                    .to_js_string();
                let parsed: JsonValue =
                    serde_json::from_str(&input).map_err(|e| RuntimeError::TypeError {
                        message: format!("JSON.parse failed: {e}"),
                    })?;
                Ok(self.convert_json_value(&parsed))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("JSON has no method '{property}'"),
            }),
        }
    }

    pub(crate) fn builtin_date_call(
        &self,
        property: &str,
        _args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match property {
            "now" => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as f64)
                    .unwrap_or(0.0);
                Ok(JsValue::Number(now))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("Date has no static method '{property}'"),
            }),
        }
    }

    pub(crate) fn object_keys(&mut self, value: JsValue) -> Result<Vec<String>, RuntimeError> {
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
                            message: "ownKeys trap must return an array".to_string(),
                        })
                    }
                } else {
                    self.object_keys(target)
                }
            }
            _ => Ok(Vec::new()),
        }
    }

    fn object_values(&mut self, value: JsValue) -> Result<Vec<JsValue>, RuntimeError> {
        let keys = self.object_keys(value.clone())?;
        let mut values = Vec::new();
        for key in &keys {
            values.push(self.get_property(&value, key)?);
        }
        Ok(values)
    }

    fn object_entries(&mut self, value: JsValue) -> Result<Vec<(String, JsValue)>, RuntimeError> {
        let keys = self.object_keys(value.clone())?;
        let mut entries = Vec::new();
        for key in &keys {
            let val = self.get_property(&value, key)?;
            entries.push((key.clone(), val));
        }
        Ok(entries)
    }

    pub(crate) fn object_get_prototype_of(
        &mut self,
        value: &JsValue,
    ) -> Result<JsValue, RuntimeError> {
        match value {
            JsValue::Object(obj) => match obj.borrow().prototype {
                Some(proto) => Ok(JsValue::Object(proto)),
                None => Ok(JsValue::Null),
            },
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

    fn to_json_value(
        &self,
        value: &JsValue,
        seen: &mut HashSet<usize>,
    ) -> Result<JsonValue, RuntimeError> {
        Ok(match value {
            JsValue::Undefined => JsonValue::Null,
            JsValue::Null => JsonValue::Null,
            JsValue::Boolean(b) => JsonValue::Bool(*b),
            JsValue::Number(n) => serde_json::Number::from_f64(*n)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            JsValue::String(s) => JsonValue::String(s.clone()),
            JsValue::Function { .. } => JsonValue::Null,
            JsValue::NativeFunction { .. } => JsonValue::Null,
            JsValue::Array(arr) => {
                let ptr = Gc::as_usize(*arr);
                if !seen.insert(ptr) {
                    return Err(RuntimeError::TypeError {
                        message: "Converting circular structure to JSON".to_string(),
                    });
                }
                let elements = arr.borrow().elements.clone();
                let mut out = Vec::new();
                for el in &elements {
                    out.push(self.to_json_value(el, seen)?);
                }
                seen.remove(&ptr);
                JsonValue::Array(out)
            }
            JsValue::Object(obj) => {
                let ptr = Gc::as_usize(*obj);
                if !seen.insert(ptr) {
                    return Err(RuntimeError::TypeError {
                        message: "Converting circular structure to JSON".to_string(),
                    });
                }
                let borrowed = obj.borrow();
                let mut map = serde_json::Map::new();
                for (k, p) in &borrowed.properties {
                    map.insert(k.clone(), self.to_json_value(&p.value, seen)?);
                }
                seen.remove(&ptr);
                JsonValue::Object(map)
            }
            JsValue::Promise(_) => JsonValue::Null,
            JsValue::Symbol(_) => JsonValue::Null,
            JsValue::Map(_) => JsonValue::Null,
            JsValue::Set(_) => JsonValue::Null,
            JsValue::WeakMap(_) => JsonValue::Null,
            JsValue::WeakSet(_) => JsonValue::Null,
            JsValue::RegExp(re) => JsonValue::Object({
                let mut map = serde_json::Map::new();
                let re = re.borrow();
                map.insert("source".to_string(), JsonValue::String(re.pattern.clone()));
                map.insert("flags".to_string(), JsonValue::String(re.flag_string()));
                map
            }),
            JsValue::Proxy(_) => JsonValue::Object(serde_json::Map::new()),
        })
    }

    fn convert_json_value(&mut self, value: &JsonValue) -> JsValue {
        match value {
            JsonValue::Null => JsValue::Null,
            JsonValue::Bool(b) => JsValue::Boolean(*b),
            JsonValue::Number(n) => JsValue::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::String(s) => JsValue::String(s.clone()),
            JsonValue::Array(items) => {
                let elements: Vec<JsValue> =
                    items.iter().map(|v| self.convert_json_value(v)).collect();
                JsValue::Array(self.heap.alloc_cell(JsArray::new(elements)))
            }
            JsonValue::Object(map) => {
                let mut obj = JsObject::new();
                for (k, v) in map {
                    obj.set(k.clone(), self.convert_json_value(v));
                }
                JsValue::Object(self.heap.alloc_cell(obj))
            }
        }
    }
}

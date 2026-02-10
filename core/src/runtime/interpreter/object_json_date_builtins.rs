use super::Interpreter;
use crate::errors::RuntimeError;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

impl Interpreter {
    pub(crate) fn builtin_object_static_call(
        &self,
        property: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match property {
            "keys" => {
                let keys = self.object_keys(args.first().cloned().unwrap_or(JsValue::Undefined));
                Ok(JsValue::Array(
                    JsArray::new(keys.into_iter().map(JsValue::String).collect()).wrapped(),
                ))
            }
            "values" => {
                let values =
                    self.object_values(args.first().cloned().unwrap_or(JsValue::Undefined));
                Ok(JsValue::Array(JsArray::new(values).wrapped()))
            }
            "entries" => {
                let entries =
                    self.object_entries(args.first().cloned().unwrap_or(JsValue::Undefined));
                let pairs = entries
                    .into_iter()
                    .map(|(k, v)| {
                        JsValue::Array(JsArray::new(vec![JsValue::String(k), v]).wrapped())
                    })
                    .collect();
                Ok(JsValue::Array(JsArray::new(pairs).wrapped()))
            }
            "assign" => {
                let target = args
                    .first()
                    .cloned()
                    .unwrap_or(JsValue::Object(JsObject::new().wrapped()));
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
            _ => Err(RuntimeError::TypeError {
                message: format!("Object has no static method '{property}'"),
            }),
        }
    }

    pub(crate) fn builtin_json_call(
        &self,
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
                Ok(self.from_json_value(&parsed))
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

    fn object_keys(&self, value: JsValue) -> Vec<String> {
        match value {
            JsValue::Object(obj) => obj.borrow().properties.keys().cloned().collect(),
            _ => Vec::new(),
        }
    }

    fn object_values(&self, value: JsValue) -> Vec<JsValue> {
        match value {
            JsValue::Object(obj) => obj
                .borrow()
                .properties
                .values()
                .map(|p| p.value.clone())
                .collect(),
            _ => Vec::new(),
        }
    }

    fn object_entries(&self, value: JsValue) -> Vec<(String, JsValue)> {
        match value {
            JsValue::Object(obj) => obj
                .borrow()
                .properties
                .iter()
                .map(|(k, p)| (k.clone(), p.value.clone()))
                .collect(),
            _ => Vec::new(),
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
            JsValue::Array(arr) => {
                let ptr = std::rc::Rc::as_ptr(arr) as usize;
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
                let ptr = std::rc::Rc::as_ptr(obj) as usize;
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
        })
    }

    fn from_json_value(&self, value: &JsonValue) -> JsValue {
        match value {
            JsonValue::Null => JsValue::Null,
            JsonValue::Bool(b) => JsValue::Boolean(*b),
            JsonValue::Number(n) => JsValue::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::String(s) => JsValue::String(s.clone()),
            JsonValue::Array(items) => JsValue::Array(
                JsArray::new(items.iter().map(|v| self.from_json_value(v)).collect()).wrapped(),
            ),
            JsonValue::Object(map) => {
                let mut obj = JsObject::new();
                for (k, v) in map {
                    obj.set(k.clone(), self.from_json_value(v));
                }
                JsValue::Object(obj.wrapped())
            }
        }
    }
}

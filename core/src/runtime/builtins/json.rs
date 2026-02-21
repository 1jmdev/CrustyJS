use crate::errors::RuntimeError;
use crate::runtime::gc::Gc;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;
use serde_json::Value as JsonValue;
use std::collections::HashSet;

impl Interpreter {
    pub(crate) fn builtin_json_call(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
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
                message: format!("JSON.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn to_json_value(
        &self,
        value: &JsValue,
        seen: &mut HashSet<usize>,
    ) -> Result<JsonValue, RuntimeError> {
        Ok(match value {
            JsValue::Undefined | JsValue::Null => JsonValue::Null,
            JsValue::Boolean(b) => JsonValue::Bool(*b),
            JsValue::Number(n) => serde_json::Number::from_f64(*n)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            JsValue::String(s) => JsonValue::String(s.clone()),
            JsValue::Function { .. } | JsValue::NativeFunction { .. } => JsonValue::Null,
            JsValue::Symbol(_)
            | JsValue::Promise(_)
            | JsValue::Map(_)
            | JsValue::Set(_)
            | JsValue::WeakMap(_)
            | JsValue::WeakSet(_) => JsonValue::Null,
            JsValue::Array(arr) => {
                let ptr = Gc::as_usize(*arr);
                if !seen.insert(ptr) {
                    return Err(RuntimeError::TypeError {
                        message: "Converting circular structure to JSON".into(),
                    });
                }
                let elements = arr.borrow().elements.clone();
                let out = elements
                    .iter()
                    .map(|el| self.to_json_value(el, seen))
                    .collect::<Result<Vec<_>, _>>()?;
                seen.remove(&ptr);
                JsonValue::Array(out)
            }
            JsValue::Object(obj) => {
                let ptr = Gc::as_usize(*obj);
                if !seen.insert(ptr) {
                    return Err(RuntimeError::TypeError {
                        message: "Converting circular structure to JSON".into(),
                    });
                }
                let mut map = serde_json::Map::new();
                for (k, p) in &obj.borrow().properties {
                    map.insert(k.clone(), self.to_json_value(&p.value, seen)?);
                }
                seen.remove(&ptr);
                JsonValue::Object(map)
            }
            JsValue::RegExp(re) => {
                let re = re.borrow();
                let mut map = serde_json::Map::new();
                map.insert("source".into(), JsonValue::String(re.pattern.clone()));
                map.insert("flags".into(), JsonValue::String(re.flag_string()));
                JsonValue::Object(map)
            }
            JsValue::Proxy(_) => JsonValue::Object(serde_json::Map::new()),
        })
    }

    pub(crate) fn from_json_value(&mut self, value: &JsonValue) -> JsValue {
        match value {
            JsonValue::Null => JsValue::Null,
            JsonValue::Bool(b) => JsValue::Boolean(*b),
            JsonValue::Number(n) => JsValue::Number(n.as_f64().unwrap_or(0.0)),
            JsonValue::String(s) => JsValue::String(s.clone()),
            JsonValue::Array(items) => {
                let elements: Vec<JsValue> =
                    items.iter().map(|v| self.from_json_value(v)).collect();
                JsValue::Array(self.heap.alloc_cell(JsArray::new(elements)))
            }
            JsonValue::Object(map) => {
                let mut obj = JsObject::new();
                for (k, v) in map {
                    obj.set(k.clone(), self.from_json_value(v));
                }
                JsValue::Object(self.heap.alloc_cell(obj))
            }
        }
    }
}

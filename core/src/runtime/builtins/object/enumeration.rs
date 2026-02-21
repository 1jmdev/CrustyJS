use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn object_keys(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let keys = self.object_own_keys(args.first().cloned().unwrap_or(JsValue::Undefined))?;
        Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(
            keys.into_iter().map(JsValue::String).collect(),
        ))))
    }

    pub(crate) fn object_values(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let obj = args.first().cloned().unwrap_or(JsValue::Undefined);
        let keys = self.object_own_keys(obj.clone())?;
        let mut values = Vec::with_capacity(keys.len());
        for key in keys {
            values.push(self.get_property(&obj, &key)?);
        }

        Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(values))))
    }

    pub(crate) fn object_entries(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let obj = args.first().cloned().unwrap_or(JsValue::Undefined);
        let keys = self.object_own_keys(obj.clone())?;
        let mut pairs = Vec::with_capacity(keys.len());
        for key in keys {
            let value = self.get_property(&obj, &key)?;
            let pair = JsArray::new(vec![JsValue::String(key), value]);
            pairs.push(JsValue::Array(self.heap.alloc_cell(pair)));
        }

        Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(pairs))))
    }

    /// Returns own enumerable string-keyed property names, respecting Proxy ownKeys trap.
    pub(crate) fn object_own_keys(&mut self, value: JsValue) -> Result<Vec<String>, RuntimeError> {
        match &value {
            JsValue::Object(obj) => Ok(obj
                .borrow()
                .properties
                .iter()
                .filter_map(|(k, p)| p.enumerable.then_some(k.clone()))
                .collect()),
            JsValue::Array(arr) => Ok((0..arr.borrow().len()).map(|i| i.to_string()).collect()),
            JsValue::Function { properties, .. } => Ok(properties
                .as_ref()
                .map(|props| {
                    props
                        .borrow()
                        .properties
                        .iter()
                        .filter_map(|(k, p)| p.enumerable.then_some(k.clone()))
                        .collect()
                })
                .unwrap_or_default()),
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

    pub(crate) fn object_get_all_own_keys(
        &mut self,
        value: JsValue,
    ) -> Result<Vec<String>, RuntimeError> {
        match &value {
            JsValue::Object(obj) => Ok(obj.borrow().properties.keys().cloned().collect()),
            JsValue::Array(arr) => {
                let mut keys: Vec<String> =
                    (0..arr.borrow().len()).map(|i| i.to_string()).collect();
                keys.push("length".to_string());
                Ok(keys)
            }
            JsValue::Function { properties, .. } => {
                let mut keys = vec!["name".to_string(), "length".to_string()];
                if let Some(props) = properties {
                    keys.extend(props.borrow().properties.keys().cloned());
                }
                Ok(keys)
            }
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
                    self.object_get_all_own_keys(target)
                }
            }
            _ => Ok(Vec::new()),
        }
    }
}

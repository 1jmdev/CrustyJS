use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn object_create(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let proto = args.first().cloned().unwrap_or(JsValue::Null);
        let mut obj = JsObject::new();
        obj.prototype = match proto {
            JsValue::Object(p) => Some(p),
            JsValue::Null | JsValue::Undefined => None,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Object.create: prototype must be object or null".into(),
                });
            }
        };
        Ok(JsValue::Object(self.heap.alloc_cell(obj)))
    }

    pub(crate) fn object_set_prototype_of(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let proto = args.get(1).cloned().unwrap_or(JsValue::Null);

        let next_proto = match proto {
            JsValue::Object(p) => Some(p),
            JsValue::Null | JsValue::Undefined => None,
            _ => {
                return Err(RuntimeError::TypeError {
                    message: "Object.setPrototypeOf: prototype must be object or null".into(),
                });
            }
        };

        match &target {
            JsValue::Object(obj) => {
                obj.borrow_mut().set_prototype(next_proto);
                Ok(target)
            }
            JsValue::Proxy(proxy) => {
                let (trap, proxied_target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("setPrototypeOf"), p.target.clone())
                };

                if let Some(trap_fn) = trap {
                    let trap_proto = match next_proto {
                        Some(p) => JsValue::Object(p),
                        None => JsValue::Null,
                    };
                    self.call_function(&trap_fn, &[proxied_target.clone(), trap_proto])?;
                    Ok(target)
                } else {
                    self.object_set_prototype_of(&[
                        proxied_target,
                        match next_proto {
                            Some(p) => JsValue::Object(p),
                            None => JsValue::Null,
                        },
                    ])
                }
            }
            _ => Err(RuntimeError::TypeError {
                message: "Object.setPrototypeOf: target must be an object".into(),
            }),
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

    pub(crate) fn call_object_prototype_method(
        &mut self,
        receiver: &JsValue,
        method: &str,
        args: &[JsValue],
    ) -> Option<Result<JsValue, RuntimeError>> {
        let result = match method {
            "hasOwnProperty" => self.object_proto_has_own_property(receiver, args),
            "isPrototypeOf" => self.object_proto_is_prototype_of(receiver, args),
            "propertyIsEnumerable" => self.object_proto_property_is_enumerable(receiver, args),
            "toLocaleString" => self.object_proto_to_locale_string(receiver),
            "toString" => self.object_proto_to_string(receiver),
            "valueOf" => self.object_proto_value_of(receiver),
            _ => return None,
        };
        Some(result)
    }

    fn object_proto_has_own_property(
        &mut self,
        receiver: &JsValue,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let key = args
            .first()
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();
        Ok(JsValue::Boolean(
            self.object_has_own_named_property(receiver, &key),
        ))
    }

    fn object_proto_property_is_enumerable(
        &mut self,
        receiver: &JsValue,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let key = args
            .first()
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();
        let is_enum = match receiver {
            JsValue::Object(obj) => obj
                .borrow()
                .properties
                .get(&key)
                .map(|p| p.enumerable)
                .unwrap_or(false),
            JsValue::Array(arr) => key
                .parse::<usize>()
                .ok()
                .map(|idx| idx < arr.borrow().len())
                .unwrap_or(false),
            _ => false,
        };
        Ok(JsValue::Boolean(is_enum))
    }

    fn object_proto_is_prototype_of(
        &mut self,
        receiver: &JsValue,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let JsValue::Object(proto_candidate) = receiver else {
            return Ok(JsValue::Boolean(false));
        };

        let mut current = match args.first().cloned().unwrap_or(JsValue::Undefined) {
            JsValue::Object(obj) => obj.borrow().prototype,
            JsValue::Proxy(proxy) => {
                let p = proxy.borrow();
                p.check_revoked()
                    .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                match &p.target {
                    JsValue::Object(obj) => obj.borrow().prototype,
                    _ => None,
                }
            }
            _ => None,
        };

        while let Some(candidate) = current {
            if crate::runtime::gc::Gc::ptr_eq(candidate, *proto_candidate) {
                return Ok(JsValue::Boolean(true));
            }
            current = candidate.borrow().prototype;
        }

        Ok(JsValue::Boolean(false))
    }

    fn object_proto_value_of(&mut self, receiver: &JsValue) -> Result<JsValue, RuntimeError> {
        Ok(receiver.clone())
    }

    fn object_proto_to_string(&mut self, receiver: &JsValue) -> Result<JsValue, RuntimeError> {
        let tag = match receiver {
            JsValue::Array(_) => "Array",
            JsValue::Function { .. } | JsValue::NativeFunction { .. } => "Function",
            JsValue::Map(_) => "Map",
            JsValue::Set(_) => "Set",
            JsValue::WeakMap(_) => "WeakMap",
            JsValue::WeakSet(_) => "WeakSet",
            JsValue::RegExp(_) => "RegExp",
            JsValue::Promise(_) => "Promise",
            JsValue::Object(_) | JsValue::Proxy(_) => "Object",
            JsValue::Null => "Null",
            JsValue::Undefined => "Undefined",
            _ => "Object",
        };
        Ok(JsValue::String(format!("[object {tag}]")))
    }

    fn object_proto_to_locale_string(
        &mut self,
        receiver: &JsValue,
    ) -> Result<JsValue, RuntimeError> {
        self.object_proto_to_string(receiver)
    }

    pub(crate) fn object_has_own_named_property(&self, target: &JsValue, key: &str) -> bool {
        match target {
            JsValue::Object(obj) => obj.borrow().properties.contains_key(key),
            JsValue::Array(arr) => {
                if key == "length" {
                    return true;
                }
                key.parse::<usize>()
                    .ok()
                    .map(|idx| idx < arr.borrow().len())
                    .unwrap_or(false)
            }
            JsValue::Function { properties, .. } => match key {
                "name" => true,
                "length" => true,
                _ => properties
                    .as_ref()
                    .map(|p| p.borrow().properties.contains_key(key))
                    .unwrap_or(false),
            },
            _ => false,
        }
    }
}

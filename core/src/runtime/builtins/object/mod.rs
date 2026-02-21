mod descriptor;
mod enumeration;
mod prototype;

use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn builtin_object_static(
        &mut self,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "assign" => self.object_assign(args),
            "create" => self.object_create(args),
            "defineProperties" => self.object_define_properties(args),
            "defineProperty" => self.object_define_property(args),
            "entries" => self.object_entries(args),
            "freeze" => self.object_freeze(args),
            "seal" => self.object_seal(args),
            "preventExtensions" => self.object_prevent_extensions(args),
            "fromEntries" => self.object_from_entries(args),
            "getOwnPropertyDescriptor" => self.object_get_own_property_descriptor(args),
            "getOwnPropertyDescriptors" => self.object_get_own_property_descriptors(args),
            "getOwnPropertyNames" => self.object_get_own_property_names(args),
            "getOwnPropertySymbols" => self.object_get_own_property_symbols(args),
            "getPrototypeOf" => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                self.object_get_prototype_of(&val)
            }
            "hasOwn" => self.object_has_own(args),
            "is" => self.object_is(args),
            "isExtensible" => self.object_is_extensible(args),
            "isFrozen" => self.object_is_frozen(args),
            "isSealed" => self.object_is_sealed(args),
            "keys" => self.object_keys(args),
            "setPrototypeOf" => self.object_set_prototype_of(args),
            "values" => self.object_values(args),
            _ => Err(RuntimeError::TypeError {
                message: format!("Object.{method} is not a function"),
            }),
        }
    }

    fn object_assign(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
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
            let keys = self.object_own_keys(source.clone())?;
            for key in keys {
                let value = self.get_property(source, &key)?;
                target_obj.borrow_mut().set(key, value);
            }
        }

        Ok(target)
    }

    fn object_has_own(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let key = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();

        match target {
            JsValue::Object(_) | JsValue::Array(_) | JsValue::Function { .. } => Ok(
                JsValue::Boolean(self.object_has_own_named_property(&target, &key)),
            ),
            JsValue::Proxy(_) => {
                let keys = self.object_get_all_own_keys(target)?;
                Ok(JsValue::Boolean(keys.iter().any(|k| k == &key)))
            }
            _ => Ok(JsValue::Boolean(false)),
        }
    }

    fn object_is(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let left = args.first().cloned().unwrap_or(JsValue::Undefined);
        let right = args.get(1).cloned().unwrap_or(JsValue::Undefined);
        let result = same_value(&left, &right);
        Ok(JsValue::Boolean(result))
    }

    fn object_from_entries(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let entries = args.first().cloned().unwrap_or(JsValue::Undefined);
        let JsValue::Array(arr) = entries else {
            return Err(RuntimeError::TypeError {
                message: "Object.fromEntries currently expects an array".into(),
            });
        };

        let out = self.heap.alloc_cell(JsObject::new());
        let elements = arr.borrow().elements.clone();
        for entry in elements {
            let JsValue::Array(pair) = entry else {
                return Err(RuntimeError::TypeError {
                    message: "Object.fromEntries entry must be a [key, value] pair".into(),
                });
            };

            let pair_elements = pair.borrow().elements.clone();
            let key = pair_elements
                .first()
                .cloned()
                .unwrap_or(JsValue::Undefined)
                .to_js_string();
            let value = pair_elements.get(1).cloned().unwrap_or(JsValue::Undefined);
            out.borrow_mut().set(key, value);
        }

        Ok(JsValue::Object(out))
    }

    fn object_is_extensible(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let result = match target {
            JsValue::Object(obj) => obj.borrow().extensible,
            JsValue::Proxy(proxy) => {
                let p = proxy.borrow();
                p.check_revoked()
                    .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                matches!(p.target, JsValue::Object(_))
            }
            _ => false,
        };
        Ok(JsValue::Boolean(result))
    }

    fn object_prevent_extensions(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        if let JsValue::Object(obj) = &target {
            obj.borrow_mut().prevent_extensions();
        }
        Ok(target)
    }

    fn object_seal(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        if let JsValue::Object(obj) = &target {
            obj.borrow_mut().seal();
        }
        Ok(target)
    }

    fn object_freeze(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        if let JsValue::Object(obj) = &target {
            obj.borrow_mut().freeze();
        }
        Ok(target)
    }

    fn object_is_sealed(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let result = match target {
            JsValue::Object(obj) => obj.borrow().sealed,
            _ => false,
        };
        Ok(JsValue::Boolean(result))
    }

    fn object_is_frozen(&mut self, args: &[JsValue]) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let result = match target {
            JsValue::Object(obj) => obj.borrow().frozen,
            _ => false,
        };
        Ok(JsValue::Boolean(result))
    }
}

fn same_value(left: &JsValue, right: &JsValue) -> bool {
    match (left, right) {
        (JsValue::Number(a), JsValue::Number(b)) => {
            if a.is_nan() && b.is_nan() {
                true
            } else if *a == 0.0 && *b == 0.0 {
                a.is_sign_positive() == b.is_sign_positive()
            } else {
                a == b
            }
        }
        _ => left == right,
    }
}

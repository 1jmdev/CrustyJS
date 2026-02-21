use crate::errors::RuntimeError;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn object_get_own_property_names(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let keys =
            self.object_get_all_own_keys(args.first().cloned().unwrap_or(JsValue::Undefined))?;
        Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(
            keys.into_iter().map(JsValue::String).collect(),
        ))))
    }

    pub(crate) fn object_get_own_property_symbols(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let symbols = match target {
            JsValue::Object(obj) => obj
                .borrow()
                .symbol_properties
                .values()
                .map(|(sym, _)| JsValue::Symbol(sym.clone()))
                .collect(),
            _ => Vec::new(),
        };

        Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(symbols))))
    }

    pub(crate) fn object_get_own_property_descriptor(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let key = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();

        let descriptor = match target {
            JsValue::Object(obj) => {
                let maybe_prop = obj.borrow().properties.get(&key).cloned();
                match maybe_prop {
                    Some(prop) => self.object_descriptor_to_js_object(&prop),
                    None => JsValue::Undefined,
                }
            }
            JsValue::Array(arr) => {
                if key == "length" {
                    let mut p = crate::runtime::value::object::Property::new(JsValue::Number(
                        arr.borrow().len() as f64,
                    ));
                    p.writable = true;
                    p.enumerable = false;
                    p.configurable = false;
                    self.object_descriptor_to_js_object(&p)
                } else if let Ok(idx) = key.parse::<usize>() {
                    if idx < arr.borrow().len() {
                        self.object_descriptor_to_js_object(
                            &crate::runtime::value::object::Property::new(arr.borrow().get(idx)),
                        )
                    } else {
                        JsValue::Undefined
                    }
                } else {
                    JsValue::Undefined
                }
            }
            _ => JsValue::Undefined,
        };

        Ok(descriptor)
    }

    pub(crate) fn object_get_own_property_descriptors(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let out = self.heap.alloc_cell(JsObject::new());

        if let JsValue::Object(obj) = target {
            let props: Vec<(String, crate::runtime::value::object::Property)> = obj
                .borrow()
                .properties
                .iter()
                .map(|(k, p)| (k.clone(), p.clone()))
                .collect();

            for (name, prop) in props {
                out.borrow_mut()
                    .set(name, self.object_descriptor_to_js_object(&prop));
            }
        }

        Ok(JsValue::Object(out))
    }

    pub(crate) fn object_define_property(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let key = args
            .get(1)
            .cloned()
            .unwrap_or(JsValue::Undefined)
            .to_js_string();
        let descriptor = args.get(2).cloned().unwrap_or(JsValue::Undefined);

        let JsValue::Object(target_obj) = &target else {
            return Err(RuntimeError::TypeError {
                message: "Object.defineProperty: target must be an object".into(),
            });
        };
        let JsValue::Object(descriptor_obj) = descriptor else {
            return Err(RuntimeError::TypeError {
                message: "Object.defineProperty: descriptor must be an object".into(),
            });
        };

        let desc_value = self.get_property(&JsValue::Object(descriptor_obj), "value")?;
        let desc_get = self.get_property(&JsValue::Object(descriptor_obj), "get")?;
        let desc_set = self.get_property(&JsValue::Object(descriptor_obj), "set")?;
        let desc_writable = self.get_property(&JsValue::Object(descriptor_obj), "writable")?;
        let desc_enumerable = self.get_property(&JsValue::Object(descriptor_obj), "enumerable")?;
        let desc_configurable =
            self.get_property(&JsValue::Object(descriptor_obj), "configurable")?;

        {
            let mut target_ref = target_obj.borrow_mut();
            let mut prop = target_ref.properties.get(&key).cloned().unwrap_or_else(|| {
                crate::runtime::value::object::Property::new(JsValue::Undefined)
            });

            if !matches!(desc_value, JsValue::Undefined) {
                prop.value = desc_value;
            }
            if !matches!(desc_get, JsValue::Undefined) {
                prop.getter = Some(desc_get);
                prop.writable = false;
            }
            if !matches!(desc_set, JsValue::Undefined) {
                prop.setter = Some(desc_set);
                prop.writable = false;
            }
            if !matches!(desc_writable, JsValue::Undefined) {
                prop.writable = desc_writable.to_boolean();
            }
            if !matches!(desc_enumerable, JsValue::Undefined) {
                prop.enumerable = desc_enumerable.to_boolean();
            }
            if !matches!(desc_configurable, JsValue::Undefined) {
                prop.configurable = desc_configurable.to_boolean();
            }

            target_ref.properties.insert(key, prop);
        }

        Ok(target)
    }

    pub(crate) fn object_define_properties(
        &mut self,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let target = args.first().cloned().unwrap_or(JsValue::Undefined);
        let descriptors = args.get(1).cloned().unwrap_or(JsValue::Undefined);

        let JsValue::Object(descriptor_map) = descriptors else {
            return Err(RuntimeError::TypeError {
                message: "Object.defineProperties: descriptors must be an object".into(),
            });
        };

        let entries: Vec<(String, JsValue)> = descriptor_map
            .borrow()
            .properties
            .iter()
            .map(|(k, p)| (k.clone(), p.value.clone()))
            .collect();

        for (key, descriptor) in entries {
            self.object_define_property(&[target.clone(), JsValue::String(key), descriptor])?;
        }

        Ok(target)
    }

    fn object_descriptor_to_js_object(
        &mut self,
        prop: &crate::runtime::value::object::Property,
    ) -> JsValue {
        let descriptor = self.heap.alloc_cell(JsObject::new());
        {
            let mut obj = descriptor.borrow_mut();
            obj.set(
                "configurable".to_string(),
                JsValue::Boolean(prop.configurable),
            );
            obj.set("enumerable".to_string(), JsValue::Boolean(prop.enumerable));
            obj.set("writable".to_string(), JsValue::Boolean(prop.writable));
            obj.set("value".to_string(), prop.value.clone());

            if let Some(get) = &prop.getter {
                obj.set("get".to_string(), get.clone());
            }
            if let Some(set) = &prop.setter {
                obj.set("set".to_string(), set.clone());
            }
        }

        JsValue::Object(descriptor)
    }
}

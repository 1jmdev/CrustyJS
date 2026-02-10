use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Pattern;
use crate::runtime::environment::BindingKind;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::JsValue;
use std::collections::HashSet;

impl Interpreter {
    pub(crate) fn eval_pattern_binding(
        &mut self,
        pattern: &Pattern,
        value: JsValue,
    ) -> Result<(), RuntimeError> {
        self.eval_pattern_binding_with_kind(pattern, value, BindingKind::Let)
    }

    pub(crate) fn eval_pattern_binding_with_kind(
        &mut self,
        pattern: &Pattern,
        value: JsValue,
        kind: BindingKind,
    ) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Identifier(name) => {
                self.env.define_with_kind(name.clone(), value, kind);
                Ok(())
            }
            Pattern::ArrayPattern { elements } => {
                let source = match value {
                    JsValue::Array(arr) => arr.borrow().elements.clone(),
                    JsValue::Undefined | JsValue::Null => {
                        return Err(RuntimeError::TypeError {
                            message: "cannot destructure array from nullish value".to_string(),
                        });
                    }
                    _ => Vec::new(),
                };

                let mut idx = 0usize;
                for elem in elements {
                    match elem {
                        None => {
                            idx += 1;
                        }
                        Some(Pattern::Rest(inner)) => {
                            let rest = if idx >= source.len() {
                                Vec::new()
                            } else {
                                source[idx..].to_vec()
                            };
                            self.eval_pattern_binding_with_kind(
                                inner,
                                JsValue::Array(JsArray::new(rest).wrapped()),
                                kind,
                            )?;
                            break;
                        }
                        Some(inner) => {
                            let val = source.get(idx).cloned().unwrap_or(JsValue::Undefined);
                            self.eval_pattern_binding_with_kind(inner, val, kind)?;
                            idx += 1;
                        }
                    }
                }

                Ok(())
            }
            Pattern::ObjectPattern { properties } => {
                let object = match value {
                    JsValue::Object(obj) => obj,
                    JsValue::Undefined | JsValue::Null => {
                        return Err(RuntimeError::TypeError {
                            message: "cannot destructure object from nullish value".to_string(),
                        });
                    }
                    _ => JsObject::new().wrapped(),
                };

                let mut used = HashSet::new();

                for prop in properties {
                    if prop.is_rest {
                        continue;
                    }

                    let mut prop_value =
                        object.borrow().get(&prop.key).unwrap_or(JsValue::Undefined);
                    if matches!(prop_value, JsValue::Undefined) {
                        if let Some(default) = &prop.default {
                            prop_value = self.eval_expr(default)?;
                        }
                    }

                    let target = prop
                        .alias
                        .as_ref()
                        .cloned()
                        .unwrap_or(Pattern::Identifier(prop.key.clone()));
                    self.eval_pattern_binding_with_kind(&target, prop_value, kind)?;
                    used.insert(prop.key.clone());
                }

                for prop in properties {
                    if !prop.is_rest {
                        continue;
                    }

                    let mut rest_obj = JsObject::new();
                    {
                        let borrowed = object.borrow();
                        for (k, property) in &borrowed.properties {
                            if !used.contains(k) {
                                rest_obj.set(k.clone(), property.value.clone());
                            }
                        }
                    }

                    let rest_target = match &prop.alias {
                        Some(Pattern::Rest(inner)) => inner,
                        Some(other) => other,
                        None => {
                            return Err(RuntimeError::TypeError {
                                message: "invalid object rest binding target".to_string(),
                            });
                        }
                    };

                    self.eval_pattern_binding_with_kind(
                        rest_target,
                        JsValue::Object(rest_obj.wrapped()),
                        kind,
                    )?;
                }

                Ok(())
            }
            Pattern::Rest(inner) => self.eval_pattern_binding_with_kind(inner, value, kind),
        }
    }
}

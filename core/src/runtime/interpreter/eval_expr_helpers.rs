use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::{AssignOp, BinOp, PropertyKey};
use crate::runtime::value::abstract_equals;
use crate::runtime::value::iterator::get_property_simple;
use crate::runtime::value::symbol;
use crate::runtime::value::JsValue;

fn to_int32(value: f64) -> i32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    let truncated = value.trunc();
    let int = truncated as i64;
    int as i32
}

impl Interpreter {
    pub(crate) fn eval_call_args(
        &mut self,
        args: &[crate::parser::ast::Expr],
    ) -> Result<Vec<JsValue>, RuntimeError> {
        let mut values = Vec::new();
        for arg in args {
            match arg {
                crate::parser::ast::Expr::Spread(inner) => {
                    let spread_val = self.eval_expr(inner)?;
                    values.extend(self.collect_iterable(&spread_val)?);
                }
                other => values.push(self.eval_expr(other)?),
            }
        }
        Ok(values)
    }

    pub(crate) fn eval_property_key(&mut self, key: &PropertyKey) -> Result<String, RuntimeError> {
        match key {
            PropertyKey::Identifier(name) => Ok(name.clone()),
            PropertyKey::Computed(expr) => Ok(self.eval_expr(expr)?.to_js_string()),
        }
    }

    pub(crate) fn collect_iterable(
        &mut self,
        value: &JsValue,
    ) -> Result<Vec<JsValue>, RuntimeError> {
        match value {
            JsValue::Array(arr) => Ok(arr.borrow().elements.clone()),
            JsValue::String(s) => Ok(s
                .chars()
                .map(|ch| JsValue::String(ch.to_string()))
                .collect()),
            JsValue::Map(map) => {
                let borrowed = map.borrow();
                let entries: Vec<Vec<JsValue>> = borrowed
                    .entries
                    .iter()
                    .map(|(k, v)| vec![k.clone(), v.clone()])
                    .collect::<Vec<_>>();
                drop(borrowed);
                let entries: Vec<JsValue> = entries
                    .into_iter()
                    .map(|pair| {
                        JsValue::Array(
                            self.heap
                                .alloc_cell(crate::runtime::value::array::JsArray::new(pair)),
                        )
                    })
                    .collect();
                Ok(entries)
            }
            JsValue::Set(set) => Ok(set.borrow().entries.clone()),
            JsValue::Object(obj) => {
                let iter_sym = symbol::symbol_iterator();
                let method = obj.borrow().get_symbol(&iter_sym);
                let Some(iter_fn) = method else {
                    return Err(RuntimeError::TypeError {
                        message: "object is not iterable".to_string(),
                    });
                };
                let iterator = self.call_function_with_this(&iter_fn, &[], Some(value.clone()))?;
                let mut results = Vec::new();
                loop {
                    let next_fn = get_property_simple(&iterator, "next").ok_or_else(|| {
                        RuntimeError::TypeError {
                            message: "iterator has no next method".to_string(),
                        }
                    })?;
                    let result =
                        self.call_function_with_this(&next_fn, &[], Some(iterator.clone()))?;
                    let done = get_property_simple(&result, "done")
                        .map(|v| v.to_boolean())
                        .unwrap_or(false);
                    if done {
                        break;
                    }
                    let val = get_property_simple(&result, "value").unwrap_or(JsValue::Undefined);
                    results.push(val);
                }
                Ok(results)
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("{value} is not iterable"),
            }),
        }
    }

    /// ToPrimitive: convert an object to a primitive value by calling
    /// valueOf() and toString() methods.
    /// `preferred_type`: "number" calls valueOf first, "string" calls toString first.
    /// "default" behaves like "number".
    pub(crate) fn to_primitive(
        &mut self,
        val: &JsValue,
        preferred_type: &str,
    ) -> Result<JsValue, RuntimeError> {
        match val {
            // Already primitive - return as-is
            JsValue::Undefined
            | JsValue::Null
            | JsValue::Boolean(_)
            | JsValue::Number(_)
            | JsValue::String(_)
            | JsValue::Symbol(_) => Ok(val.clone()),
            JsValue::Object(_) => {
                // Check [[PrimitiveValue]] first (wrapper objects)
                if let Some(prim) = val.get_primitive_value() {
                    return Ok(prim);
                }

                let try_methods = if preferred_type == "string" {
                    ["toString", "valueOf"]
                } else {
                    // "number" or "default"
                    ["valueOf", "toString"]
                };

                let mut has_method = false;
                for method_name in &try_methods {
                    let method = self.get_property(val, method_name)?;
                    if matches!(
                        method,
                        JsValue::Function { .. } | JsValue::NativeFunction { .. }
                    ) {
                        has_method = true;
                        let result =
                            self.call_function_with_this(&method, &[], Some(val.clone()))?;
                        // If result is primitive, return it
                        if !matches!(result, JsValue::Object(_) | JsValue::Array(_)) {
                            return Ok(result);
                        }
                    }
                }

                // If we found methods but none returned a primitive, throw TypeError
                if has_method {
                    return Err(self.throw_type_error("Cannot convert object to primitive value"));
                }

                // No valueOf/toString methods found (no prototype chain) - use defaults
                // Default: valueOf() returns `this` (not primitive), then toString() returns "[object Object]"
                Ok(JsValue::String("[object Object]".to_string()))
            }
            JsValue::Array(arr) => {
                // Arrays: ToPrimitive calls toString which joins elements
                let borrowed = arr.borrow();
                let items: Vec<String> =
                    borrowed.elements.iter().map(|v| v.to_js_string()).collect();
                Ok(JsValue::String(items.join(",")))
            }
            // For other types, just return as-is (they'll be coerced by to_number/to_js_string)
            _ => Ok(val.clone()),
        }
    }

    pub(crate) fn eval_binary(
        &mut self,
        lhs: JsValue,
        op: &BinOp,
        rhs: JsValue,
    ) -> Result<JsValue, RuntimeError> {
        // For addition, apply ToPrimitive first (with "default" hint)
        if matches!(op, BinOp::Add) {
            let lhs_prim = self.to_primitive(&lhs, "default")?;
            let rhs_prim = self.to_primitive(&rhs, "default")?;

            // If either side is a string after ToPrimitive, do string concatenation
            if matches!(&lhs_prim, JsValue::String(_)) || matches!(&rhs_prim, JsValue::String(_)) {
                let a = lhs_prim.to_js_string();
                let b = rhs_prim.to_js_string();
                return Ok(JsValue::String(format!("{a}{b}")));
            }

            let ln = lhs_prim.to_number();
            let rn = rhs_prim.to_number();
            return Ok(JsValue::Number(ln + rn));
        }

        // For comparison/arithmetic, apply ToPrimitive with "number" hint
        let lhs_prim = self.to_primitive(&lhs, "number")?;
        let rhs_prim = self.to_primitive(&rhs, "number")?;

        // Comparison operators need special handling for string comparison
        if matches!(
            op,
            BinOp::Less | BinOp::LessEq | BinOp::Greater | BinOp::GreaterEq
        ) {
            if matches!(&lhs_prim, JsValue::String(_)) && matches!(&rhs_prim, JsValue::String(_)) {
                let ls = lhs_prim.to_js_string();
                let rs = rhs_prim.to_js_string();
                return match op {
                    BinOp::Less => Ok(JsValue::Boolean(ls < rs)),
                    BinOp::LessEq => Ok(JsValue::Boolean(ls <= rs)),
                    BinOp::Greater => Ok(JsValue::Boolean(ls > rs)),
                    BinOp::GreaterEq => Ok(JsValue::Boolean(ls >= rs)),
                    _ => unreachable!(),
                };
            }
        }

        let ln = lhs_prim.to_number();
        let rn = rhs_prim.to_number();

        match op {
            BinOp::Add => unreachable!("handled above"),
            BinOp::Sub => Ok(JsValue::Number(ln - rn)),
            BinOp::Mul => Ok(JsValue::Number(ln * rn)),
            BinOp::Div => Ok(JsValue::Number(ln / rn)),
            BinOp::Mod => Ok(JsValue::Number(ln % rn)),
            BinOp::Less => Ok(JsValue::Boolean(ln < rn)),
            BinOp::LessEq => Ok(JsValue::Boolean(ln <= rn)),
            BinOp::Greater => Ok(JsValue::Boolean(ln > rn)),
            BinOp::GreaterEq => Ok(JsValue::Boolean(ln >= rn)),
            BinOp::BitAnd => Ok(JsValue::Number((to_int32(ln) & to_int32(rn)) as f64)),
            BinOp::EqEqEq => Ok(JsValue::Boolean(lhs == rhs)),
            BinOp::NotEqEq => Ok(JsValue::Boolean(lhs != rhs)),
            BinOp::EqEq => Ok(JsValue::Boolean(abstract_equals(&lhs, &rhs))),
            BinOp::NotEq => Ok(JsValue::Boolean(!abstract_equals(&lhs, &rhs))),
            BinOp::Instanceof => unreachable!("instanceof handled before eval_binary"),
            BinOp::In => unreachable!("in handled before eval_binary"),
        }
    }

    pub(crate) fn eval_compound(
        &mut self,
        lhs: JsValue,
        op: &AssignOp,
        rhs: JsValue,
    ) -> Result<JsValue, RuntimeError> {
        let bin = match op {
            AssignOp::Add => BinOp::Add,
            AssignOp::Sub => BinOp::Sub,
            AssignOp::Mul => BinOp::Mul,
            AssignOp::Div => BinOp::Div,
            AssignOp::Mod => BinOp::Mod,
        };
        self.eval_binary(lhs, &bin, rhs)
    }
}

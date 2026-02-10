use super::Interpreter;
use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::value::regexp::{JsRegExp, RegExpFlags};
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn call_regexp_method(
        &mut self,
        re: &Gc<GcCell<JsRegExp>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        match method {
            "test" => {
                let input = args.first().map(|v| v.to_js_string()).unwrap_or_default();
                let result = re.borrow_mut().test(&input);
                Ok(JsValue::Boolean(result))
            }
            "exec" => {
                let input = args.first().map(|v| v.to_js_string()).unwrap_or_default();
                match re.borrow_mut().exec(&input) {
                    Some(m) => {
                        let arr = exec_result_to_array(m);
                        Ok(JsValue::Array(self.heap.alloc_cell(arr)))
                    }
                    None => Ok(JsValue::Null),
                }
            }
            "toString" => {
                let re = re.borrow();
                Ok(JsValue::String(re.to_string()))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("regexp has no method '{method}'"),
            }),
        }
    }

    pub(crate) fn get_regexp_property(
        &self,
        re: &Gc<GcCell<JsRegExp>>,
        property: &str,
    ) -> Result<JsValue, RuntimeError> {
        let re = re.borrow();
        match property {
            "source" => Ok(JsValue::String(re.pattern.clone())),
            "flags" => Ok(JsValue::String(re.flag_string())),
            "global" => Ok(JsValue::Boolean(re.flags.global)),
            "ignoreCase" => Ok(JsValue::Boolean(re.flags.ignore_case)),
            "multiline" => Ok(JsValue::Boolean(re.flags.multiline)),
            "dotAll" => Ok(JsValue::Boolean(re.flags.dotall)),
            "unicode" => Ok(JsValue::Boolean(re.flags.unicode)),
            "sticky" => Ok(JsValue::Boolean(re.flags.sticky)),
            "lastIndex" => Ok(JsValue::Number(re.last_index as f64)),
            _ => Ok(JsValue::Undefined),
        }
    }

    pub(crate) fn eval_new_regexp(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let arg_values: Vec<JsValue> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<_, _>>()?;

        let (pattern, flags) = match arg_values.as_slice() {
            [JsValue::RegExp(re)] => {
                let re = re.borrow();
                (re.pattern.clone(), re.flag_string())
            }
            [JsValue::RegExp(re), flags_val] => {
                let re = re.borrow();
                (re.pattern.clone(), flags_val.to_js_string())
            }
            _ => {
                let pattern = arg_values
                    .first()
                    .map(|v| match v {
                        JsValue::Undefined => String::new(),
                        other => other.to_js_string(),
                    })
                    .unwrap_or_default();
                let flags = arg_values
                    .get(1)
                    .map(|v| v.to_js_string())
                    .unwrap_or_default();
                (pattern, flags)
            }
        };

        let re_flags =
            RegExpFlags::from_str(&flags).map_err(|e| RuntimeError::TypeError { message: e })?;
        let re = JsRegExp::new(&pattern, re_flags)
            .map_err(|e| RuntimeError::TypeError { message: e })?;
        Ok(JsValue::RegExp(self.heap.alloc_cell(re)))
    }
}

fn exec_result_to_array(
    m: crate::runtime::value::regexp::MatchResult,
) -> crate::runtime::value::array::JsArray {
    let mut items: Vec<JsValue> = m
        .captures
        .iter()
        .map(|c| match c {
            Some(s) => JsValue::String(s.clone()),
            None => JsValue::Undefined,
        })
        .collect();

    if items.is_empty() {
        items.push(JsValue::String(m.full_match));
    }

    crate::runtime::value::array::JsArray::new(items)
}

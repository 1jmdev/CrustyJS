use crate::errors::RuntimeError;
use crate::parser::ast::Expr;
use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::regexp::{JsRegExp, MatchResult, RegExpFlags};
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
                Ok(JsValue::Boolean(re.borrow_mut().test(&input)))
            }
            "exec" => {
                let input = args.first().map(|v| v.to_js_string()).unwrap_or_default();
                match re.borrow_mut().exec(&input) {
                    Some(m) => Ok(JsValue::Array(self.heap.alloc_cell(match_to_array(m)))),
                    None => Ok(JsValue::Null),
                }
            }
            "toString" => Ok(JsValue::String(re.borrow().to_string())),
            _ => Err(RuntimeError::TypeError {
                message: format!("RegExp.{method} is not a function"),
            }),
        }
    }

    pub(crate) fn get_regexp_property(
        &self,
        re: &Gc<GcCell<JsRegExp>>,
        prop: &str,
    ) -> Result<JsValue, RuntimeError> {
        let re = re.borrow();
        Ok(match prop {
            "source" => JsValue::String(re.pattern.clone()),
            "flags" => JsValue::String(re.flag_string()),
            "global" => JsValue::Boolean(re.flags.global),
            "ignoreCase" => JsValue::Boolean(re.flags.ignore_case),
            "multiline" => JsValue::Boolean(re.flags.multiline),
            "dotAll" => JsValue::Boolean(re.flags.dotall),
            "unicode" => JsValue::Boolean(re.flags.unicode),
            "sticky" => JsValue::Boolean(re.flags.sticky),
            "lastIndex" => JsValue::Number(re.last_index as f64),
            _ => JsValue::Undefined,
        })
    }

    pub(crate) fn eval_new_regexp(&mut self, args: &[Expr]) -> Result<JsValue, RuntimeError> {
        let vals: Vec<JsValue> = args
            .iter()
            .map(|a| self.eval_expr(a))
            .collect::<Result<_, _>>()?;

        let (pattern, flags_str) = match vals.as_slice() {
            [JsValue::RegExp(re)] => {
                let re = re.borrow();
                (re.pattern.clone(), re.flag_string())
            }
            [JsValue::RegExp(re), f] => (re.borrow().pattern.clone(), f.to_js_string()),
            _ => {
                let p = vals
                    .first()
                    .map(|v| match v {
                        JsValue::Undefined => String::new(),
                        other => other.to_js_string(),
                    })
                    .unwrap_or_default();
                let f = vals.get(1).map(|v| v.to_js_string()).unwrap_or_default();
                (p, f)
            }
        };

        let flags = RegExpFlags::from_str(&flags_str)
            .map_err(|e| RuntimeError::TypeError { message: e })?;
        let re =
            JsRegExp::new(&pattern, flags).map_err(|e| RuntimeError::TypeError { message: e })?;
        Ok(JsValue::RegExp(self.heap.alloc_cell(re)))
    }
}

fn match_to_array(m: MatchResult) -> JsArray {
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
    JsArray::new(items)
}

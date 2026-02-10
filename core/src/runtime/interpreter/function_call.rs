use super::Interpreter;
use crate::diagnostics::stack_trace::CallFrame;
use crate::errors::RuntimeError;
use crate::runtime::value::array::JsArray;
use crate::runtime::value::generator::JsGenerator;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::symbol;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_array_callback_method(
        &mut self,
        arr: &std::rc::Rc<std::cell::RefCell<JsArray>>,
        method: &str,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let callback = args.first().ok_or_else(|| RuntimeError::TypeError {
            message: format!("{method} requires a callback argument"),
        })?;
        let elements = arr.borrow().elements.clone();

        match method {
            "map" => {
                let mut result = Vec::new();
                for elem in &elements {
                    let val = self.call_function(callback, std::slice::from_ref(elem))?;
                    result.push(val);
                }
                Ok(JsValue::Array(JsArray::new(result).wrapped()))
            }
            "filter" => {
                let mut result = Vec::new();
                for elem in &elements {
                    let val = self.call_function(callback, std::slice::from_ref(elem))?;
                    if val.to_boolean() {
                        result.push(elem.clone());
                    }
                }
                Ok(JsValue::Array(JsArray::new(result).wrapped()))
            }
            "forEach" => {
                for elem in &elements {
                    self.call_function(callback, std::slice::from_ref(elem))?;
                }
                Ok(JsValue::Undefined)
            }
            "reduce" => {
                let init = args.get(1).cloned().unwrap_or(JsValue::Undefined);
                let mut acc = init;
                for elem in &elements {
                    acc = self.call_function(callback, &[acc, elem.clone()])?;
                }
                Ok(acc)
            }
            "sort" => {
                let mut sorted = arr.borrow().elements.clone();
                if matches!(callback, JsValue::Undefined) {
                    sorted.sort_by_key(|a| a.to_js_string());
                } else {
                    sorted.sort_by(|a, b| {
                        let res = self
                            .call_function(callback, &[a.clone(), b.clone()])
                            .ok()
                            .map(|v| v.to_number())
                            .unwrap_or(0.0);
                        if res < 0.0 {
                            std::cmp::Ordering::Less
                        } else if res > 0.0 {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });
                }
                arr.borrow_mut().elements = sorted.clone();
                Ok(JsValue::Array(JsArray::new(sorted).wrapped()))
            }
            _ => Err(RuntimeError::TypeError {
                message: format!("array has no method '{method}'"),
            }),
        }
    }

    pub(crate) fn call_function(
        &mut self,
        func: &JsValue,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        self.call_function_with_this(func, args, None)
    }

    pub(crate) fn call_function_with_this(
        &mut self,
        func: &JsValue,
        args: &[JsValue],
        this_binding: Option<JsValue>,
    ) -> Result<JsValue, RuntimeError> {
        match func {
            JsValue::Function {
                name,
                params,
                body,
                closure_env,
                is_async,
                is_generator,
                source_path,
                source_offset,
            } => {
                if *is_generator {
                    return self.create_generator_object(
                        params,
                        body,
                        closure_env,
                        this_binding,
                        args,
                    );
                }

                let file = source_path
                    .clone()
                    .or_else(|| self.module_stack.last().map(|p| p.display().to_string()))
                    .unwrap_or_else(|| "<script>".to_string());
                let pos = self.source_pos_for(&file, *source_offset);
                self.call_stack.push_frame(CallFrame {
                    function_name: name.clone(),
                    file,
                    line: pos.line,
                    col: pos.col,
                });

                let result = if *is_async {
                    self.execute_async_function_body(params, body, closure_env, this_binding, args)
                } else {
                    self.execute_function_body(params, body, closure_env, this_binding, args)
                };

                let trace = self.call_stack.format_trace();
                self.call_stack.pop_frame();
                result.map_err(|err| self.attach_stack_to_error(err, &trace))
            }
            JsValue::NativeFunction { handler, .. } => {
                self.call_native_function(handler, args, this_binding)
            }
            other => Err(RuntimeError::NotAFunction {
                name: format!("{other}"),
            }),
        }
    }

    pub(crate) fn attach_stack_to_error(&self, err: RuntimeError, trace: &str) -> RuntimeError {
        if trace.is_empty() {
            return err;
        }

        match err {
            RuntimeError::TypeError { message } => {
                if message.contains("\n    at ") {
                    RuntimeError::TypeError { message }
                } else {
                    RuntimeError::TypeError {
                        message: format!("{message}\n{trace}"),
                    }
                }
            }
            RuntimeError::UndefinedVariable { name } => RuntimeError::TypeError {
                message: format!("ReferenceError: '{name}' is not defined\n{trace}"),
            },
            RuntimeError::NotAFunction { name } => RuntimeError::TypeError {
                message: format!("TypeError: '{name}' is not a function\n{trace}"),
            },
            RuntimeError::ArityMismatch { expected, got } => RuntimeError::TypeError {
                message: format!("TypeError: expected {expected} arguments but got {got}\n{trace}"),
            },
            RuntimeError::ConstReassignment { name } => RuntimeError::TypeError {
                message: format!("TypeError: Assignment to constant variable '{name}'\n{trace}"),
            },
            RuntimeError::Thrown { .. } => err,
        }
    }

    fn create_generator_object(
        &mut self,
        params: &[crate::parser::ast::Param],
        body: &[crate::parser::ast::Stmt],
        closure_env: &[std::rc::Rc<std::cell::RefCell<crate::runtime::environment::Scope>>],
        this_binding: Option<JsValue>,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let saved_yields = std::mem::take(&mut self.generator_yields);
        self.generator_depth += 1;
        let return_value =
            self.execute_function_body(params, body, closure_env, this_binding, args);
        self.generator_depth -= 1;
        let yielded = std::mem::replace(&mut self.generator_yields, saved_yields);

        let ret_val = return_value?;

        let mut gen_state = JsGenerator::new();
        gen_state.yielded_values = yielded.into();
        gen_state.return_value = ret_val;
        gen_state.state = crate::runtime::value::generator::GeneratorState::Completed;
        let gen_rc = gen_state.wrapped();

        let mut obj = JsObject::new();

        obj.set(
            "next".to_string(),
            JsValue::NativeFunction {
                name: "next".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorNext(gen_rc.clone()),
            },
        );

        obj.set(
            "return".to_string(),
            JsValue::NativeFunction {
                name: "return".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorReturn(gen_rc),
            },
        );

        obj.set(
            "throw".to_string(),
            JsValue::NativeFunction {
                name: "throw".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorThrow,
            },
        );

        let iter_sym = symbol::symbol_iterator();
        let obj_rc = obj.wrapped();
        obj_rc.borrow_mut().set_symbol(
            iter_sym,
            JsValue::NativeFunction {
                name: "[Symbol.iterator]".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorIterator,
            },
        );

        Ok(JsValue::Object(obj_rc))
    }
}

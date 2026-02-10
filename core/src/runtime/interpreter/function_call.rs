use super::Interpreter;
use crate::diagnostics::stack_trace::CallFrame;
use crate::errors::RuntimeError;
use crate::runtime::gc::{Gc, GcCell};
use crate::runtime::value::array::JsArray;
use crate::runtime::value::generator::{GeneratorState, JsGenerator};
use crate::runtime::value::object::JsObject;
use crate::runtime::value::symbol;
use crate::runtime::value::JsValue;

impl Interpreter {
    pub(crate) fn eval_array_callback_method(
        &mut self,
        arr: &Gc<GcCell<JsArray>>,
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
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(result))))
            }
            "filter" => {
                let mut result = Vec::new();
                for elem in &elements {
                    let val = self.call_function(callback, std::slice::from_ref(elem))?;
                    if val.to_boolean() {
                        result.push(elem.clone());
                    }
                }
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(result))))
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
                Ok(JsValue::Array(self.heap.alloc_cell(JsArray::new(sorted))))
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
                ..
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
            JsValue::Proxy(proxy) => {
                let (trap, target) = {
                    let p = proxy.borrow();
                    p.check_revoked()
                        .map_err(|msg| RuntimeError::TypeError { message: msg })?;
                    (p.get_trap("apply"), p.target.clone())
                };
                if let Some(trap_fn) = trap {
                    let this_arg = this_binding.clone().unwrap_or(JsValue::Undefined);
                    let args_array =
                        JsValue::Array(self.heap.alloc_cell(JsArray::new(args.to_vec())));
                    self.call_function(&trap_fn, &[target, this_arg, args_array])
                } else {
                    self.call_function_with_this(&target, args, this_binding)
                }
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
        closure_env: &[Gc<GcCell<crate::runtime::environment::Scope>>],
        this_binding: Option<JsValue>,
        args: &[JsValue],
    ) -> Result<JsValue, RuntimeError> {
        let gen_state = JsGenerator::new(
            params.to_vec(),
            body.to_vec(),
            closure_env.to_vec(),
            this_binding,
            args.to_vec(),
        );
        let gen_gc = self.heap.alloc_cell(gen_state);

        let mut obj = JsObject::new();

        obj.set(
            "next".to_string(),
            JsValue::NativeFunction {
                name: "next".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorNext(gen_gc),
            },
        );

        obj.set(
            "return".to_string(),
            JsValue::NativeFunction {
                name: "return".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorReturn(gen_gc),
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
        let obj_gc = self.heap.alloc_cell(obj);
        obj_gc.borrow_mut().set_symbol(
            iter_sym,
            JsValue::NativeFunction {
                name: "[Symbol.iterator]".to_string(),
                handler: crate::runtime::value::NativeFunction::GeneratorIterator,
            },
        );

        Ok(JsValue::Object(obj_gc))
    }

    pub(crate) fn step_generator(
        &mut self,
        generator: &Gc<GcCell<JsGenerator>>,
    ) -> Result<JsValue, RuntimeError> {
        {
            let g = generator.borrow();
            if g.state == GeneratorState::Completed && g.yielded_values.is_empty() {
                return Ok(crate::runtime::value::iterator::iter_result(
                    g.return_value.clone(),
                    true,
                    &mut self.heap,
                ));
            }
        }

        let needs_execute = {
            let g = generator.borrow();
            g.state == GeneratorState::SuspendedStart
        };

        if needs_execute {
            let (params, body, captured_env, this_binding, args) = {
                let mut g = generator.borrow_mut();
                g.state = GeneratorState::Executing;
                (
                    g.params.clone(),
                    g.body.clone(),
                    g.captured_env.clone(),
                    g.this_binding.clone(),
                    g.args.clone(),
                )
            };

            let saved_yields = std::mem::take(&mut self.generator_yields);
            self.generator_depth += 1;
            let result =
                self.execute_function_body(&params, &body, &captured_env, this_binding, &args);
            self.generator_depth -= 1;
            let yielded = std::mem::replace(&mut self.generator_yields, saved_yields);

            let ret_val = result.unwrap_or(JsValue::Undefined);
            let mut g = generator.borrow_mut();
            g.yielded_values = yielded.into();
            g.return_value = ret_val;
            g.state = GeneratorState::Completed;
        }

        let mut g = generator.borrow_mut();
        if let Some(value) = g.yielded_values.pop_front() {
            drop(g);
            Ok(crate::runtime::value::iterator::iter_result(
                value,
                false,
                &mut self.heap,
            ))
        } else {
            let ret = g.return_value.clone();
            drop(g);
            Ok(crate::runtime::value::iterator::iter_result(
                ret,
                true,
                &mut self.heap,
            ))
        }
    }

    pub(crate) fn construct_native_class(
        &mut self,
        class_name: &str,
        args: &[JsValue],
        _this_binding: Option<JsValue>,
    ) -> Result<JsValue, RuntimeError> {
        let class_def = self
            .native_classes
            .get(class_name)
            .cloned()
            .ok_or_else(|| RuntimeError::TypeError {
                message: format!("native class '{class_name}' not found"),
            })?;

        let this_obj = self.heap.alloc_cell(JsObject::new());
        let this = JsValue::Object(this_obj);
        let fn_args =
            crate::embedding::function_args::FunctionArgs::new(this.clone(), args.to_vec());

        let mut instance = if let Some(constructor) = &class_def.constructor {
            let result = constructor.call(fn_args)?;
            match result {
                JsValue::Undefined => this,
                other => other,
            }
        } else {
            this
        };

        if let JsValue::Object(object) = &mut instance {
            let mut obj = object.borrow_mut();
            for (name, callback) in &class_def.methods {
                obj.set(
                    name.clone(),
                    JsValue::NativeFunction {
                        name: name.clone(),
                        handler: crate::runtime::value::NativeFunction::Host(callback.clone()),
                    },
                );
            }
            for (name, callback) in &class_def.getters {
                obj.set_getter(
                    name.clone(),
                    JsValue::NativeFunction {
                        name: format!("get {name}"),
                        handler: crate::runtime::value::NativeFunction::Host(callback.clone()),
                    },
                );
            }
            for (name, callback) in &class_def.setters {
                obj.set_setter(
                    name.clone(),
                    JsValue::NativeFunction {
                        name: format!("set {name}"),
                        handler: crate::runtime::value::NativeFunction::Host(callback.clone()),
                    },
                );
            }
        }

        Ok(instance)
    }
}

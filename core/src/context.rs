use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::embedding::callback::NativeFunctionBoxed;
use crate::embedding::class_builder::NativeClassDef;
use crate::embedding::function_args::FunctionArgs;
use crate::errors::CrustyError;
use crate::runtime::environment::BindingKind;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::object::JsObject;
use crate::runtime::value::{JsValue, NativeFunction};

/// A single JavaScript execution context.
pub struct Context {
    interpreter: Interpreter,
    native_classes: HashMap<String, NativeClassDef>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new_with_realtime_timers(true),
            native_classes: HashMap::new(),
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<(), CrustyError> {
        let tokens = crate::lexer::lex(source)?;
        let program = crate::parser::parse(tokens)?;
        self.interpreter.run(&program)?;
        Ok(())
    }

    pub fn eval_module<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CrustyError> {
        let path_buf: PathBuf = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path_buf).map_err(|err| {
            CrustyError::Runtime(crate::errors::RuntimeError::TypeError {
                message: format!("failed to read module '{}': {err}", path_buf.display()),
            })
        })?;
        let tokens = crate::lexer::lex(&source)?;
        let program = crate::parser::parse(tokens)?;
        self.interpreter.run_with_path(&program, path_buf)?;
        Ok(())
    }

    pub fn get_global(&self, name: &str) -> Result<JsValue, CrustyError> {
        Ok(self.interpreter.env.get(name)?)
    }

    pub fn set_global(&mut self, name: impl Into<String>, value: JsValue) {
        let name = name.into();
        if self.interpreter.env.set(&name, value.clone()).is_err() {
            self.interpreter
                .env
                .define_with_kind(name, value, BindingKind::Var);
        }
    }

    pub fn set_global_function<F>(&mut self, name: impl Into<String>, callback: F)
    where
        F: Fn(FunctionArgs) -> Result<JsValue, crate::errors::RuntimeError> + Send + Sync + 'static,
    {
        let name = name.into();
        let function = JsValue::NativeFunction {
            name: name.clone(),
            handler: NativeFunction::Host(NativeFunctionBoxed::from_closure(callback)),
        };
        self.set_global(name, function);
    }

    pub fn register_class(&mut self, class_def: NativeClassDef) {
        let class_name = class_def.name.clone();

        let mut merged_methods = HashMap::new();
        let mut merged_getters = HashMap::new();
        let mut merged_setters = HashMap::new();

        if let Some(parent_name) = &class_def.parent {
            if let Some(parent) = self.native_classes.get(parent_name) {
                merged_methods.extend(parent.methods.clone());
                merged_getters.extend(parent.getters.clone());
                merged_setters.extend(parent.setters.clone());
            }
        }

        merged_methods.extend(class_def.methods.clone());
        merged_getters.extend(class_def.getters.clone());
        merged_setters.extend(class_def.setters.clone());

        let constructor = class_def.constructor.clone();
        let methods = merged_methods;
        let getters = merged_getters;
        let setters = merged_setters;
        self.set_global_function(class_name, move |args| {
            let mut instance = if let Some(constructor) = &constructor {
                constructor.call(args)?
            } else {
                JsValue::Object(JsObject::new().wrapped())
            };

            if let JsValue::Object(object) = &mut instance {
                let mut object = object.borrow_mut();
                for (name, callback) in &methods {
                    object.set(
                        name.clone(),
                        JsValue::NativeFunction {
                            name: name.clone(),
                            handler: NativeFunction::Host(callback.clone()),
                        },
                    );
                }
                for (name, callback) in &getters {
                    object.set_getter(
                        name.clone(),
                        JsValue::NativeFunction {
                            name: format!("get {name}"),
                            handler: NativeFunction::Host(callback.clone()),
                        },
                    );
                }
                for (name, callback) in &setters {
                    object.set_setter(
                        name.clone(),
                        JsValue::NativeFunction {
                            name: format!("set {name}"),
                            handler: NativeFunction::Host(callback.clone()),
                        },
                    );
                }
            }

            Ok(instance)
        });

        self.native_classes
            .insert(class_def.name.clone(), class_def);
    }

    pub fn run_microtasks(&mut self) -> Result<(), CrustyError> {
        self.interpreter.run_microtasks_only()?;
        Ok(())
    }

    pub fn run_pending_timers(&mut self) -> Result<(), CrustyError> {
        self.interpreter.run_pending_timers()?;
        Ok(())
    }

    pub fn output(&self) -> &[String] {
        self.interpreter.output()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

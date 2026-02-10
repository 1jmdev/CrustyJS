use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::embedding::callback::NativeFunctionBoxed;
use crate::embedding::class_builder::NativeClassDef;
use crate::embedding::event_target::EventTarget;
use crate::embedding::function_args::FunctionArgs;
use crate::errors::CrustyError;
use crate::runtime::environment::BindingKind;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::{JsValue, NativeFunction};

pub struct Context {
    interpreter: Interpreter,
}

impl Context {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new_with_realtime_timers(true),
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

        if let Some(parent_name) = &class_def.parent
            && let Some(parent) = self.interpreter.native_classes.get(parent_name)
        {
            merged_methods.extend(parent.methods.clone());
            merged_getters.extend(parent.getters.clone());
            merged_setters.extend(parent.setters.clone());
        }

        merged_methods.extend(class_def.methods.clone());
        merged_getters.extend(class_def.getters.clone());
        merged_setters.extend(class_def.setters.clone());

        let stored_def = NativeClassDef {
            name: class_def.name.clone(),
            constructor: class_def.constructor.clone(),
            methods: merged_methods,
            static_methods: class_def.static_methods.clone(),
            getters: merged_getters,
            setters: merged_setters,
            parent: class_def.parent.clone(),
        };

        self.interpreter
            .native_classes
            .insert(class_name.clone(), stored_def);

        let function = JsValue::NativeFunction {
            name: class_name.clone(),
            handler: NativeFunction::NativeClassConstructor(class_name),
        };
        self.set_global(class_def.name, function);
    }

    pub fn run_microtasks(&mut self) -> Result<(), CrustyError> {
        self.interpreter.run_microtasks_only()?;
        Ok(())
    }

    pub fn run_pending_timers(&mut self) -> Result<(), CrustyError> {
        self.interpreter.run_pending_timers()?;
        Ok(())
    }

    pub fn run_animation_callbacks(&mut self, timestamp_ms: f64) -> Result<(), CrustyError> {
        self.interpreter.run_animation_callbacks(timestamp_ms)?;
        Ok(())
    }

    pub fn dispatch_event(
        &mut self,
        target: &EventTarget,
        event_name: &str,
        event_obj: JsValue,
    ) -> Result<(), CrustyError> {
        for listener in target.listeners_for(event_name) {
            self.interpreter.call_function_with_this(
                &listener,
                std::slice::from_ref(&event_obj),
                None,
            )?;
        }
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

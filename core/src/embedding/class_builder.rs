use std::collections::HashMap;

use crate::embedding::callback::NativeFunctionBoxed;
use crate::embedding::function_args::FunctionArgs;
use crate::errors::RuntimeError;
use crate::runtime::value::JsValue;

pub struct NativeClassDef {
    pub name: String,
    pub constructor: Option<NativeFunctionBoxed>,
    pub methods: HashMap<String, NativeFunctionBoxed>,
    pub static_methods: HashMap<String, NativeFunctionBoxed>,
    pub getters: HashMap<String, NativeFunctionBoxed>,
    pub setters: HashMap<String, NativeFunctionBoxed>,
    pub parent: Option<String>,
}

pub struct ClassBuilder {
    name: String,
    constructor: Option<NativeFunctionBoxed>,
    methods: HashMap<String, NativeFunctionBoxed>,
    static_methods: HashMap<String, NativeFunctionBoxed>,
    getters: HashMap<String, NativeFunctionBoxed>,
    setters: HashMap<String, NativeFunctionBoxed>,
    parent: Option<String>,
}

impl ClassBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constructor: None,
            methods: HashMap::new(),
            static_methods: HashMap::new(),
            getters: HashMap::new(),
            setters: HashMap::new(),
            parent: None,
        }
    }

    pub fn constructor<F>(mut self, callback: F) -> Self
    where
        F: Fn(FunctionArgs) -> Result<JsValue, RuntimeError> + Send + Sync + 'static,
    {
        self.constructor = Some(NativeFunctionBoxed::from_closure(callback));
        self
    }

    pub fn method<F>(mut self, name: impl Into<String>, callback: F) -> Self
    where
        F: Fn(FunctionArgs) -> Result<JsValue, RuntimeError> + Send + Sync + 'static,
    {
        self.methods
            .insert(name.into(), NativeFunctionBoxed::from_closure(callback));
        self
    }

    pub fn static_method<F>(mut self, name: impl Into<String>, callback: F) -> Self
    where
        F: Fn(FunctionArgs) -> Result<JsValue, RuntimeError> + Send + Sync + 'static,
    {
        self.static_methods
            .insert(name.into(), NativeFunctionBoxed::from_closure(callback));
        self
    }

    pub fn property_getter<F>(mut self, name: impl Into<String>, callback: F) -> Self
    where
        F: Fn(FunctionArgs) -> Result<JsValue, RuntimeError> + Send + Sync + 'static,
    {
        self.getters
            .insert(name.into(), NativeFunctionBoxed::from_closure(callback));
        self
    }

    pub fn property_setter<F>(mut self, name: impl Into<String>, callback: F) -> Self
    where
        F: Fn(FunctionArgs) -> Result<JsValue, RuntimeError> + Send + Sync + 'static,
    {
        self.setters
            .insert(name.into(), NativeFunctionBoxed::from_closure(callback));
        self
    }

    pub fn inherit(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn build(self) -> NativeClassDef {
        NativeClassDef {
            name: self.name,
            constructor: self.constructor,
            methods: self.methods,
            static_methods: self.static_methods,
            getters: self.getters,
            setters: self.setters,
            parent: self.parent,
        }
    }
}

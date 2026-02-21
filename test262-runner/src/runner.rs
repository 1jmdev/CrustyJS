use std::path::Path;
use std::sync::{Arc, Mutex};

use crustyjs_core::errors::{CrustyError, RuntimeError};
use crustyjs_core::{Context, Value};

use crate::harness;
use crate::metadata::{strip_frontmatter, Negative, TestMetadata};

#[derive(Debug, Clone)]
pub enum TestResult {
    Passed,
    Failed(String),
    Skipped(String),
}

pub fn run_test(path: &Path, source: &str, metadata: &TestMetadata) -> TestResult {
    if metadata.is_module() {
        return run_module_test(path, metadata);
    }

    let test_source = strip_frontmatter(source);

    if metadata.is_raw() {
        return run_single(test_source, metadata, metadata.is_async());
    }

    if metadata.is_no_strict() {
        return run_single(
            &compose(metadata, test_source),
            metadata,
            metadata.is_async(),
        );
    }

    if metadata.is_only_strict() {
        let strict_source = format!("\"use strict\";\n{}", compose(metadata, test_source));
        return run_single(&strict_source, metadata, metadata.is_async());
    }

    let sloppy_result = run_single(
        &compose(metadata, test_source),
        metadata,
        metadata.is_async(),
    );
    if matches!(sloppy_result, TestResult::Failed(_)) {
        return sloppy_result;
    }

    let strict_source = format!("\"use strict\";\n{}", compose(metadata, test_source));
    run_single(&strict_source, metadata, metadata.is_async())
}

fn compose(metadata: &TestMetadata, test_source: &str) -> String {
    harness::compose_source(&metadata.includes, test_source)
}

fn run_module_test(path: &Path, metadata: &TestMetadata) -> TestResult {
    let mut ctx = Context::new_with_realtime(false);
    ctx.set_max_steps(1_000_000);

    if let Err(err) = ctx.eval(harness::host_harness_source()) {
        return TestResult::Failed(format!("failed to initialize host harness: {err}"));
    }

    let done_state = if metadata.is_async() {
        Some(install_done_callback(&mut ctx))
    } else {
        None
    };

    match ctx.eval_module(path) {
        Ok(()) => {
            if metadata.negative.is_some() {
                TestResult::Failed("expected error but test passed".into())
            } else if let Some(state) = done_state.as_ref() {
                evaluate_async_completion(state)
            } else {
                TestResult::Passed
            }
        }
        Err(e) => evaluate_error_with_neg(&metadata.negative, &safe_error_message(&e)),
    }
}

fn run_single(source: &str, metadata: &TestMetadata, is_async: bool) -> TestResult {
    let negative = metadata.negative.clone();

    let mut ctx = Context::new_with_realtime(false);
    ctx.set_max_steps(1_000_000);

    let done_state = if is_async {
        Some(install_done_callback(&mut ctx))
    } else {
        None
    };

    let result = ctx.eval(source);

    match result {
        Ok(()) => {
            if negative.is_some() {
                TestResult::Failed("expected error but test passed".into())
            } else if let Some(state) = done_state.as_ref() {
                evaluate_async_completion(state)
            } else {
                TestResult::Passed
            }
        }
        Err(e) => evaluate_error_with_neg(&negative, &safe_error_message(&e)),
    }
}

#[derive(Default)]
struct AsyncDoneState {
    called: bool,
    error: Option<String>,
}

fn install_done_callback(ctx: &mut Context) -> Arc<Mutex<AsyncDoneState>> {
    let state = Arc::new(Mutex::new(AsyncDoneState::default()));
    let callback_state = Arc::clone(&state);

    ctx.set_global_function("$DONE", move |args| {
        let mut state = callback_state
            .lock()
            .expect("$DONE state mutex should not be poisoned");

        state.called = true;
        if state.error.is_none() {
            state.error = args.get(0).and_then(done_error_message);
        }

        Ok(Value::Undefined)
    });

    state
}

fn evaluate_async_completion(state: &Arc<Mutex<AsyncDoneState>>) -> TestResult {
    let state = state
        .lock()
        .expect("$DONE state mutex should not be poisoned");

    if !state.called {
        return TestResult::Failed("async test did not call $DONE".into());
    }

    if let Some(error) = &state.error {
        return TestResult::Failed(format!("$DONE called with error: {error}"));
    }

    TestResult::Passed
}

fn done_error_message(value: &Value) -> Option<String> {
    match value {
        Value::Undefined => None,
        Value::Null | Value::Boolean(_) | Value::Number(_) | Value::String(_) => {
            Some(value.to_string())
        }
        Value::Function { .. } => Some("function".into()),
        Value::NativeFunction { .. } => Some("native function".into()),
        Value::Symbol(_) => Some("symbol".into()),
        Value::Object(_) => Some("object".into()),
        Value::Array(_) => Some("array".into()),
        Value::Promise(_) => Some("promise".into()),
        Value::Map(_) => Some("map".into()),
        Value::Set(_) => Some("set".into()),
        Value::WeakMap(_) => Some("weakmap".into()),
        Value::WeakSet(_) => Some("weakset".into()),
        Value::RegExp(_) => Some("regexp".into()),
        Value::Proxy(_) => Some("proxy".into()),
    }
}

fn safe_error_message(error: &CrustyError) -> String {
    match error {
        CrustyError::Syntax(err) => format!("SyntaxError: {}", err.message),
        CrustyError::Runtime(err) => safe_runtime_error_message(err),
    }
}

fn safe_runtime_error_message(error: &RuntimeError) -> String {
    match error {
        RuntimeError::UndefinedVariable { name } => {
            format!("ReferenceError: '{name}' is not defined")
        }
        RuntimeError::NotAFunction { name } => format!("TypeError: '{name}' is not a function"),
        RuntimeError::ArityMismatch { expected, got } => {
            format!("TypeError: expected {expected} arguments but got {got}")
        }
        RuntimeError::TypeError { message } => format!("TypeError: {message}"),
        RuntimeError::ConstReassignment { name } => {
            format!("TypeError: Assignment to constant variable '{name}'")
        }
        RuntimeError::Thrown { value } => format!("Uncaught {}", format_thrown_value(value)),
    }
}

fn format_thrown_value(value: &Value) -> String {
    match value {
        Value::Object(obj) => {
            let obj = obj.borrow();
            let name = obj.get("name");
            let message = obj.get("message");

            match (name, message) {
                (Some(Value::String(name)), Some(Value::String(message)))
                    if !message.is_empty() =>
                {
                    format!("{name}: {message}")
                }
                (Some(Value::String(name)), _) => name,
                _ => "object".into(),
            }
        }
        _ => value.to_string(),
    }
}

fn evaluate_error_with_neg(negative: &Option<Negative>, error_msg: &str) -> TestResult {
    match negative {
        Some(neg) => {
            if error_msg.contains(&neg.error_type) {
                TestResult::Passed
            } else {
                TestResult::Failed(format!(
                    "expected {} but got: {}",
                    neg.error_type, error_msg
                ))
            }
        }
        None => TestResult::Failed(error_msg.to_string()),
    }
}

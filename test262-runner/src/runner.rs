use std::path::Path;
use std::sync::{Arc, Mutex};

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
        let strict_source = format!("\"use strict\";\n{}", test_source);
        return run_single(
            &compose(metadata, &strict_source),
            metadata,
            metadata.is_async(),
        );
    }

    let sloppy_result = run_single(
        &compose(metadata, test_source),
        metadata,
        metadata.is_async(),
    );
    if matches!(sloppy_result, TestResult::Failed(_)) {
        return sloppy_result;
    }

    let strict_source = format!("\"use strict\";\n{}", test_source);
    run_single(
        &compose(metadata, &strict_source),
        metadata,
        metadata.is_async(),
    )
}

fn compose(metadata: &TestMetadata, test_source: &str) -> String {
    harness::compose_source(&metadata.includes, test_source)
}

fn run_module_test(path: &Path, metadata: &TestMetadata) -> TestResult {
    let mut ctx = Context::new();
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
        Err(e) => evaluate_error_with_neg(&metadata.negative, &e.to_string()),
    }
}

fn run_single(source: &str, metadata: &TestMetadata, is_async: bool) -> TestResult {
    let negative = metadata.negative.clone();

    let mut ctx = Context::new();
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
        Err(e) => evaluate_error_with_neg(&negative, &e.to_string()),
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
            state.error = args
                .get(0)
                .filter(|value| !matches!(value, Value::Undefined))
                .map(ToString::to_string);
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

use std::path::Path;

use crustyjs_core::Context;

use crate::harness;
use crate::metadata::{Negative, TestMetadata, strip_frontmatter};

#[derive(Debug, Clone)]
pub enum TestResult {
    Passed,
    Failed(String),
    Skipped(String),
}

pub fn run_test(path: &Path, source: &str, metadata: &TestMetadata) -> TestResult {
    if metadata.is_async() {
        return TestResult::Skipped("async tests not supported".into());
    }

    if metadata.is_module() {
        return run_module_test(path, metadata);
    }

    let test_source = strip_frontmatter(source);

    if metadata.is_raw() {
        return run_single(test_source, metadata);
    }

    if metadata.is_no_strict() {
        return run_single(&compose(metadata, test_source), metadata);
    }

    if metadata.is_only_strict() {
        let strict_source = format!("\"use strict\";\n{}", test_source);
        return run_single(&compose(metadata, &strict_source), metadata);
    }

    let sloppy_result = run_single(&compose(metadata, test_source), metadata);
    if matches!(sloppy_result, TestResult::Failed(_)) {
        return sloppy_result;
    }

    let strict_source = format!("\"use strict\";\n{}", test_source);
    run_single(&compose(metadata, &strict_source), metadata)
}

fn compose(metadata: &TestMetadata, test_source: &str) -> String {
    harness::compose_source(&metadata.includes, test_source)
}

fn run_module_test(path: &Path, metadata: &TestMetadata) -> TestResult {
    let mut ctx = Context::new();
    match ctx.eval_module(path) {
        Ok(()) => {
            if metadata.negative.is_some() {
                TestResult::Failed("expected error but test passed".into())
            } else {
                TestResult::Passed
            }
        }
        Err(e) => evaluate_error_with_neg(&metadata.negative, &e.to_string()),
    }
}

fn run_single(source: &str, metadata: &TestMetadata) -> TestResult {
    let negative = metadata.negative.clone();

    let mut ctx = Context::new();
    ctx.set_max_steps(1_000_000);
    let result = ctx.eval(source);

    match result {
        Ok(()) => {
            if negative.is_some() {
                TestResult::Failed("expected error but test passed".into())
            } else {
                TestResult::Passed
            }
        }
        Err(e) => evaluate_error_with_neg(&negative, &e.to_string()),
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

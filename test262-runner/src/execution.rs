use std::fs;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use colored::Colorize;
use rayon::prelude::*;

use crate::metadata::{self, parse_frontmatter};
use crate::panic_message::format_panic;
use crate::runner::{TestResult, run_test};
use crate::stats::Analysis;

pub struct SuiteSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub failures: Vec<(PathBuf, String)>,
    pub analysis: Analysis,
}

pub fn init_thread_pool() {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);

    rayon::ThreadPoolBuilder::new()
        .stack_size(32 * 1024 * 1024)
        .num_threads(threads)
        .build_global()
        .ok();
}

pub fn run_suite(root: &Path, files: &[PathBuf], verbose: bool, analyze: bool) -> SuiteSummary {
    let records: Vec<_> = files.par_iter().map(|path| run_case(path)).collect();
    let mut summary = SuiteSummary {
        passed: 0,
        failed: 0,
        skipped: 0,
        failures: Vec::new(),
        analysis: Analysis::default(),
    };

    for record in records {
        if analyze {
            summary.analysis.record(root, &record.path, &record.result);
        }
        apply_record(&mut summary, &record.path, &record.result, verbose);
    }

    summary
}

struct CaseRecord {
    path: PathBuf,
    result: TestResult,
}

fn run_case(path: &Path) -> CaseRecord {
    let source = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            return CaseRecord {
                path: path.to_path_buf(),
                result: TestResult::Skipped(format!("failed to read test: {err}")),
            };
        }
    };

    let meta = parse_frontmatter(&source).unwrap_or_else(metadata::TestMetadata::default);
    let result = panic::catch_unwind(AssertUnwindSafe(|| run_test(path, &source, &meta)))
        .unwrap_or_else(|panic_payload| TestResult::Failed(format_panic(panic_payload)));

    CaseRecord {
        path: path.to_path_buf(),
        result,
    }
}

fn apply_record(summary: &mut SuiteSummary, path: &Path, result: &TestResult, verbose: bool) {
    match result {
        TestResult::Passed => {
            summary.passed += 1;
            if verbose {
                println!("{} {}", "PASS".green(), path.display());
            }
        }
        TestResult::Failed(reason) => {
            summary.failed += 1;
            if verbose {
                println!("{} {} - {}", "FAIL".red(), path.display(), reason);
            }
            summary.failures.push((path.to_path_buf(), reason.clone()));
        }
        TestResult::Skipped(reason) => {
            summary.skipped += 1;
            if verbose {
                println!("{} {} - {}", "SKIP".yellow(), path.display(), reason);
            }
        }
    }
}

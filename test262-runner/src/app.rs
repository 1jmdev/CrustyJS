use std::time::Instant;

use clap::Parser;
use colored::Colorize;

use crate::cli::Cli;
use crate::discovery::collect_test_files;
use crate::execution::{init_thread_pool, run_suite};
use crate::harness;
use crate::stats::print_analysis;

pub fn run() {
    let cli = Cli::parse();
    let start = Instant::now();

    init_thread_pool();
    harness::get_harness_cache();

    let files = collect_test_files(&cli.path);
    println!(
        "{} {} test files...\n",
        "Running".bold().cyan(),
        files.len()
    );

    let summary = run_suite(&cli.path, &files, cli.verbose, cli.analyze);
    print_totals(
        summary.passed,
        summary.failed,
        summary.skipped,
        start.elapsed().as_secs_f64(),
    );

    if cli.analyze {
        print_analysis(&summary.analysis);
    }

    if summary.failed > 0 && !cli.verbose {
        print_failure_sample(&summary.failures);
    }
}

fn print_totals(passed: usize, failed: usize, skipped: usize, elapsed_secs: f64) {
    println!("\n{}", "=".repeat(60));
    println!(
        "Passed: {} | Failed: {} | Skipped: {}",
        passed.to_string().green().bold(),
        failed.to_string().red().bold(),
        skipped.to_string().yellow().bold()
    );
    println!("Completed in {:.2}s", elapsed_secs);
    println!("{}", "=".repeat(60));
}

fn print_failure_sample(failures: &[(std::path::PathBuf, String)]) {
    let max_rows = 10;
    let shown = failures.len().min(max_rows);

    println!("\n{}", "Sample failures:".red().bold());
    for (path, reason) in failures.iter().take(shown) {
        println!("  {} - {}", path.display(), reason);
    }
    if failures.len() > shown {
        println!(
            "  ... and {} more (use --verbose for all)",
            failures.len() - shown
        );
    }
}

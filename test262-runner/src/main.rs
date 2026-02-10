mod harness;
mod metadata;
mod runner;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use clap::Parser;
use colored::Colorize;
use rayon::prelude::*;
use walkdir::WalkDir;

use metadata::parse_frontmatter;
use runner::{run_test, TestResult};

#[derive(Parser)]
#[command(name = "test262-runner", about = "Run ECMAScript Test262 suite")]
struct Cli {
    #[arg(default_value = "test262/test")]
    path: PathBuf,

    #[arg(long, default_value_t = false)]
    verbose: bool,
}

fn collect_test_files(root: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().is_some_and(|ext| ext == "js")
                && !e.path().components().any(|c| {
                    let s = c.as_os_str().to_string_lossy();
                    s == "_FIXTURE" || s == "intl402"
                })
        })
        .map(|e| e.into_path())
        .collect()
}

fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    harness::get_harness_cache();

    let files = collect_test_files(&cli.path);
    let total = files.len();

    println!("{} {} test files...\n", "Running".bold().cyan(), total);

    let passed = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    let failures: std::sync::Mutex<Vec<(PathBuf, String)>> = std::sync::Mutex::new(Vec::new());

    files.par_iter().for_each(|path| {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => {
                skipped.fetch_add(1, Ordering::Relaxed);
                return;
            }
        };

        let meta = match parse_frontmatter(&source) {
            Some(m) => m,
            None => metadata::TestMetadata::default(),
        };

        let result = run_test(path, &source, &meta);

        match &result {
            TestResult::Passed => {
                passed.fetch_add(1, Ordering::Relaxed);
                if cli.verbose {
                    println!("{} {}", "PASS".green(), path.display());
                }
            }
            TestResult::Failed(reason) => {
                failed.fetch_add(1, Ordering::Relaxed);
                if cli.verbose {
                    println!("{} {} - {}", "FAIL".red(), path.display(), reason);
                }
                failures
                    .lock()
                    .unwrap()
                    .push((path.clone(), reason.clone()));
            }
            TestResult::Skipped(reason) => {
                skipped.fetch_add(1, Ordering::Relaxed);
                if cli.verbose {
                    println!("{} {} - {}", "SKIP".yellow(), path.display(), reason);
                }
            }
        }
    });

    let elapsed = start.elapsed();
    let p = passed.load(Ordering::Relaxed);
    let f = failed.load(Ordering::Relaxed);
    let s = skipped.load(Ordering::Relaxed);

    println!("\n{}", "=".repeat(60));
    println!(
        "Passed: {} | Failed: {} | Skipped: {}",
        p.to_string().green().bold(),
        f.to_string().red().bold(),
        s.to_string().yellow().bold()
    );
    println!("Completed in {:.2}s", elapsed.as_secs_f64());
    println!("{}", "=".repeat(60));

    if f > 0 && !cli.verbose {
        let fail_list = failures.lock().unwrap();
        let show = fail_list.len().min(20);
        println!("\n{}", "First failures:".red().bold());
        for (path, reason) in fail_list.iter().take(show) {
            println!("  {} - {}", path.display(), reason);
        }
        if fail_list.len() > show {
            println!("  ... and {} more", fail_list.len() - show);
        }
    }
}

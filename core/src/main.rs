use std::fs;
use std::process;

use clap::Parser;

#[derive(Parser)]
#[command(name = "crustyjs", about = "A minimal JavaScript interpreter in Rust")]
struct Cli {
    /// Path to a .js file to execute
    file: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    if cli.file.is_none() {
        if let Err(err) = crustyjs::repl::run() {
            eprintln!("{err:?}");
            process::exit(1);
        }
        return;
    }

    let file = cli.file.expect("checked above");
    let source = match fs::read_to_string(&file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {e}", file);
            process::exit(1);
        }
    };

    if let Err(err) = crustyjs::run(&source) {
        eprintln!("{err:?}");
        process::exit(1);
    }
}

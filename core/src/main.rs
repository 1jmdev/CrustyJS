use std::fs;
use std::process;

use clap::Parser;
use miette::{IntoDiagnostic, NamedSource};

#[derive(Parser)]
#[command(name = "crustyjs", about = "A minimal JavaScript interpreter in Rust")]
struct Cli {
    /// Path to a .js file to execute
    file: String,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    let source = fs::read_to_string(&cli.file)
        .into_diagnostic()
        .map_err(|e| {
            eprintln!("Error reading file '{}': {e}", cli.file);
            process::exit(1);
        })
        .unwrap();

    let _named = NamedSource::new(&cli.file, source.clone());
    let _tokens = crustyjs::lexer::lex(&source)?;

    Ok(())
}

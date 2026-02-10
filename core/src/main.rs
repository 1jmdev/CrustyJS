use std::fs;
use std::process;

use clap::Parser;
use miette::NamedSource;

#[derive(Parser)]
#[command(name = "crustyjs", about = "A minimal JavaScript interpreter in Rust")]
struct Cli {
    /// Path to a .js file to execute
    file: String,
}

fn main() {
    let cli = Cli::parse();
    let source = match fs::read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {e}", cli.file);
            process::exit(1);
        }
    };

    if let Err(err) = crustyjs::run(&source) {
        let report = miette::Report::new(err).with_source_code(NamedSource::new(&cli.file, source));
        eprintln!("{report:?}");
        process::exit(1);
    }
}

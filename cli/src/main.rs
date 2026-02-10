#![allow(clippy::result_large_err)]

use std::fs;
use std::process;

use clap::Parser;

mod repl;

#[derive(Parser)]
#[command(name = "crustyjs", about = "A minimal JavaScript interpreter in Rust")]
struct Cli {
    /// Path to a .js file to execute
    file: Option<String>,
    /// Execute via bytecode VM path
    #[arg(long)]
    vm: bool,
    /// Print token stream
    #[arg(long)]
    tokens: bool,
    /// Print parsed AST
    #[arg(long)]
    ast: bool,
    /// Print compiled bytecode (VM compiler)
    #[arg(long)]
    bytecode: bool,
    /// Evaluate inline JavaScript source
    #[arg(long)]
    eval: Option<String>,
    /// Print version and exit
    #[arg(long)]
    version: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("crustyjs {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if cli.file.is_none() && cli.eval.is_none() {
        if let Err(err) = repl::run() {
            eprintln!("{err:?}");
            process::exit(1);
        }
        return;
    }

    let (source, source_path) = if let Some(code) = cli.eval {
        (code, std::path::PathBuf::from("."))
    } else {
        let file = cli.file.expect("checked above");
        match fs::read_to_string(&file) {
            Ok(s) => (s, std::path::PathBuf::from(file)),
            Err(e) => {
                eprintln!("error: could not read '{}': {e}", file);
                process::exit(1);
            }
        }
    };

    let tokens = match crustyjs::lexer::lex(&source) {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!(
                "{}",
                format_syntax_error(&source, &source_path, "lex", &err)
            );
            process::exit(1);
        }
    };

    if cli.tokens {
        for token in &tokens {
            println!("{:?}", token);
        }
    }

    let program = match crustyjs::parser::parse(tokens.clone()) {
        Ok(program) => program,
        Err(err) => {
            eprintln!(
                "{}",
                format_syntax_error(&source, &source_path, "parse", &err)
            );
            process::exit(1);
        }
    };

    if cli.ast {
        println!("{:#?}", program);
    }

    if cli.bytecode {
        let mut compiler = crustyjs::vm::compiler::Compiler::new();
        let chunk = compiler.compile(program.clone());
        print!("{}", chunk.disassemble());
    }

    let result = if cli.vm {
        crustyjs::run_vm_with_path(&source, Some(source_path.clone())).map(|_| ())
    } else {
        let mut interp =
            crustyjs::runtime::interpreter::Interpreter::new_with_realtime_timers(true);
        interp
            .run_with_path(&program, source_path)
            .map_err(crustyjs::errors::CrustyError::from)
    };

    if let Err(err) = result {
        eprintln!("{err:?}");
        process::exit(1);
    }
}

fn format_syntax_error(
    source: &str,
    source_path: &std::path::Path,
    phase: &str,
    err: &crustyjs::errors::SyntaxError,
) -> String {
    let map = crustyjs::diagnostics::source_map::SourceMap::from_source(source);
    let pos = map.byte_to_pos(err.span.offset());
    format!(
        "{} error at {}:{}:{}: {}",
        phase,
        source_path.display(),
        pos.line,
        pos.col,
        err.message
    )
}

use std::fs;
use std::process;

use clap::Parser;

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
        if let Err(err) = crustyjs::repl::run() {
            eprintln!("{err:?}");
            process::exit(1);
        }
        return;
    }

    let source = if let Some(code) = cli.eval {
        code
    } else {
        let file = cli.file.expect("checked above");
        match fs::read_to_string(&file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: could not read '{}': {e}", file);
                process::exit(1);
            }
        }
    };

    let tokens = match crustyjs::lexer::lex(&source) {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("{err:?}");
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
            eprintln!("{err:?}");
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
        crustyjs::run_vm(&source).map(|_| ())
    } else {
        let mut interp =
            crustyjs::runtime::interpreter::Interpreter::new_with_realtime_timers(true);
        interp
            .run(&program)
            .map_err(crustyjs::errors::CrustyError::from)
    };

    if let Err(err) = result {
        eprintln!("{err:?}");
        process::exit(1);
    }
}

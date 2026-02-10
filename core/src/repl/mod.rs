mod completer;

use crate::errors::CrustyError;
use crate::lexer;
use crate::parser;
use crate::runtime::interpreter::Interpreter;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub fn run() -> Result<(), CrustyError> {
    let mut rl = DefaultEditor::new().map_err(|e| {
        CrustyError::Runtime(crate::errors::RuntimeError::TypeError {
            message: format!("failed to initialize REPL: {e}"),
        })
    })?;
    let mut interp = Interpreter::new();

    let _ = completer::keywords();
    println!("CrustyJS v0.1.0");

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == ".exit" || trimmed == "exit" {
                    break;
                }
                let _ = rl.add_history_entry(trimmed);

                match lexer::lex(trimmed)
                    .map_err(CrustyError::from)
                    .and_then(|tokens| parser::parse(tokens).map_err(CrustyError::from))
                    .and_then(|program| interp.run(&program).map_err(CrustyError::from))
                {
                    Ok(_) => println!("undefined"),
                    Err(err) => eprintln!("{err:?}"),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("repl error: {e}");
                break;
            }
        }
    }

    Ok(())
}

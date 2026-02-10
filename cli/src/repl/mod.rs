mod completer;

use crustyjs::errors::{CrustyError, RuntimeError};
use crustyjs::runtime::interpreter::Interpreter;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;

pub fn run() -> Result<(), CrustyError> {
    let mut rl = DefaultEditor::new().map_err(|e| {
        CrustyError::Runtime(RuntimeError::TypeError {
            message: format!("failed to initialize REPL: {e}"),
        })
    })?;
    let mut interp = Interpreter::new_with_realtime_timers(true);

    let _ = completer::keywords();
    println!("CrustyJS v0.1.0");

    loop {
        match rl.readline("> ") {
            Ok(mut line) => {
                while needs_more_input(&line) {
                    match rl.readline("... ") {
                        Ok(next) => {
                            line.push('\n');
                            line.push_str(&next);
                        }
                        Err(_) => break,
                    }
                }

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == ".exit" || trimmed == "exit" {
                    break;
                }
                if trimmed == ".help" {
                    println!(".help                Show commands");
                    println!(".clear               Reset interpreter state");
                    println!(".load <file.js>      Load and run script");
                    println!(".exit                Exit REPL");
                    continue;
                }
                if trimmed == ".clear" {
                    interp = Interpreter::new_with_realtime_timers(true);
                    println!("environment cleared");
                    continue;
                }
                if let Some(path) = trimmed.strip_prefix(".load ") {
                    match fs::read_to_string(path.trim()) {
                        Ok(src) => {
                            run_snippet(&mut interp, &src)?;
                        }
                        Err(err) => eprintln!("load error: {err}"),
                    }
                    continue;
                }

                let suggestions = completer::suggest_for(trimmed);
                if suggestions.len() <= 5 && suggestions.iter().any(|kw| kw.starts_with(trimmed)) {
                    let _ = suggestions;
                }
                let _ = rl.add_history_entry(trimmed);

                match run_snippet(&mut interp, trimmed) {
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

fn run_snippet(interp: &mut Interpreter, source: &str) -> Result<(), CrustyError> {
    crustyjs::lexer::lex(source)
        .map_err(CrustyError::from)
        .and_then(|tokens| crustyjs::parser::parse(tokens).map_err(CrustyError::from))
        .and_then(|program| {
            interp
                .run_with_path(&program, std::path::PathBuf::from("."))
                .map_err(CrustyError::from)
        })
}

fn needs_more_input(source: &str) -> bool {
    let mut parens = 0i32;
    let mut braces = 0i32;
    let mut brackets = 0i32;
    for ch in source.chars() {
        match ch {
            '(' => parens += 1,
            ')' => parens -= 1,
            '{' => braces += 1,
            '}' => braces -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            _ => {}
        }
    }

    parens > 0 || braces > 0 || brackets > 0 || source.trim_end().ends_with('\\')
}

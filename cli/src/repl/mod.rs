mod completer;
mod helper;
mod highlighter;
mod hinter;

use crustyjs::context::Context;
use crustyjs::errors::{CrustyError, RuntimeError};
use owo_colors::OwoColorize;
use rustyline::error::ReadlineError;
use rustyline::{Config, EditMode, Editor};
use std::fs;

use self::helper::ReplHelper;

pub fn run() -> Result<(), CrustyError> {
    let config = Config::builder()
        .history_ignore_dups(true)
        .map_err(to_runtime_error)?
        .completion_type(rustyline::CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();

    let mut rl: Editor<ReplHelper, rustyline::history::DefaultHistory> =
        Editor::with_config(config).map_err(to_runtime_error)?;
    rl.set_helper(Some(ReplHelper));

    let mut ctx = Context::new_with_realtime(true);

    println!(
        "{} {}",
        "CrustyJS".bright_cyan().bold(),
        env!("CARGO_PKG_VERSION").bright_black()
    );
    println!("{}", "Type .help for REPL commands".bright_black());

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if handle_command(trimmed, &mut ctx)? {
                    continue;
                }

                let _ = rl.add_history_entry(trimmed);
                run_snippet(&mut ctx, trimmed);
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".yellow());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "bye".bright_black());
                break;
            }
            Err(err) => {
                eprintln!("{} {err}", "repl error:".red().bold());
                break;
            }
        }
    }

    Ok(())
}

fn handle_command(trimmed: &str, ctx: &mut Context) -> Result<bool, CrustyError> {
    if trimmed == ".exit" || trimmed == "exit" {
        std::process::exit(0);
    }
    if trimmed == ".help" {
        println!("{}", ".help                show commands".bright_blue());
        println!(
            "{}",
            ".clear               reset interpreter state".bright_blue()
        );
        println!(
            "{}",
            ".load <file.js>      load and run script".bright_blue()
        );
        println!("{}", ".exit                exit REPL".bright_blue());
        return Ok(true);
    }
    if trimmed == ".clear" {
        *ctx = Context::new_with_realtime(true);
        println!("{}", "environment cleared".green());
        return Ok(true);
    }
    if let Some(path) = trimmed.strip_prefix(".load ") {
        let path = path.trim();
        match fs::read_to_string(path) {
            Ok(source) => {
                run_snippet(ctx, &source);
            }
            Err(err) => eprintln!("{} {err}", "load error:".red().bold()),
        }
        return Ok(true);
    }
    Ok(false)
}

fn run_snippet(ctx: &mut Context, source: &str) {
    match ctx.eval(source) {
        Ok(()) => println!("{}", "undefined".bright_black()),
        Err(err) => eprintln!("{} {err:?}", "error:".red().bold()),
    }
}

pub fn needs_more_input(source: &str) -> bool {
    let mut parens = 0i32;
    let mut braces = 0i32;
    let mut brackets = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for ch in source.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if !in_double && ch == '\'' {
            in_single = !in_single;
            continue;
        }
        if !in_single && ch == '"' {
            in_double = !in_double;
            continue;
        }
        if in_single || in_double {
            continue;
        }
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

    in_single
        || in_double
        || parens > 0
        || braces > 0
        || brackets > 0
        || source.trim_end().ends_with('\\')
}

fn to_runtime_error(err: ReadlineError) -> CrustyError {
    CrustyError::Runtime(RuntimeError::TypeError {
        message: format!("failed to initialize REPL: {err}"),
    })
}

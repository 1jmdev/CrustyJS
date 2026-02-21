use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "test262-runner", about = "Run ECMAScript Test262 suite")]
pub struct Cli {
    #[arg(default_value = "test262/test")]
    pub path: PathBuf,

    #[arg(long, default_value_t = false)]
    pub verbose: bool,

    #[arg(long, default_value_t = false)]
    pub analyze: bool,
}

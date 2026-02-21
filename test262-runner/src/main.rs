mod app;
mod cli;
mod discovery;
mod execution;
mod harness;
mod metadata;
mod panic_message;
mod runner;
mod stats;

fn main() {
    app::run();
}

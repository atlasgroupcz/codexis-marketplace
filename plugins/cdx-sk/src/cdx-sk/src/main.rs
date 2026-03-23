mod cli;
mod core;
mod get;
mod sources;

use clap::Parser;
use std::process;

fn main() {
    let cli = cli::Cli::parse();

    if let Err(error) = cli::run(cli) {
        if error.should_print() {
            eprintln!("error: {error}");
        }
        process::exit(error.exit_code());
    }
}

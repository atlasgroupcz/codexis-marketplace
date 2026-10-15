use std::process::ExitCode;

use clap::Parser;
use turboleal::cli::Cli;

fn main() -> ExitCode {
    match turboleal::run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

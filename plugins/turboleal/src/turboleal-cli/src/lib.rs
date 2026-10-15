pub mod api;
pub mod cli;
mod commands;
pub mod model;
mod output;

use anyhow::{Context, Result};
use clap::CommandFactory;

use crate::api::ApiClient;
use crate::cli::{Cli, Command};
use crate::output::Output;

pub fn run(cli: Cli) -> Result<()> {
    if let Command::Completions { shell } = cli.command {
        clap_complete::generate(
            shell,
            &mut Cli::command(),
            "turboleal-cli",
            &mut std::io::stdout(),
        );
        return Ok(());
    }
    let token = cli
        .token
        .context("no access token: pass --token or set CODEXIS_USER_API_TOKEN")?;
    let client = ApiClient::new(&cli.api_url, &token)?;
    let output = Output::new(cli.output);
    commands::dispatch(&client, &output, cli.command)
}

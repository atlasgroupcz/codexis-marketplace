pub mod api;
pub mod cli;
mod commands;
pub mod config;
pub mod diff;
pub mod format;
pub mod model;
mod output;
pub mod settings;
mod sources;
pub mod summary;

use std::io::Read;

use anyhow::{Context, Result, anyhow};
use clap::CommandFactory;
use serde_json::Value;

use crate::api::ApiClient;
use crate::cli::{AuthCommand, Cli, Command};
use crate::output::Output;
use crate::settings::SettingsError;

pub fn run(cli: Cli) -> Result<()> {
    let output = Output::new(cli.output);
    match cli.command {
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "turboleal-cli",
                &mut std::io::stdout(),
            );
            Ok(())
        }
        Command::Auth(command) => auth(&cli.api_url, cli.api_key.as_deref(), &output, command),
        command => {
            let key = config::resolve_api_key(cli.api_key.as_deref())
                .ok_or_else(|| anyhow!(missing_key_hint(&cli.api_url)))?;
            let client = ApiClient::new(&cli.api_url, &key.value)?;
            commands::dispatch(&client, &output, command)
        }
    }
}

fn auth(api_url: &str, flag: Option<&str>, output: &Output, command: AuthCommand) -> Result<()> {
    match command {
        AuthCommand::Status => {
            let status = settings::status(api_url, flag).map_err(|error| match error {
                SettingsError::NotConfigured => anyhow!(missing_key_hint(api_url)),
                SettingsError::Rejected(_) => anyhow!("{error}\n{}", missing_key_hint(api_url)),
                other => other.into(),
            })?;
            output.print(&status, |status| {
                println!(
                    "API key {} ({})",
                    field(status, "maskedKey"),
                    field(status, "keySource")
                );
                println!("Signed in as {}", user(&status["user"]));
                println!("Turboleal {}", field(status, "apiUrl"));
            })
        }
        AuthCommand::Login { key } => {
            let key = match key.filter(|key| key != "-") {
                Some(key) => key,
                None => read_stdin()?,
            };
            let saved = settings::login(api_url, &key).map_err(|error| match error {
                SettingsError::Rejected(_) => anyhow!("{error}\n{}", missing_key_hint(api_url)),
                other => other.into(),
            })?;
            output.print(&saved, |saved| {
                println!(
                    "Saved API key {} for {}",
                    field(saved, "maskedKey"),
                    user(&saved["user"])
                );
            })
        }
        AuthCommand::Logout => {
            let removed = settings::logout()?;
            output.print(&removed, |_| println!("Removed the saved API key"))
        }
    }
}

fn read_stdin() -> Result<String> {
    let mut key = String::new();
    std::io::stdin()
        .read_to_string(&mut key)
        .context("cannot read the API key from standard input")?;
    Ok(key)
}

fn missing_key_hint(api_url: &str) -> String {
    let url = config::api_keys_url(api_url);
    format!(
        "no Turboleal API key. Create one at {url} (API klíče) and save it in the app \
         \"Turboleal - API klíč\" or with `turboleal-cli auth login`.\n\
         Chybí API klíč Turbolealu. Vytvořte ho na {url} (API klíče) a uložte v aplikaci \
         \"Turboleal - API klíč\" nebo přes `turboleal-cli auth login`."
    )
}

fn field<'a>(value: &'a Value, name: &str) -> &'a str {
    value[name].as_str().unwrap_or("-")
}

fn user(user: &Value) -> String {
    match (user["name"].as_str(), user["email"].as_str()) {
        (Some(name), Some(email)) => format!("{name} <{email}>"),
        (Some(name), None) => name.to_owned(),
        (None, Some(email)) => email.to_owned(),
        (None, None) => user["id"].as_str().unwrap_or("-").to_owned(),
    }
}

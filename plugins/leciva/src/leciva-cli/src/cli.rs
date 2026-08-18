use clap::{Parser, Subcommand};

use crate::core::config::{Config, CODEXIS_PLUGIN_LECIVA_API_URL_ENV, DEFAULT_API_BASE_URL};
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::leciva::{SearchLecivaArgs, SEARCH_LECIVA_HELP};

const ROOT_HELP: &str = "\
Data sources:
  CZSUKL   Registr léčiv SÚKL   Czech Medicines Registry (prehledy.sukl.cz)

Examples:
  leciva-cli search CZSUKL --query \"paralen\"
  leciva-cli search CZSUKL --latka ibuprofen --only-dodavka
  leciva-cli get leciva://doc/CZSUKL0000009/meta
  leciva-cli get leciva://doc/CZSUKL0000009/cena

Detailed source help:
  leciva-cli search CZSUKL --help

Get resource:
  leciva://doc/<DOC_ID>/meta | /cena   ·   leciva://doc/values/<FIELD>";

#[derive(Parser, Debug)]
#[command(
    name = "leciva-cli",
    version,
    about = "leciva-cli for SÚKL medicine search and leciva:// resource fetches",
    disable_version_flag = true,
    disable_help_subcommand = true,
    subcommand_required = true,
    arg_required_else_help = true,
    after_help = ROOT_HELP
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Search the Czech medicines registry",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a leciva:// resource", arg_required_else_help = true)]
    Get(GetArgs),
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "CZSUKL",
        visible_alias = "czsukl",
        about = "Czech Medicines Registry (prehledy.sukl.cz)",
        after_help = SEARCH_LECIVA_HELP,
        arg_required_else_help = true
    )]
    Czsukl(Box<SearchLecivaArgs>),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Czsukl(_) => "CZSUKL",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Czsukl(args) => args.build_payload(),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Czsukl(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Czsukl(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Czsukl(args) => args.order(),
        }
    }
}

pub(crate) fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Search { source } => execute_search(source),
        Commands::Get(args) => execute_get(args),
    }
}

fn execute_search(source: SearchSource) -> Result<(), CliError> {
    let config = Config::load_or_default(CODEXIS_PLUGIN_LECIVA_API_URL_ENV, DEFAULT_API_BASE_URL);
    let payload = source.build_payload()?;
    execute_search_request(
        &config.base_url,
        config.auth_header.as_deref(),
        source.source_code(),
        &payload,
        source.dry_run(),
        source.sort(),
        source.order(),
    )
}

fn execute_get(args: GetArgs) -> Result<(), CliError> {
    let config = Config::load_or_default(CODEXIS_PLUGIN_LECIVA_API_URL_ENV, DEFAULT_API_BASE_URL);
    execute_get_request(
        &config.base_url,
        config.auth_header.as_deref(),
        &args.resource,
        args.dry_run,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn parses_uppercase_source_and_flags() {
        let cli = Cli::try_parse_from([
            "leciva-cli",
            "search",
            "CZSUKL",
            "--query",
            "paralen",
            "--only-dodavka",
        ])
        .unwrap();
        match cli.command {
            Commands::Search {
                source: SearchSource::Czsukl(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("paralen"));
                assert!(args.only_dodavka);
            }
            _ => panic!("expected CZSUKL search"),
        }
    }

    #[test]
    fn accepts_lowercase_alias() {
        let cli = Cli::try_parse_from(["leciva-cli", "search", "czsukl", "--kod-sukl", "0000009"])
            .unwrap();
        match cli.command {
            Commands::Search {
                source: SearchSource::Czsukl(args),
            } => {
                assert_eq!(args.kod_sukl.as_deref(), Some("0000009"));
            }
            _ => panic!("expected CZSUKL search"),
        }
    }

    #[test]
    fn parses_get_command() {
        let cli = Cli::try_parse_from([
            "leciva-cli",
            "get",
            "--dry-run",
            "leciva://doc/CZSUKL0000009/meta",
        ])
        .unwrap();
        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "leciva://doc/CZSUKL0000009/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn root_help_lists_source_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CZSUKL"));
        assert!(help.contains("leciva-cli search CZSUKL --query"));
    }
}

use clap::{Parser, Subcommand};

use crate::core::config::{Config, CODEXIS_PLUGIN_REGISTR_SMLUV_API_URL_ENV, DEFAULT_API_BASE_URL};
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::smlouvy::{SearchSmlouvyArgs, SEARCH_SMLOUVY_HELP};

const ROOT_HELP: &str = "\
Data sources:
  CZRS     Registr smluv     Czech Register of Contracts (smlouvy.gov.cz)

Examples:
  registr-smluv search CZRS --query \"rekonstrukce silnice\" --limit 5
  registr-smluv search CZRS --publisher-ico 00064581 --date-from 2024-01-01
  registr-smluv search CZRS --ico 27074358 --value-min 1000000 --only-valid
  registr-smluv get registr-smluv://doc/CZRS34840741/meta

Detailed source help:
  registr-smluv search <DATA_SOURCE> --help
  registr-smluv search CZRS --help

Get resource:
  registr-smluv://doc/<DOC_ID>/meta";

#[derive(Parser, Debug)]
#[command(
    name = "registr-smluv",
    version,
    about = "registr-smluv CLI for contract search and registr-smluv:// resource fetches",
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
        about = "Search the Czech register of contracts",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a registr-smluv:// resource", arg_required_else_help = true)]
    Get(GetArgs),
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "CZRS",
        visible_alias = "czrs",
        about = "Czech Register of Contracts (smlouvy.gov.cz)",
        after_help = SEARCH_SMLOUVY_HELP,
        arg_required_else_help = true
    )]
    // Boxed so the enclosing enums stay small (the filter struct is the only large variant).
    Czrs(Box<SearchSmlouvyArgs>),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Czrs(_) => "CZRS",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Czrs(args) => args.build_payload(),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Czrs(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Czrs(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Czrs(args) => args.order(),
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
    let config = Config::load_or_default(
        CODEXIS_PLUGIN_REGISTR_SMLUV_API_URL_ENV,
        DEFAULT_API_BASE_URL,
    );
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
    let config = Config::load_or_default(
        CODEXIS_PLUGIN_REGISTR_SMLUV_API_URL_ENV,
        DEFAULT_API_BASE_URL,
    );
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
    fn cli_parses_uppercase_source_subcommand_and_flags() {
        let cli = Cli::try_parse_from([
            "registr-smluv",
            "search",
            "CZRS",
            "--query",
            "silnice",
            "--publisher-ico",
            "00064581",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czrs(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("silnice"));
                assert_eq!(args.publisher_ico.as_deref(), Some("00064581"));
            }
            _ => panic!("expected CZRS search command"),
        }
    }

    #[test]
    fn cli_accepts_lowercase_source_alias() {
        let cli =
            Cli::try_parse_from(["registr-smluv", "search", "czrs", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czrs(args),
            } => assert_eq!(args.base.query.as_deref(), Some("test")),
            _ => panic!("expected CZRS search command"),
        }
    }

    #[test]
    fn cli_parses_value_and_only_valid_filters() {
        let cli = Cli::try_parse_from([
            "registr-smluv",
            "search",
            "CZRS",
            "--query",
            "x",
            "--value-min",
            "1000000",
            "--value-max",
            "5000000",
            "--only-valid",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czrs(args),
            } => {
                assert_eq!(args.value_min, Some(1_000_000.0));
                assert_eq!(args.value_max, Some(5_000_000.0));
                assert!(args.only_valid);
            }
            _ => panic!("expected CZRS search command"),
        }
    }

    #[test]
    fn cli_parses_search_with_date_filters() {
        let cli = Cli::try_parse_from([
            "registr-smluv",
            "search",
            "CZRS",
            "--date-from",
            "2024-01-01",
            "--date-to",
            "2024-12-31",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czrs(args),
            } => {
                assert_eq!(args.date_from.as_deref(), Some("2024-01-01"));
                assert_eq!(args.date_to.as_deref(), Some("2024-12-31"));
            }
            _ => panic!("expected CZRS search command"),
        }
    }

    #[test]
    fn cli_parses_get_command() {
        let cli = Cli::try_parse_from([
            "registr-smluv",
            "get",
            "--dry-run",
            "registr-smluv://doc/CZRS1/meta",
        ])
        .unwrap();

        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "registr-smluv://doc/CZRS1/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn cli_parses_search_with_sort_and_order() {
        let cli = Cli::try_parse_from([
            "registr-smluv",
            "search",
            "CZRS",
            "--query",
            "test",
            "--sort",
            "date",
            "--order",
            "asc",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czrs(args),
            } => {
                assert_eq!(args.base.sort.as_deref(), Some("date"));
                assert_eq!(args.base.order.as_deref(), Some("asc"));
            }
            _ => panic!("expected CZRS search command"),
        }
    }

    #[test]
    fn root_help_mentions_data_sources_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CZRS     Registr smluv"));
        assert!(help.contains("registr-smluv search CZRS --query"));
        assert!(help.contains("registr-smluv get registr-smluv://doc/CZRS34840741/meta"));
        assert!(help.contains("registr-smluv search <DATA_SOURCE> --help"));
    }

    #[test]
    fn get_help_mentions_meta_resource() {
        let mut command = Cli::command();
        let get = command.find_subcommand_mut("get").unwrap();
        let help = get.render_long_help().to_string();

        assert!(help.contains("registr-smluv://doc/<DOC_ID>/meta"));
        // metadata-only v1: no legislation-style resources
        assert!(!help.contains("/toc"));
        assert!(!help.contains("/versions"));
    }

    #[test]
    fn search_help_uses_data_source_placeholder_and_heading() {
        let mut command = Cli::command();
        let search = command.find_subcommand_mut("search").unwrap();
        let help = search.render_long_help().to_string();

        assert!(help.contains("<DATA_SOURCE>"));
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CZRS"));
    }
}

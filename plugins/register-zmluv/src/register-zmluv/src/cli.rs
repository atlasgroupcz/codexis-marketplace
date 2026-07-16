use clap::{Parser, Subcommand};

use crate::core::config::{Config, CODEXIS_PLUGIN_REGISTER_ZMLUV_API_URL_ENV, DEFAULT_API_BASE_URL};
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::zmluvy::{SearchZmluvyArgs, SEARCH_ZMLUVY_HELP};

const ROOT_HELP: &str = "\
Data sources:
  SKCRZ    Register zmlúv    Slovak Register of Contracts (crz.gov.sk)

Examples:
  register-zmluv search SKCRZ --query \"údržba\" --limit 5
  register-zmluv search SKCRZ --ico 00151513 --date-from 2024-01-01
  register-zmluv search SKCRZ --query \"nájomná zmluva\" --value-min 100000
  register-zmluv get register-zmluv://doc/SKCRZ123456/meta

Detailed source help:
  register-zmluv search <DATA_SOURCE> --help
  register-zmluv search SKCRZ --help

Get resource:
  register-zmluv://doc/<DOC_ID>/meta";

#[derive(Parser, Debug)]
#[command(
    name = "register-zmluv",
    version,
    about = "register-zmluv CLI for contract search and register-zmluv:// resource fetches",
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
        about = "Search the Slovak register of contracts",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a register-zmluv:// resource", arg_required_else_help = true)]
    Get(GetArgs),
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "SKCRZ",
        visible_alias = "skcrz",
        about = "Slovak Register of Contracts (crz.gov.sk)",
        after_help = SEARCH_ZMLUVY_HELP,
        arg_required_else_help = true
    )]
    // Boxed so the enclosing enums stay small (the filter struct is the only large variant).
    Skcrz(Box<SearchZmluvyArgs>),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Skcrz(_) => "SKCRZ",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Skcrz(args) => args.build_payload(),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Skcrz(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Skcrz(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Skcrz(args) => args.order(),
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
        CODEXIS_PLUGIN_REGISTER_ZMLUV_API_URL_ENV,
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
        CODEXIS_PLUGIN_REGISTER_ZMLUV_API_URL_ENV,
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
            "register-zmluv",
            "search",
            "SKCRZ",
            "--query",
            "údržba",
            "--ico",
            "00151513",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skcrz(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("údržba"));
                assert_eq!(args.ico.as_deref(), Some("00151513"));
            }
            _ => panic!("expected SKCRZ search command"),
        }
    }

    #[test]
    fn cli_accepts_lowercase_source_alias() {
        let cli =
            Cli::try_parse_from(["register-zmluv", "search", "skcrz", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skcrz(args),
            } => assert_eq!(args.base.query.as_deref(), Some("test")),
            _ => panic!("expected SKCRZ search command"),
        }
    }

    #[test]
    fn cli_parses_value_and_rezort_filters() {
        let cli = Cli::try_parse_from([
            "register-zmluv",
            "search",
            "SKCRZ",
            "--query",
            "x",
            "--value-min",
            "100000",
            "--value-max",
            "500000",
            "--rezort",
            "114573",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skcrz(args),
            } => {
                assert_eq!(args.value_min, Some(100_000.0));
                assert_eq!(args.value_max, Some(500_000.0));
                assert_eq!(args.rezort.as_deref(), Some("114573"));
            }
            _ => panic!("expected SKCRZ search command"),
        }
    }

    #[test]
    fn ico_flag_is_canonicalized_at_parse() {
        // A user-supplied IČO missing leading zeros is padded to 8 digits so it
        // matches the backend's zero-padded stored value.
        let cli = Cli::try_parse_from([
            "register-zmluv",
            "search",
            "SKCRZ",
            "--ico",
            "151513",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skcrz(args),
            } => {
                assert_eq!(args.ico.as_deref(), Some("00151513"));
            }
            _ => panic!("expected SKCRZ search command"),
        }
    }

    #[test]
    fn cli_parses_search_with_date_filters() {
        let cli = Cli::try_parse_from([
            "register-zmluv",
            "search",
            "SKCRZ",
            "--date-from",
            "2024-01-01",
            "--date-to",
            "2024-12-31",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skcrz(args),
            } => {
                assert_eq!(args.date_from.as_deref(), Some("2024-01-01"));
                assert_eq!(args.date_to.as_deref(), Some("2024-12-31"));
            }
            _ => panic!("expected SKCRZ search command"),
        }
    }

    #[test]
    fn cli_parses_get_command() {
        let cli = Cli::try_parse_from([
            "register-zmluv",
            "get",
            "--dry-run",
            "register-zmluv://doc/SKCRZ1/meta",
        ])
        .unwrap();

        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "register-zmluv://doc/SKCRZ1/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn cli_parses_search_with_sort_and_order() {
        let cli = Cli::try_parse_from([
            "register-zmluv",
            "search",
            "SKCRZ",
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
                source: SearchSource::Skcrz(args),
            } => {
                assert_eq!(args.base.sort.as_deref(), Some("date"));
                assert_eq!(args.base.order.as_deref(), Some("asc"));
            }
            _ => panic!("expected SKCRZ search command"),
        }
    }

    // The CZ sibling's filters must not exist here — the SK request DTO has no
    // publisher/counterparty split and no validity flag.
    #[test]
    fn cli_rejects_czech_only_filters() {
        assert!(Cli::try_parse_from([
            "register-zmluv",
            "search",
            "SKCRZ",
            "--publisher-ico",
            "00151513",
        ])
        .is_err());
        assert!(
            Cli::try_parse_from(["register-zmluv", "search", "SKCRZ", "--only-valid"]).is_err()
        );
    }

    #[test]
    fn root_help_mentions_data_sources_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("SKCRZ    Register zmlúv"));
        assert!(help.contains("register-zmluv search SKCRZ --query"));
        assert!(help.contains("register-zmluv get register-zmluv://doc/SKCRZ123456/meta"));
        assert!(help.contains("register-zmluv search <DATA_SOURCE> --help"));
    }

    #[test]
    fn get_help_mentions_meta_resource() {
        let mut command = Cli::command();
        let get = command.find_subcommand_mut("get").unwrap();
        let help = get.render_long_help().to_string();

        assert!(help.contains("register-zmluv://doc/<DOC_ID>/meta"));
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
        assert!(help.contains("SKCRZ"));
    }
}

use clap::{Parser, Subcommand};

use crate::core::config::{Config, CODEXIS_PLUGIN_RZPRO_API_URL_ENV, DEFAULT_API_BASE_URL};
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::osoby::{SearchOsobyArgs, SEARCH_OSOBY_HELP};
use crate::sources::rzpro::{SearchRzproArgs, SEARCH_RZPRO_HELP};

const ROOT_HELP: &str = "\
Data sources:
  CZRZP      Registr ZP        Czech medical devices (eregpublicsecure.ksrzis.cz)
  CZRZP-OS   Registrované osoby   RZPRO registered persons/companies

Examples:
  rzpro-cli search CZRZP --query \"gliadin\"
  rzpro-cli search CZRZP --druh \"in vitro\" --riziko III
  rzpro-cli search CZRZP-OS --query \"Medtronic\"
  rzpro-cli get rzpro://doc/CZRZP00000019/meta
  rzpro-cli get rzpro://osoba/002276

Detailed source help:
  rzpro-cli search CZRZP --help
  rzpro-cli search CZRZP-OS --help

Get resource:
  rzpro://doc/<DOC_ID>/meta   ·   rzpro://osoba/<KEY>   ·   rzpro://doc/values/<FIELD>";

#[derive(Parser, Debug)]
#[command(
    name = "rzpro-cli",
    version,
    about = "rzpro-cli for RZPRO device/person search and rzpro:// resource fetches",
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
        about = "Search the Czech register of medical devices",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a rzpro:// resource", arg_required_else_help = true)]
    Get(GetArgs),
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "CZRZP",
        visible_alias = "czrzp",
        about = "Czech medical devices (eregpublicsecure.ksrzis.cz)",
        after_help = SEARCH_RZPRO_HELP,
        arg_required_else_help = true
    )]
    Czrzp(Box<SearchRzproArgs>),

    #[command(
        name = "CZRZP-OS",
        visible_alias = "czrzp-os",
        about = "RZPRO registered persons/companies",
        after_help = SEARCH_OSOBY_HELP,
        arg_required_else_help = true
    )]
    CzrzpOs(Box<SearchOsobyArgs>),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Czrzp(_) => "CZRZP",
            Self::CzrzpOs(_) => "CZRZP-OS",
        }
    }

    /// The selected source's filter args as a trait object. Payload building and
    /// the sort/order/dry-run accessors all go through the shared trait, so only
    /// `source_code` needs its own per-variant match.
    fn args(&self) -> &dyn SearchPayloadArgs {
        match self {
            Self::Czrzp(args) => args.as_ref(),
            Self::CzrzpOs(args) => args.as_ref(),
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
    let config = Config::load_or_default(CODEXIS_PLUGIN_RZPRO_API_URL_ENV, DEFAULT_API_BASE_URL);
    let args = source.args();
    let payload = args.build_payload()?;
    execute_search_request(
        &config.base_url,
        config.auth_header.as_deref(),
        source.source_code(),
        &payload,
        args.dry_run(),
        args.sort(),
        args.order(),
    )
}

fn execute_get(args: GetArgs) -> Result<(), CliError> {
    let config = Config::load_or_default(CODEXIS_PLUGIN_RZPRO_API_URL_ENV, DEFAULT_API_BASE_URL);
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
    fn parses_device_search_and_filters() {
        let cli = Cli::try_parse_from([
            "rzpro-cli",
            "search",
            "CZRZP",
            "--query",
            "gliadin",
            "--riziko",
            "III",
        ])
        .unwrap();
        match cli.command {
            Commands::Search {
                source: SearchSource::Czrzp(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("gliadin"));
                assert_eq!(args.riziko.as_deref(), Some("III"));
            }
            _ => panic!("expected CZRZP search"),
        }
    }

    #[test]
    fn parses_person_search_source() {
        let cli = Cli::try_parse_from(["rzpro-cli", "search", "CZRZP-OS", "--query", "Medtronic"])
            .unwrap();
        match cli.command {
            Commands::Search {
                source: SearchSource::CzrzpOs(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("Medtronic"));
            }
            _ => panic!("expected CZRZP-OS search"),
        }
    }

    #[test]
    fn ico_flag_is_canonicalized_at_parse() {
        // A user-supplied IČO missing leading zeros is padded to 8 digits so it
        // matches the backend's zero-padded stored value.
        let cli = Cli::try_parse_from(["rzpro-cli", "search", "CZRZP", "--ico", "565474"]).unwrap();
        match cli.command {
            Commands::Search {
                source: SearchSource::Czrzp(args),
            } => {
                assert_eq!(args.ico.as_deref(), Some("00565474"));
            }
            _ => panic!("expected CZRZP search"),
        }
    }

    #[test]
    fn parses_get_osoba() {
        let cli = Cli::try_parse_from(["rzpro-cli", "get", "rzpro://osoba/002276"]).unwrap();
        match cli.command {
            Commands::Get(args) => assert_eq!(args.resource, "rzpro://osoba/002276"),
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn root_help_lists_both_sources() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("CZRZP"));
        assert!(help.contains("CZRZP-OS"));
        assert!(help.contains("rzpro-cli get rzpro://osoba/002276"));
    }
}

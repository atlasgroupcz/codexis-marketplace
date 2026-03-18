use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::execute_search_request;
use crate::sources::all::{SearchAllArgs, SEARCH_ALL_HELP};
use crate::sources::comment::{SearchCommentArgs, SEARCH_COMMENT_HELP};
use crate::sources::common::SearchPayloadArgs;
use crate::sources::cr::{SearchCrArgs, SEARCH_CR_HELP};
use crate::sources::es::{SearchEsArgs, SEARCH_ES_HELP};
use crate::sources::eu::{SearchEuArgs, SEARCH_EU_HELP};
use crate::sources::jd::{SearchJdArgs, SEARCH_JD_HELP};
use crate::sources::lt::{SearchLtArgs, SEARCH_LT_HELP};
use crate::sources::sk::{SearchSkArgs, SEARCH_SK_HELP};
use crate::sources::vs::{SearchVsArgs, SEARCH_VS_HELP};

const GLOBAL_AFTER_HELP: &str = r#"Examples:
  cdx-cli search JD --query "náhrada škody" --limit 5
  cdx-cli search ALL '{"query":"insolvence","limit":5}'
  cdx-cli search JD --help

Configuration:
  Reads CODEXIS_API_URL and CDX_API_JWT_AUTH from the environment or ~/.cdx/.env."#;

const SEARCH_AFTER_HELP: &str = r#"Sources:
  ALL      Search across all data sources
  COMMENT  Legal commentaries
  CR       Czech legislation
  ES       EU court decisions
  EU       EU legislation
  JD       Czech case law
  LT       Legal literature
  SK       Slovak legislation
  VS       Contract templates

Examples:
  cdx-cli search JD --query "náhrada škody" --court "Nejvyšší soud" --limit 5
  cdx-cli search CR --query "občanský zákoník" --type Zákon --current
  cdx-cli search JD '{"query":"náhrada škody","limit":5}'
  cat payload.json | cdx-cli search CR -

Formats:
  Dates: YYYY-MM-DD
  JSON booleans: true / false
  JSON sort fields: use sort / sortOrder across all sources
  CLI boolean filters: presence-only flags, for example --current
  Defaults: limit 10, offset 1, sort RELEVANCE, sort-order DESC

Flags map to JSON fields. If JSON_PAYLOAD is also provided, matching keys from JSON win.
Backend-specific request fields are handled internally, for example CR sort -> sortBy."#;

#[derive(Parser, Debug)]
#[command(
    name = "cdx-cli",
    version,
    about = "CODEXIS CLI for source-oriented search",
    after_help = GLOBAL_AFTER_HELP,
    disable_help_subcommand = true,
    subcommand_required = true,
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Search one CODEXIS data source",
        after_help = SEARCH_AFTER_HELP,
        arg_required_else_help = true
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "ALL",
        visible_alias = "all",
        about = "Search across all data sources",
        after_help = SEARCH_ALL_HELP,
        arg_required_else_help = true
    )]
    All(SearchAllArgs),

    #[command(
        name = "COMMENT",
        visible_alias = "comment",
        about = "Search legal commentaries",
        after_help = SEARCH_COMMENT_HELP,
        arg_required_else_help = true
    )]
    Comment(SearchCommentArgs),

    #[command(
        name = "CR",
        visible_alias = "cr",
        about = "Search Czech legislation",
        after_help = SEARCH_CR_HELP,
        arg_required_else_help = true
    )]
    Cr(SearchCrArgs),

    #[command(
        name = "ES",
        visible_alias = "es",
        about = "Search EU court decisions",
        after_help = SEARCH_ES_HELP,
        arg_required_else_help = true
    )]
    Es(SearchEsArgs),

    #[command(
        name = "EU",
        visible_alias = "eu",
        about = "Search EU legislation",
        after_help = SEARCH_EU_HELP,
        arg_required_else_help = true
    )]
    Eu(SearchEuArgs),

    #[command(
        name = "JD",
        visible_alias = "jd",
        about = "Search Czech case law",
        after_help = SEARCH_JD_HELP,
        arg_required_else_help = true
    )]
    Jd(SearchJdArgs),

    #[command(
        name = "LT",
        visible_alias = "lt",
        about = "Search legal literature",
        after_help = SEARCH_LT_HELP,
        arg_required_else_help = true
    )]
    Lt(SearchLtArgs),

    #[command(
        name = "SK",
        visible_alias = "sk",
        about = "Search Slovak legislation",
        after_help = SEARCH_SK_HELP,
        arg_required_else_help = true
    )]
    Sk(SearchSkArgs),

    #[command(
        name = "VS",
        visible_alias = "vs",
        about = "Search contract templates",
        after_help = SEARCH_VS_HELP,
        arg_required_else_help = true
    )]
    Vs(SearchVsArgs),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::All(_) => "ALL",
            Self::Comment(_) => "COMMENT",
            Self::Cr(_) => "CR",
            Self::Es(_) => "ES",
            Self::Eu(_) => "EU",
            Self::Jd(_) => "JD",
            Self::Lt(_) => "LT",
            Self::Sk(_) => "SK",
            Self::Vs(_) => "VS",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::All(args) => args.build_payload("ALL"),
            Self::Comment(args) => args.build_payload("COMMENT"),
            Self::Cr(args) => args.build_payload("CR"),
            Self::Es(args) => args.build_payload("ES"),
            Self::Eu(args) => args.build_payload("EU"),
            Self::Jd(args) => args.build_payload("JD"),
            Self::Lt(args) => args.build_payload("LT"),
            Self::Sk(args) => args.build_payload("SK"),
            Self::Vs(args) => args.build_payload("VS"),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::All(args) => args.dry_run(),
            Self::Comment(args) => args.dry_run(),
            Self::Cr(args) => args.dry_run(),
            Self::Es(args) => args.dry_run(),
            Self::Eu(args) => args.dry_run(),
            Self::Jd(args) => args.dry_run(),
            Self::Lt(args) => args.dry_run(),
            Self::Sk(args) => args.dry_run(),
            Self::Vs(args) => args.dry_run(),
        }
    }
}

pub(crate) fn run(cli: Cli) -> Result<(), CliError> {
    let config = Config::load()?;

    match cli.command {
        Commands::Search { source } => execute_search(&config, source),
    }
}

fn execute_search(config: &Config, source: SearchSource) -> Result<(), CliError> {
    let payload = source.build_payload()?;
    execute_search_request(
        &config.base_url,
        &config.auth_header,
        source.source_code(),
        &payload,
        source.dry_run(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_uppercase_source_subcommand_and_flags() {
        let cli = Cli::try_parse_from([
            "cdx-cli",
            "search",
            "JD",
            "--query",
            "test",
            "--court",
            "Nejvyšší soud",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Jd(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.courts, vec!["Nejvyšší soud"]);
            }
            _ => panic!("expected JD search command"),
        }
    }

    #[test]
    fn cli_accepts_native_flag_aliases() {
        let cli = Cli::try_parse_from([
            "cdx-cli",
            "search",
            "JD",
            "--query",
            "test",
            "--soud",
            "Nejvyšší soud",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Jd(args),
            } => assert_eq!(args.courts, vec!["Nejvyšší soud"]),
            _ => panic!("expected JD search command"),
        }
    }

    #[test]
    fn cli_accepts_lowercase_source_alias() {
        let cli = Cli::try_parse_from(["cdx-cli", "search", "jd", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Jd(args),
            } => assert_eq!(args.base.query.as_deref(), Some("test")),
            _ => panic!("expected JD search command"),
        }
    }
}

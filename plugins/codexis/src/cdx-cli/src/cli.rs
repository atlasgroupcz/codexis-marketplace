use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::{execute_search_request, SearchFacetMode};
use crate::core::schema::{render_search_schema, SearchSchemaKind};
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

#[derive(Parser, Debug)]
#[command(
    name = "cdx-cli",
    version,
    about = "CODEXIS CLI for source-oriented search",
    disable_version_flag = true,
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

    fn facet_mode(&self) -> SearchFacetMode {
        match self {
            Self::All(args) => args.facet_mode(),
            Self::Comment(args) => args.facet_mode(),
            Self::Cr(args) => args.facet_mode(),
            Self::Es(args) => args.facet_mode(),
            Self::Eu(args) => args.facet_mode(),
            Self::Jd(args) => args.facet_mode(),
            Self::Lt(args) => args.facet_mode(),
            Self::Sk(args) => args.facet_mode(),
            Self::Vs(args) => args.facet_mode(),
        }
    }

    fn validate_schema_request(&self) -> Result<Option<SearchSchemaKind>, CliError> {
        match self {
            Self::All(args) => args.validate_schema_request(),
            Self::Comment(args) => args.validate_schema_request(),
            Self::Cr(args) => args.validate_schema_request(),
            Self::Es(args) => args.validate_schema_request(),
            Self::Eu(args) => args.validate_schema_request(),
            Self::Jd(args) => args.validate_schema_request(),
            Self::Lt(args) => args.validate_schema_request(),
            Self::Sk(args) => args.validate_schema_request(),
            Self::Vs(args) => args.validate_schema_request(),
        }
    }
}

pub(crate) fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Search { source } => execute_search(source),
    }
}

fn execute_search(source: SearchSource) -> Result<(), CliError> {
    if let Some(kind) = source.validate_schema_request()? {
        println!("{}", render_search_schema(source.source_code(), kind)?);
        return Ok(());
    }

    let config = Config::load()?;
    let payload = source.build_payload()?;
    execute_search_request(
        &config.base_url,
        &config.auth_header,
        source.source_code(),
        &payload,
        source.dry_run(),
        source.facet_mode(),
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

    #[test]
    fn cli_accepts_schema_input_flag() {
        let cli = Cli::try_parse_from(["cdx-cli", "search", "JD", "--schema-input"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Jd(args),
            } => assert!(args.base.schema_input),
            _ => panic!("expected JD search command"),
        }
    }

    #[test]
    fn cli_accepts_with_facets_flag() {
        let cli = Cli::try_parse_from([
            "cdx-cli",
            "search",
            "JD",
            "--with-facets",
            "--query",
            "test",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Jd(args),
            } => assert!(args.facets.with_facets),
            _ => panic!("expected JD search command"),
        }
    }

    #[test]
    fn cli_accepts_with_full_facets_flag() {
        let cli = Cli::try_parse_from([
            "cdx-cli",
            "search",
            "JD",
            "--with-full-facets",
            "--query",
            "test",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Jd(args),
            } => assert!(args.facets.with_full_facets),
            _ => panic!("expected JD search command"),
        }
    }

    #[test]
    fn cli_rejects_facet_flags_for_all_search() {
        let error = Cli::try_parse_from([
            "cdx-cli",
            "search",
            "ALL",
            "--with-facets",
            "--query",
            "test",
        ])
        .unwrap_err();

        let rendered = error.to_string();
        assert!(rendered.contains("--with-facets"));
        assert!(rendered.contains("unexpected"));
    }

    #[test]
    fn cli_rejects_conflicting_facet_flags() {
        let error = Cli::try_parse_from([
            "cdx-cli",
            "search",
            "JD",
            "--with-facets",
            "--with-full-facets",
            "--query",
            "test",
        ])
        .unwrap_err();

        let rendered = error.to_string();
        assert!(rendered.contains("--with-full-facets"));
        assert!(rendered.contains("cannot be used with"));
    }

    #[test]
    fn search_source_uses_hidden_facets_for_all() {
        let cli = Cli::try_parse_from(["cdx-cli", "search", "ALL", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::All(args),
            } => assert_eq!(args.facet_mode(), SearchFacetMode::Hidden),
            _ => panic!("expected ALL search command"),
        }
    }
}

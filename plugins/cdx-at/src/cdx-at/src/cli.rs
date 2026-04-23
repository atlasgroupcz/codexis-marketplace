use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::core::schema::{render_resource_schema, ResourceSchemaKind, SchemaSource};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::atbr::{SearchAtbrArgs, SEARCH_ATBR_HELP};
use crate::sources::atjd::{SearchAtjdArgs, SEARCH_ATJD_HELP};
use crate::sources::atlr::{SearchAtlrArgs, SEARCH_ATLR_HELP};
use crate::sources::atso::{SearchAtsoArgs, SEARCH_ATSO_HELP};
use crate::sources::athi::{SearchAthiArgs, SEARCH_ATHI_HELP};

const ROOT_HELP: &str = "\
Data sources:
  ATBR     Federal Legislation  Austrian federal laws and regulations (Bundesrecht)
  ATJD     Case Law             Austrian court decisions (Judikatur)
  ATLR     State Legislation    Austrian state laws and regulations (Landesrecht)
  ATSO     Miscellaneous        Austrian miscellaneous legal documents (Sonstige)
  ATHI     History              Austrian consolidated federal norms history

Examples:
  cdx-at search ATBR --query \"Verordnung\" --limit 5
  cdx-at search ATJD --query \"Grundrechte\" --application Vfgh --limit 5
  cdx-at search ATHI --abbreviation ASVG
  cdx-at get cdx-at://doc/ATBR1234/meta
  cdx-at get cdx-at://doc/ATJD5678/text
  cdx-at get cdx-at://resolve/ATBR1234

Detailed source help:
  cdx-at search <DATA_SOURCE> --help
  cdx-at search [ATBR|ATJD|ATLR|ATSO|ATHI] --help

Endpoint schema help:
  cdx-at schema [meta|text]
  cdx-at schema meta [ATBR|ATJD|ATLR|ATSO|ATHI]

Common get resource suffixes:
  /meta, /text, /attachment/<FILE>";

#[derive(Parser, Debug)]
#[command(
    name = "cdx-at",
    version,
    about = "CDX-AT CLI for search, cdx-at:// resource fetches, and endpoint schemas",
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
        about = "Search one Austrian data source",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a cdx-at:// resource", arg_required_else_help = true)]
    Get(GetArgs),

    #[command(
        about = "Print cdx-at-oriented output schema and query parameters for get endpoints",
        arg_required_else_help = true,
        subcommand_value_name = "ENDPOINT",
        subcommand_help_heading = "Schema endpoints"
    )]
    Schema {
        #[command(subcommand)]
        endpoint: ResourceSchemaCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ResourceSchemaCommand {
    #[command(
        about = "Output schema for /meta",
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Schema sources"
    )]
    Meta {
        #[command(subcommand)]
        source: Option<SchemaSourceCommand>,
    },

    #[command(about = "Output schema for /text")]
    Text,
}

impl ResourceSchemaCommand {
    fn kind(&self) -> ResourceSchemaKind {
        match self {
            Self::Meta { .. } => ResourceSchemaKind::Meta,
            Self::Text => ResourceSchemaKind::Text,
        }
    }

    fn schema_source(&self) -> Option<SchemaSource> {
        match self {
            Self::Meta { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Text => None,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
enum SchemaSourceCommand {
    #[command(name = "ATBR", visible_alias = "atbr", about = "ATBR schema")]
    Atbr,

    #[command(name = "ATJD", visible_alias = "atjd", about = "ATJD schema")]
    Atjd,

    #[command(name = "ATLR", visible_alias = "atlr", about = "ATLR schema")]
    Atlr,

    #[command(name = "ATSO", visible_alias = "atso", about = "ATSO schema")]
    Atso,

    #[command(name = "ATHI", visible_alias = "athi", about = "ATHI schema")]
    Athi,
}

impl SchemaSourceCommand {
    fn kind(&self) -> SchemaSource {
        match self {
            Self::Atbr => SchemaSource::Atbr,
            Self::Atjd => SchemaSource::Atjd,
            Self::Atlr => SchemaSource::Atlr,
            Self::Atso => SchemaSource::Atso,
            Self::Athi => SchemaSource::Athi,
        }
    }
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "ATBR",
        visible_alias = "atbr",
        about = "Federal Legislation: Austrian federal laws and regulations (Bundesrecht)",
        after_help = SEARCH_ATBR_HELP,
        arg_required_else_help = true
    )]
    Atbr(SearchAtbrArgs),

    #[command(
        name = "ATJD",
        visible_alias = "atjd",
        about = "Case Law: Austrian court decisions (Judikatur)",
        after_help = SEARCH_ATJD_HELP,
        arg_required_else_help = true
    )]
    Atjd(SearchAtjdArgs),

    #[command(
        name = "ATLR",
        visible_alias = "atlr",
        about = "State Legislation: Austrian state laws and regulations (Landesrecht)",
        after_help = SEARCH_ATLR_HELP,
        arg_required_else_help = true
    )]
    Atlr(SearchAtlrArgs),

    #[command(
        name = "ATSO",
        visible_alias = "atso",
        about = "Miscellaneous: Austrian miscellaneous legal documents (Sonstige)",
        after_help = SEARCH_ATSO_HELP,
        arg_required_else_help = true
    )]
    Atso(SearchAtsoArgs),

    #[command(
        name = "ATHI",
        visible_alias = "athi",
        about = "History: Austrian consolidated federal norms history",
        after_help = SEARCH_ATHI_HELP,
        arg_required_else_help = true
    )]
    Athi(SearchAthiArgs),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Atbr(_) => "ATBR",
            Self::Atjd(_) => "ATJD",
            Self::Atlr(_) => "ATLR",
            Self::Atso(_) => "ATSO",
            Self::Athi(_) => "ATHI",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Atbr(args) => args.build_payload("ATBR"),
            Self::Atjd(args) => args.build_payload("ATJD"),
            Self::Atlr(args) => args.build_payload("ATLR"),
            Self::Atso(args) => args.build_payload("ATSO"),
            Self::Athi(args) => args.build_payload("ATHI"),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Atbr(args) => args.dry_run(),
            Self::Atjd(args) => args.dry_run(),
            Self::Atlr(args) => args.dry_run(),
            Self::Atso(args) => args.dry_run(),
            Self::Athi(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Atbr(args) => args.sort(),
            Self::Atjd(args) => args.sort(),
            Self::Atlr(args) => args.sort(),
            Self::Atso(args) => args.sort(),
            Self::Athi(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Atbr(args) => args.order(),
            Self::Atjd(args) => args.order(),
            Self::Atlr(args) => args.order(),
            Self::Atso(args) => args.order(),
            Self::Athi(args) => args.order(),
        }
    }
}

pub(crate) fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Search { source } => execute_search(source),
        Commands::Get(args) => execute_get(args),
        Commands::Schema { endpoint } => execute_schema(endpoint),
    }
}

fn execute_search(source: SearchSource) -> Result<(), CliError> {
    let config = Config::load()?;
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
    let config = Config::load()?;
    execute_get_request(&config.base_url, config.auth_header.as_deref(), &args.resource, args.dry_run)
}

fn execute_schema(endpoint: ResourceSchemaCommand) -> Result<(), CliError> {
    println!(
        "{}",
        render_resource_schema(endpoint.kind(), endpoint.schema_source())?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_parses_uppercase_source_subcommand_and_flags() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATBR",
            "--query",
            "test",
            "--document-type",
            "Verordnung",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Atbr(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.document_type.as_deref(), Some("Verordnung"));
            }
            _ => panic!("expected ATBR search command"),
        }
    }

    #[test]
    fn cli_accepts_lowercase_source_alias() {
        let cli =
            Cli::try_parse_from(["cdx-at", "search", "atbr", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Atbr(args),
            } => assert_eq!(args.base.query.as_deref(), Some("test")),
            _ => panic!("expected ATBR search command"),
        }
    }

    #[test]
    fn cli_parses_atjd_search_with_application() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATJD",
            "--query",
            "test",
            "--application",
            "Vfgh",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Atjd(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.application.as_deref(), Some("Vfgh"));
            }
            _ => panic!("expected ATJD search command"),
        }
    }

    #[test]
    fn cli_parses_atjd_search_with_decision_type() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATJD",
            "--query",
            "test",
            "--decision-type",
            "Erkenntnis",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Atjd(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.decision_type.as_deref(), Some("Erkenntnis"));
            }
            _ => panic!("expected ATJD search command"),
        }
    }

    #[test]
    fn cli_parses_atlr_search_with_state() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATLR",
            "--query",
            "test",
            "--state",
            "Tirol",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Atlr(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.state.as_deref(), Some("Tirol"));
            }
            _ => panic!("expected ATLR search command"),
        }
    }

    #[test]
    fn cli_parses_atso_search_with_application() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATSO",
            "--query",
            "test",
            "--application",
            "Erlaesse",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Atso(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.application.as_deref(), Some("Erlaesse"));
            }
            _ => panic!("expected ATSO search command"),
        }
    }

    #[test]
    fn cli_parses_athi_search_with_abbreviation() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATHI",
            "--abbreviation",
            "ASVG",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Athi(args),
            } => {
                assert_eq!(args.abbreviation.as_deref(), Some("ASVG"));
            }
            _ => panic!("expected ATHI search command"),
        }
    }

    #[test]
    fn cli_parses_get_command() {
        let cli =
            Cli::try_parse_from(["cdx-at", "get", "--dry-run", "cdx-at://doc/ATBR1/meta"])
                .unwrap();

        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "cdx-at://doc/ATBR1/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn cli_parses_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-at", "schema", "meta"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::Meta { source: None },
            } => {}
            _ => panic!("expected schema meta command"),
        }
    }

    #[test]
    fn cli_parses_typed_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-at", "schema", "meta", "ATBR"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Meta {
                        source: Some(SchemaSourceCommand::Atbr),
                    },
            } => {}
            _ => panic!("expected schema meta ATBR command"),
        }
    }

    #[test]
    fn cli_parses_schema_text_command() {
        let cli = Cli::try_parse_from(["cdx-at", "schema", "text"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::Text,
            } => {}
            _ => panic!("expected schema text command"),
        }
    }

    #[test]
    fn cli_parses_search_with_sort_and_order() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATBR",
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
                source: SearchSource::Atbr(args),
            } => {
                assert_eq!(args.base.sort.as_deref(), Some("date"));
                assert_eq!(args.base.order.as_deref(), Some("asc"));
            }
            _ => panic!("expected ATBR search command"),
        }
    }

    #[test]
    fn cli_parses_search_with_date_range() {
        let cli = Cli::try_parse_from([
            "cdx-at",
            "search",
            "ATBR",
            "--query",
            "test",
            "--date-from",
            "2024-01-01",
            "--date-to",
            "2024-12-31",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Atbr(args),
            } => {
                assert_eq!(args.dates.date_from.as_deref(), Some("2024-01-01"));
                assert_eq!(args.dates.date_to.as_deref(), Some("2024-12-31"));
            }
            _ => panic!("expected ATBR search command"),
        }
    }

    #[test]
    fn root_help_mentions_data_sources_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("ATBR     Federal Legislation"));
        assert!(help.contains("ATJD     Case Law"));
        assert!(help.contains("ATLR     State Legislation"));
        assert!(help.contains("ATSO     Miscellaneous"));
        assert!(help.contains("ATHI     History"));
        assert!(help.contains("cdx-at search ATBR --query"));
        assert!(help.contains("cdx-at get cdx-at://doc/ATBR1234/meta"));
        assert!(help.contains("cdx-at search <DATA_SOURCE> --help"));
        assert!(help.contains("cdx-at schema [meta|text]"));
    }

    #[test]
    fn get_help_mentions_document_resources() {
        let mut command = Cli::command();
        let get = command.find_subcommand_mut("get").unwrap();
        let help = get.render_long_help().to_string();

        assert!(help.contains("cdx-at://doc/<DOC_ID>/meta"));
        assert!(help.contains("cdx-at://doc/<DOC_ID>/text"));
        assert!(help.contains("cdx-at://doc/<DOC_ID>/attachment/<FILE>"));
        assert!(help.contains("cdx-at://resolve/<ID>"));
        // No law routes for AT
        assert!(!help.contains("law/"));
    }

    #[test]
    fn schema_help_lists_expected_endpoint_subcommands() {
        let mut command = Cli::command();
        let schema = command.find_subcommand_mut("schema").unwrap();
        let help = schema.render_long_help().to_string();

        assert!(help.contains("<ENDPOINT>"));
        assert!(help.contains("Schema endpoints:"));
        assert!(help.contains("meta"));
        assert!(help.contains("text"));
        // No toc/parts/versions/related for AT
        assert!(!help.contains("toc"));
        assert!(!help.contains("parts"));
        assert!(!help.contains("versions"));
        assert!(!help.contains("related"));
    }

    #[test]
    fn schema_meta_help_lists_source_subcommands() {
        let mut command = Cli::command();
        let schema = command.find_subcommand_mut("schema").unwrap();
        let meta = schema.find_subcommand_mut("meta").unwrap();
        let help = meta.render_long_help().to_string();

        assert!(help.contains("[DATA_SOURCE]"));
        assert!(help.contains("Schema sources:"));
        assert!(help.contains("ATBR"));
        assert!(help.contains("ATJD"));
        assert!(help.contains("ATLR"));
        assert!(help.contains("ATSO"));
        assert!(help.contains("ATHI"));
    }

    #[test]
    fn search_help_uses_data_source_placeholder_and_heading() {
        let mut command = Cli::command();
        let search = command.find_subcommand_mut("search").unwrap();
        let help = search.render_long_help().to_string();

        assert!(help.contains("<DATA_SOURCE>"));
        assert!(help.contains("Data sources:"));
        assert!(help.contains("ATBR"));
        assert!(help.contains("ATJD"));
        assert!(help.contains("ATLR"));
        assert!(help.contains("ATSO"));
        assert!(help.contains("ATHI"));
    }
}

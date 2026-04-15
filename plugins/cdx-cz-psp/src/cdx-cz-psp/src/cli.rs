use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::core::schema::{render_resource_schema, ResourceSchemaKind, SchemaSource};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::czpspdok::{SearchCzpspdokArgs, SEARCH_CZPSPDOK_HELP};
use crate::sources::czpsppre::{SearchCzpsppreArgs, SEARCH_CZPSPPRE_HELP};

const ROOT_HELP: &str = "\
Data sources:
  CZPSPDOK   Parliamentary Documents   Reports, interpellations, EU docs (psp.cz)
  CZPSPPRE   Legislative Proposals     Bills, amendments, legislative history (psp.cz)

Examples:
  cdx-cz-psp search CZPSPDOK --query \"interpelace\" --election-period 10 --limit 5
  cdx-cz-psp search CZPSPPRE --query \"daně\" --type \"Vládní návrh zákona\" --limit 5
  cdx-cz-psp get cdx-cz-psp://doc/CZPSPDOK1234/meta
  cdx-cz-psp get cdx-cz-psp://doc/CZPSPPRE5678/text
  cdx-cz-psp get 'cdx-cz-psp://doc/CZPSPPRE1/related?type=AMENDS&limit=10'
  cdx-cz-psp get cdx-cz-psp://resolve/CZPSPDOK1234

Detailed source help:
  cdx-cz-psp search <DATA_SOURCE> --help
  cdx-cz-psp search [CZPSPDOK|CZPSPPRE] --help

Endpoint schema help:
  cdx-cz-psp schema [meta|text|toc|parts|related|related/counts|history]
  cdx-cz-psp schema meta [CZPSPDOK|CZPSPPRE]

Common get resource suffixes:
  /meta, /toc, /text, /parts
  /related, /related/counts, /history (CZPSPPRE only)";

#[derive(Parser, Debug)]
#[command(
    name = "cdx-cz-psp",
    version,
    about = "CDX-CZ-PSP CLI for Czech Parliament search, cdx-cz-psp:// resource fetches, and endpoint schemas",
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
        about = "Search one Czech Parliament data source",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a cdx-cz-psp:// resource", arg_required_else_help = true)]
    Get(GetArgs),

    #[command(
        about = "Print cdx-cz-psp-oriented output schema and query parameters for get endpoints",
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

    #[command(
        about = "Output schema for /toc",
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Schema sources"
    )]
    Toc {
        #[command(subcommand)]
        source: Option<SchemaSourceCommand>,
    },

    #[command(
        about = "Output schema for /parts",
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Schema sources"
    )]
    Parts {
        #[command(subcommand)]
        source: Option<SchemaSourceCommand>,
    },

    #[command(
        about = "Output schema for /related",
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Schema sources"
    )]
    Related {
        #[command(subcommand)]
        source: Option<SchemaSourceCommand>,
    },

    #[command(
        name = "related/counts",
        visible_alias = "related-counts",
        about = "Output schema for /related/counts"
    )]
    RelatedCounts,

    #[command(
        about = "Output schema for /history (CZPSPPRE only)",
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Schema sources"
    )]
    History {
        #[command(subcommand)]
        source: Option<SchemaSourceCommand>,
    },
}

impl ResourceSchemaCommand {
    fn kind(&self) -> ResourceSchemaKind {
        match self {
            Self::Meta { .. } => ResourceSchemaKind::Meta,
            Self::Text => ResourceSchemaKind::Text,
            Self::Toc { .. } => ResourceSchemaKind::Toc,
            Self::Parts { .. } => ResourceSchemaKind::Parts,
            Self::Related { .. } => ResourceSchemaKind::Related,
            Self::RelatedCounts => ResourceSchemaKind::RelatedCounts,
            Self::History { .. } => ResourceSchemaKind::History,
        }
    }

    fn schema_source(&self) -> Option<SchemaSource> {
        match self {
            Self::Meta { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Toc { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Parts { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Related { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::History { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Text | Self::RelatedCounts => None,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
enum SchemaSourceCommand {
    #[command(name = "CZPSPDOK", visible_alias = "czpspdok", about = "CZPSPDOK schema")]
    Czpspdok,

    #[command(name = "CZPSPPRE", visible_alias = "czpsppre", about = "CZPSPPRE schema")]
    Czpsppre,
}

impl SchemaSourceCommand {
    fn kind(&self) -> SchemaSource {
        match self {
            Self::Czpspdok => SchemaSource::Czpspdok,
            Self::Czpsppre => SchemaSource::Czpsppre,
        }
    }
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "CZPSPDOK",
        visible_alias = "czpspdok",
        about = "Parliamentary Documents: reports, interpellations, EU docs",
        after_help = SEARCH_CZPSPDOK_HELP,
        arg_required_else_help = true
    )]
    Czpspdok(SearchCzpspdokArgs),

    #[command(
        name = "CZPSPPRE",
        visible_alias = "czpsppre",
        about = "Legislative Proposals: bills, amendments, legislative history",
        after_help = SEARCH_CZPSPPRE_HELP,
        arg_required_else_help = true
    )]
    Czpsppre(SearchCzpsppreArgs),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Czpspdok(_) => "CZPSPDOK",
            Self::Czpsppre(_) => "CZPSPPRE",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Czpspdok(args) => args.build_payload("CZPSPDOK"),
            Self::Czpsppre(args) => args.build_payload("CZPSPPRE"),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Czpspdok(args) => args.dry_run(),
            Self::Czpsppre(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Czpspdok(args) => args.sort(),
            Self::Czpsppre(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Czpspdok(args) => args.order(),
            Self::Czpsppre(args) => args.order(),
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
    fn cli_parses_czpspdok_search() {
        let cli = Cli::try_parse_from([
            "cdx-cz-psp",
            "search",
            "CZPSPDOK",
            "--query",
            "test",
            "--document-type",
            "Písemná interpelace",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czpspdok(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.document_type.as_deref(), Some("Písemná interpelace"));
            }
            _ => panic!("expected CZPSPDOK search command"),
        }
    }

    #[test]
    fn cli_accepts_lowercase_source_alias() {
        let cli =
            Cli::try_parse_from(["cdx-cz-psp", "search", "czpspdok", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czpspdok(args),
            } => assert_eq!(args.base.query.as_deref(), Some("test")),
            _ => panic!("expected CZPSPDOK search command"),
        }
    }

    #[test]
    fn cli_parses_czpsppre_search_with_type() {
        let cli = Cli::try_parse_from([
            "cdx-cz-psp",
            "search",
            "CZPSPPRE",
            "--query",
            "test",
            "--type",
            "Vládní návrh zákona",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czpsppre(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.doc_type.as_deref(), Some("Vládní návrh zákona"));
            }
            _ => panic!("expected CZPSPPRE search command"),
        }
    }

    #[test]
    fn cli_parses_get_command() {
        let cli =
            Cli::try_parse_from(["cdx-cz-psp", "get", "--dry-run", "cdx-cz-psp://doc/CZPSPDOK1/meta"])
                .unwrap();

        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "cdx-cz-psp://doc/CZPSPDOK1/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn cli_parses_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "meta"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::Meta { source: None },
            } => {}
            _ => panic!("expected schema meta command"),
        }
    }

    #[test]
    fn cli_parses_typed_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "meta", "CZPSPDOK"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Meta {
                        source: Some(SchemaSourceCommand::Czpspdok),
                    },
            } => {}
            _ => panic!("expected schema meta CZPSPDOK command"),
        }
    }

    #[test]
    fn cli_parses_schema_text_command() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "text"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::Text,
            } => {}
            _ => panic!("expected schema text command"),
        }
    }

    #[test]
    fn cli_parses_search_with_election_period() {
        let cli = Cli::try_parse_from([
            "cdx-cz-psp",
            "search",
            "CZPSPDOK",
            "--query",
            "test",
            "--election-period",
            "10",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czpspdok(args),
            } => {
                assert_eq!(args.election_period, Some(10));
            }
            _ => panic!("expected CZPSPDOK search command"),
        }
    }

    #[test]
    fn root_help_mentions_data_sources_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CZPSPDOK"));
        assert!(help.contains("CZPSPPRE"));
        assert!(help.contains("cdx-cz-psp search CZPSPDOK --query"));
        assert!(help.contains("cdx-cz-psp get cdx-cz-psp://doc/CZPSPDOK1234/meta"));
        assert!(help.contains("cdx-cz-psp schema [meta|text|toc|parts|related|related/counts|history]"));
    }

    #[test]
    fn search_help_uses_data_source_placeholder_and_heading() {
        let mut command = Cli::command();
        let search = command.find_subcommand_mut("search").unwrap();
        let help = search.render_long_help().to_string();

        assert!(help.contains("<DATA_SOURCE>"));
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CZPSPDOK"));
        assert!(help.contains("CZPSPPRE"));
    }

    #[test]
    fn cli_parses_schema_related_counts_command() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "related/counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
        }
    }

    #[test]
    fn cli_accepts_schema_related_counts_alias() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "related-counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
        }
    }

    #[test]
    fn cli_parses_schema_toc_with_source() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "toc", "CZPSPPRE"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Toc {
                        source: Some(SchemaSourceCommand::Czpsppre),
                    },
            } => {}
            _ => panic!("expected schema toc CZPSPPRE command"),
        }
    }

    #[test]
    fn cli_parses_schema_parts_with_source() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "parts", "CZPSPDOK"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Parts {
                        source: Some(SchemaSourceCommand::Czpspdok),
                    },
            } => {}
            _ => panic!("expected schema parts CZPSPDOK command"),
        }
    }

    #[test]
    fn cli_parses_schema_history_with_source() {
        let cli = Cli::try_parse_from(["cdx-cz-psp", "schema", "history", "CZPSPPRE"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::History {
                        source: Some(SchemaSourceCommand::Czpsppre),
                    },
            } => {}
            _ => panic!("expected schema history CZPSPPRE command"),
        }
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
        assert!(help.contains("toc"));
        assert!(help.contains("parts"));
        assert!(help.contains("related"));
        assert!(help.contains("related/counts"));
        assert!(help.contains("history"));
    }

    #[test]
    fn get_help_mentions_document_resources() {
        let mut command = Cli::command();
        let get = command.find_subcommand_mut("get").unwrap();
        let help = get.render_long_help().to_string();

        assert!(help.contains("cdx-cz-psp://doc/<DOC_ID>/toc"));
        assert!(help.contains("cdx-cz-psp://doc/<DOC_ID>/related/counts"));
        assert!(help.contains("cdx-cz-psp://doc/<DOC_ID>/history"));
    }
}

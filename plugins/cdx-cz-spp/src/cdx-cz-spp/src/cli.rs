use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::core::schema::{render_resource_schema, ResourceSchemaKind, SchemaSource};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::czsb::{SearchCzsbArgs, SEARCH_CZSB_HELP};

const ROOT_HELP: &str = "\
Data sources:
  CZSB     Czech Sbirkapp     Municipal regulations from Czech Sbirka pravnich predpisu

Examples:
  cdx-cz-spp search CZSB --query \"vyhláška\" --limit 5
  cdx-cz-spp search CZSB --publikujici \"Praha\" --platnost Platné
  cdx-cz-spp search CZSB --hlavni-typ pp --datum-vydani-from 2024-01-01
  cdx-cz-spp get cdx-cz-spp://doc/CZSB1234/meta
  cdx-cz-spp get cdx-cz-spp://doc/CZSB5678/text
  cdx-cz-spp get 'cdx-cz-spp://doc/CZSB1/related?type=IMPLEMENTING&limit=10'

Detailed source help:
  cdx-cz-spp search <DATA_SOURCE> --help
  cdx-cz-spp search CZSB --help

Endpoint schema help:
  cdx-cz-spp schema [meta|text|toc|parts|versions|related|related/counts]
  cdx-cz-spp schema meta CZSB

Common get resource suffixes:
  /meta, /toc, /text, /parts, /versions
  /related, /related/counts";

#[derive(Parser, Debug)]
#[command(
    name = "cdx-cz-spp",
    version,
    about = "CDX-CZ-SPP CLI for search, cdx-cz-spp:// resource fetches, and endpoint schemas",
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
        about = "Search Czech sbirkapp data source",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a cdx-cz-spp:// resource", arg_required_else_help = true)]
    Get(GetArgs),

    #[command(
        about = "Print cdx-cz-spp-oriented output schema and query parameters for get endpoints",
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
        about = "Output schema for /versions",
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Schema sources"
    )]
    Versions {
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
}

impl ResourceSchemaCommand {
    fn kind(&self) -> ResourceSchemaKind {
        match self {
            Self::Meta { .. } => ResourceSchemaKind::Meta,
            Self::Text => ResourceSchemaKind::Text,
            Self::Toc { .. } => ResourceSchemaKind::Toc,
            Self::Parts { .. } => ResourceSchemaKind::Parts,
            Self::Versions { .. } => ResourceSchemaKind::Versions,
            Self::Related { .. } => ResourceSchemaKind::Related,
            Self::RelatedCounts => ResourceSchemaKind::RelatedCounts,
        }
    }

    fn schema_source(&self) -> Option<SchemaSource> {
        match self {
            Self::Meta { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Toc { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Parts { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Versions { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Related { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Text | Self::RelatedCounts => None,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
enum SchemaSourceCommand {
    #[command(name = "CZSB", visible_alias = "czsb", about = "CZSB schema")]
    Czsb,
}

impl SchemaSourceCommand {
    fn kind(&self) -> SchemaSource {
        match self {
            Self::Czsb => SchemaSource::Czsb,
        }
    }
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "CZSB",
        visible_alias = "czsb",
        about = "Czech Sbirkapp: municipal regulations from Sbirka pravnich predpisu",
        after_help = SEARCH_CZSB_HELP,
        arg_required_else_help = true
    )]
    Czsb(SearchCzsbArgs),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Czsb(_) => "CZSB",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Czsb(args) => args.build_payload("CZSB"),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Czsb(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Czsb(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Czsb(args) => args.order(),
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
            "cdx-cz-spp",
            "search",
            "CZSB",
            "--query",
            "test",
            "--publikujici",
            "Praha",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czsb(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.publikujici.as_deref(), Some("Praha"));
            }
            _ => panic!("expected CZSB search command"),
        }
    }

    #[test]
    fn cli_accepts_lowercase_source_alias() {
        let cli =
            Cli::try_parse_from(["cdx-cz-spp", "search", "czsb", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czsb(args),
            } => assert_eq!(args.base.query.as_deref(), Some("test")),
            _ => panic!("expected CZSB search command"),
        }
    }

    #[test]
    fn cli_parses_czsb_search_with_hlavni_typ() {
        let cli = Cli::try_parse_from([
            "cdx-cz-spp",
            "search",
            "CZSB",
            "--query",
            "test",
            "--hlavni-typ",
            "pp",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czsb(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.hlavni_typ.as_deref(), Some("pp"));
            }
            _ => panic!("expected CZSB search command"),
        }
    }

    #[test]
    fn cli_parses_czsb_search_with_date_filters() {
        let cli = Cli::try_parse_from([
            "cdx-cz-spp",
            "search",
            "CZSB",
            "--query",
            "test",
            "--datum-vydani-from",
            "2024-01-01",
            "--datum-vydani-to",
            "2024-12-31",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Czsb(args),
            } => {
                assert_eq!(args.datum_vydani_from.as_deref(), Some("2024-01-01"));
                assert_eq!(args.datum_vydani_to.as_deref(), Some("2024-12-31"));
            }
            _ => panic!("expected CZSB search command"),
        }
    }

    #[test]
    fn cli_parses_get_command() {
        let cli =
            Cli::try_parse_from(["cdx-cz-spp", "get", "--dry-run", "cdx-cz-spp://doc/CZSB1/meta"])
                .unwrap();

        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "cdx-cz-spp://doc/CZSB1/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn cli_parses_schema_related_counts_command() {
        let cli = Cli::try_parse_from(["cdx-cz-spp", "schema", "related/counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
        }
    }

    #[test]
    fn cli_accepts_schema_related_counts_alias() {
        let cli = Cli::try_parse_from(["cdx-cz-spp", "schema", "related-counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
        }
    }

    #[test]
    fn cli_parses_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-cz-spp", "schema", "meta"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::Meta { source: None },
            } => {}
            _ => panic!("expected schema meta command"),
        }
    }

    #[test]
    fn cli_parses_typed_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-cz-spp", "schema", "meta", "CZSB"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Meta {
                        source: Some(SchemaSourceCommand::Czsb),
                    },
            } => {}
            _ => panic!("expected schema meta CZSB command"),
        }
    }

    #[test]
    fn cli_parses_schema_parts_with_source() {
        let cli = Cli::try_parse_from(["cdx-cz-spp", "schema", "parts", "CZSB"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Parts {
                        source: Some(SchemaSourceCommand::Czsb),
                    },
            } => {}
            _ => panic!("expected schema parts CZSB command"),
        }
    }

    #[test]
    fn cli_parses_schema_toc_with_source() {
        let cli = Cli::try_parse_from(["cdx-cz-spp", "schema", "toc", "CZSB"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Toc {
                        source: Some(SchemaSourceCommand::Czsb),
                    },
            } => {}
            _ => panic!("expected schema toc CZSB command"),
        }
    }

    #[test]
    fn cli_parses_search_with_sort_and_order() {
        let cli = Cli::try_parse_from([
            "cdx-cz-spp",
            "search",
            "CZSB",
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
                source: SearchSource::Czsb(args),
            } => {
                assert_eq!(args.base.sort.as_deref(), Some("date"));
                assert_eq!(args.base.order.as_deref(), Some("asc"));
            }
            _ => panic!("expected CZSB search command"),
        }
    }

    #[test]
    fn root_help_mentions_data_sources_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CZSB     Czech Sbirkapp"));
        assert!(help.contains("cdx-cz-spp search CZSB --query"));
        assert!(help.contains("cdx-cz-spp get cdx-cz-spp://doc/CZSB1234/meta"));
        assert!(help.contains("cdx-cz-spp search <DATA_SOURCE> --help"));
        assert!(help.contains("cdx-cz-spp schema [meta|text|toc|parts|versions|related|related/counts]"));
    }

    #[test]
    fn get_help_mentions_document_resources() {
        let mut command = Cli::command();
        let get = command.find_subcommand_mut("get").unwrap();
        let help = get.render_long_help().to_string();

        assert!(help.contains("cdx-cz-spp://doc/<DOC_ID>/versions"));
        assert!(help.contains("cdx-cz-spp://doc/<DOC_ID>/related/counts"));
        // No law routes for CZSB
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
        assert!(help.contains("toc"));
        assert!(help.contains("parts"));
        assert!(help.contains("versions"));
        assert!(help.contains("related"));
        assert!(help.contains("related/counts"));
    }

    #[test]
    fn schema_meta_help_lists_source_subcommands() {
        let mut command = Cli::command();
        let schema = command.find_subcommand_mut("schema").unwrap();
        let meta = schema.find_subcommand_mut("meta").unwrap();
        let help = meta.render_long_help().to_string();

        assert!(help.contains("[DATA_SOURCE]"));
        assert!(help.contains("Schema sources:"));
        assert!(help.contains("CZSB"));
    }

    #[test]
    fn search_help_uses_data_source_placeholder_and_heading() {
        let mut command = Cli::command();
        let search = command.find_subcommand_mut("search").unwrap();
        let help = search.render_long_help().to_string();

        assert!(help.contains("<DATA_SOURCE>"));
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CZSB"));
    }
}

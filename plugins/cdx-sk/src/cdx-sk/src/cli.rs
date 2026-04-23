use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::core::schema::{render_resource_schema, ResourceSchemaKind, SchemaSource};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::skez::{SearchSkezArgs, SEARCH_SKEZ_HELP};
use crate::sources::sknus::{SearchSknusArgs, SEARCH_SKNUS_HELP};
use crate::sources::skvs::{SearchSkvsArgs, SEARCH_SKVS_HELP};

const ROOT_HELP: &str = "\
Data sources:
  SKEZ     Slovak Legislation   e-Zbierka: Slovak laws, decrees, and regulations
  SKVS     Slovak Case Law      General courts: judicial decisions from Slovak courts
  SKNUS    Supreme & Const.     Supreme Court and Constitutional Court decisions

Examples:
  cdx-sk search SKEZ --query \"občiansky zákonník\" --limit 5
  cdx-sk search SKEZ --doc-number \"40/1964 Zb.\"
  cdx-sk search SKVS --query \"náhrada škody\" --court OSBA1 --limit 5
  cdx-sk search SKNUS --query \"ústavné právo\" --court NSSR
  cdx-sk get cdx-sk://doc/SKEZ1234/meta
  cdx-sk get cdx-sk://doc/SKVS5678/text
  cdx-sk get cdx-sk://law/SK/40/1964/meta
  cdx-sk get 'cdx-sk://doc/SKEZ1/related?type=IMPLEMENTING&limit=10'

Detailed source help:
  cdx-sk search <DATA_SOURCE> --help
  cdx-sk search [SKEZ|SKVS|SKNUS] --help

Endpoint schema help:
  cdx-sk schema [meta|text|toc|parts|versions|related|related/counts]
  cdx-sk schema meta [SKEZ|SKVS|SKNUS]

Common get resource suffixes:
  /meta, /toc, /text, /parts, /versions
  /related, /related/counts

Direct Slovak law fetches (SKEZ only):
  cdx-sk://law/SK/<NUM>/<YEAR>/meta
  cdx-sk://law/SK/<NUM>/<YEAR>/toc
  cdx-sk://law/SK/<NUM>/<YEAR>/text[?part=PART]
  cdx-sk://law/SK/<NUM>/<YEAR>/parts[?search=X&offset=N&limit=N]
  cdx-sk://law/SK/<NUM>/<YEAR>/versions
  cdx-sk://law/SK/<NUM>/<YEAR>/related[?type=TYPE&offset=N&limit=N&sort=FIELD&order=ORDER]
  cdx-sk://law/SK/<NUM>/<YEAR>/related/counts";

#[derive(Parser, Debug)]
#[command(
    name = "cdx-sk",
    version,
    about = "CDX-SK CLI for search, cdx-sk:// resource fetches, and endpoint schemas",
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
        about = "Search one Slovak data source",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a cdx-sk:// resource", arg_required_else_help = true)]
    Get(GetArgs),

    #[command(
        about = "Print cdx-sk-oriented output schema and query parameters for get endpoints",
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
    #[command(name = "SKEZ", visible_alias = "skez", about = "SKEZ schema")]
    Skez,

    #[command(name = "SKVS", visible_alias = "skvs", about = "SKVS schema")]
    Skvs,

    #[command(name = "SKNUS", visible_alias = "sknus", about = "SKNUS schema")]
    Sknus,
}

impl SchemaSourceCommand {
    fn kind(&self) -> SchemaSource {
        match self {
            Self::Skez => SchemaSource::Skez,
            Self::Skvs => SchemaSource::Skvs,
            Self::Sknus => SchemaSource::Sknus,
        }
    }
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "SKEZ",
        visible_alias = "skez",
        about = "Slovak Legislation: e-Zbierka laws, decrees, and regulations",
        after_help = SEARCH_SKEZ_HELP,
        arg_required_else_help = true
    )]
    Skez(SearchSkezArgs),

    #[command(
        name = "SKVS",
        visible_alias = "skvs",
        about = "Slovak Case Law: judicial decisions from general courts",
        after_help = SEARCH_SKVS_HELP,
        arg_required_else_help = true
    )]
    Skvs(SearchSkvsArgs),

    #[command(
        name = "SKNUS",
        visible_alias = "sknus",
        about = "Supreme & Constitutional Court: decisions from NSSR and USCSR",
        after_help = SEARCH_SKNUS_HELP,
        arg_required_else_help = true
    )]
    Sknus(SearchSknusArgs),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Skez(_) => "SKEZ",
            Self::Skvs(_) => "SKVS",
            Self::Sknus(_) => "SKNUS",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Skez(args) => args.build_payload("SKEZ"),
            Self::Skvs(args) => args.build_payload("SKVS"),
            Self::Sknus(args) => args.build_payload("SKNUS"),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Skez(args) => args.dry_run(),
            Self::Skvs(args) => args.dry_run(),
            Self::Sknus(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Skez(args) => args.sort(),
            Self::Skvs(args) => args.sort(),
            Self::Sknus(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Skez(args) => args.order(),
            Self::Skvs(args) => args.order(),
            Self::Sknus(args) => args.order(),
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
            "cdx-sk",
            "search",
            "SKEZ",
            "--query",
            "test",
            "--doc-number",
            "40/1964 Zb.",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skez(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.doc_number.as_deref(), Some("40/1964 Zb."));
            }
            _ => panic!("expected SKEZ search command"),
        }
    }

    #[test]
    fn cli_accepts_lowercase_source_alias() {
        let cli =
            Cli::try_parse_from(["cdx-sk", "search", "skez", "--query", "test"]).unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skez(args),
            } => assert_eq!(args.base.query.as_deref(), Some("test")),
            _ => panic!("expected SKEZ search command"),
        }
    }

    #[test]
    fn cli_parses_skvs_search_with_court() {
        let cli = Cli::try_parse_from([
            "cdx-sk",
            "search",
            "SKVS",
            "--query",
            "test",
            "--court",
            "OSBA1",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Skvs(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.court.as_deref(), Some("OSBA1"));
            }
            _ => panic!("expected SKVS search command"),
        }
    }

    #[test]
    fn cli_parses_sknus_search_with_decision_type() {
        let cli = Cli::try_parse_from([
            "cdx-sk",
            "search",
            "SKNUS",
            "--query",
            "test",
            "--decision-type",
            "Uznesenie",
        ])
        .unwrap();

        match cli.command {
            Commands::Search {
                source: SearchSource::Sknus(args),
            } => {
                assert_eq!(args.base.query.as_deref(), Some("test"));
                assert_eq!(args.decision_type.as_deref(), Some("Uznesenie"));
            }
            _ => panic!("expected SKNUS search command"),
        }
    }

    #[test]
    fn cli_parses_get_command() {
        let cli =
            Cli::try_parse_from(["cdx-sk", "get", "--dry-run", "cdx-sk://doc/SKEZ1/meta"])
                .unwrap();

        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "cdx-sk://doc/SKEZ1/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn cli_parses_schema_related_counts_command() {
        let cli = Cli::try_parse_from(["cdx-sk", "schema", "related/counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
        }
    }

    #[test]
    fn cli_accepts_schema_related_counts_alias() {
        let cli = Cli::try_parse_from(["cdx-sk", "schema", "related-counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
        }
    }

    #[test]
    fn cli_parses_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-sk", "schema", "meta"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::Meta { source: None },
            } => {}
            _ => panic!("expected schema meta command"),
        }
    }

    #[test]
    fn cli_parses_typed_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-sk", "schema", "meta", "SKEZ"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Meta {
                        source: Some(SchemaSourceCommand::Skez),
                    },
            } => {}
            _ => panic!("expected schema meta SKEZ command"),
        }
    }

    #[test]
    fn cli_parses_schema_parts_with_source() {
        let cli = Cli::try_parse_from(["cdx-sk", "schema", "parts", "SKVS"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Parts {
                        source: Some(SchemaSourceCommand::Skvs),
                    },
            } => {}
            _ => panic!("expected schema parts SKVS command"),
        }
    }

    #[test]
    fn cli_parses_schema_toc_with_source() {
        let cli = Cli::try_parse_from(["cdx-sk", "schema", "toc", "SKEZ"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Toc {
                        source: Some(SchemaSourceCommand::Skez),
                    },
            } => {}
            _ => panic!("expected schema toc SKEZ command"),
        }
    }

    #[test]
    fn cli_parses_search_with_sort_and_order() {
        let cli = Cli::try_parse_from([
            "cdx-sk",
            "search",
            "SKEZ",
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
                source: SearchSource::Skez(args),
            } => {
                assert_eq!(args.base.sort.as_deref(), Some("date"));
                assert_eq!(args.base.order.as_deref(), Some("asc"));
            }
            _ => panic!("expected SKEZ search command"),
        }
    }

    #[test]
    fn root_help_mentions_data_sources_and_examples() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("SKEZ     Slovak Legislation"));
        assert!(help.contains("SKVS     Slovak Case Law"));
        assert!(help.contains("SKNUS    Supreme & Const."));
        assert!(help.contains("cdx-sk search SKEZ --query"));
        assert!(help.contains("cdx-sk get cdx-sk://doc/SKEZ1234/meta"));
        assert!(help.contains("cdx-sk search <DATA_SOURCE> --help"));
        assert!(help.contains("cdx-sk schema [meta|text|toc|parts|versions|related|related/counts]"));
        assert!(help.contains("cdx-sk://law/SK/<NUM>/<YEAR>/meta"));
    }

    #[test]
    fn get_help_mentions_document_and_law_resources() {
        let mut command = Cli::command();
        let get = command.find_subcommand_mut("get").unwrap();
        let help = get.render_long_help().to_string();

        assert!(help.contains("cdx-sk://doc/<DOC_ID>/versions"));
        assert!(help.contains("cdx-sk://law/SK/<NUM>/<YEAR>/related/counts"));
        assert!(help.contains("SKEZ only"));
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
        assert!(help.contains("SKEZ"));
        assert!(help.contains("SKVS"));
        assert!(help.contains("SKNUS"));
    }

    #[test]
    fn search_help_uses_data_source_placeholder_and_heading() {
        let mut command = Cli::command();
        let search = command.find_subcommand_mut("search").unwrap();
        let help = search.render_long_help().to_string();

        assert!(help.contains("<DATA_SOURCE>"));
        assert!(help.contains("Data sources:"));
        assert!(help.contains("SKEZ"));
        assert!(help.contains("SKVS"));
        assert!(help.contains("SKNUS"));
    }
}

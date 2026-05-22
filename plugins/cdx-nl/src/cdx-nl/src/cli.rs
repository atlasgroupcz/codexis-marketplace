use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request};
use crate::core::schema::{render_resource_schema, ResourceSchemaKind, SchemaSource};
use crate::get::GetArgs;
use crate::sources::common::SearchPayloadArgs;
use crate::sources::nlbwb::{SearchNlbwbArgs, SEARCH_NLBWB_HELP};
use crate::sources::nluit::{SearchNluitArgs, SEARCH_NLUIT_HELP};

const ROOT_HELP: &str = "\
Data sources:
  NLBWB    Dutch Legislation    Basiswettenbestand (BWB): Dutch national laws and treaties
  NLUIT    Dutch Case Law       Rechtspraak.nl: judicial decisions from Dutch courts

Examples:
  cdx-nl search NLBWB --query \"Burgerlijk Wetboek\" --limit 5
  cdx-nl search NLBWB --bwb-id BWBR0001827
  cdx-nl search NLUIT --query \"huurrecht\" --court HR --limit 5
  cdx-nl get cdx-nl://doc/NLBWB1234/meta
  cdx-nl get cdx-nl://doc/NLUIT5678/text
  cdx-nl get cdx-nl://law/NL/BWBR0001827
  cdx-nl get cdx-nl://ecli/ECLI:NL:HR:2024:1234
  cdx-nl get cdx-nl://afkorting/BW/text

Detailed source help:
  cdx-nl search <DATA_SOURCE> --help
  cdx-nl search [NLBWB|NLUIT] --help

Endpoint schema help:
  cdx-nl schema [meta|text|toc|parts|versions|at|cited-by-decisions|related|related/counts|citations|publication-resolve|bwbid]
  cdx-nl schema meta [NLBWB|NLUIT]

Common get resource suffixes:
  /meta, /toc, /text, /parts, /versions, /citations
  /at?date=YYYY-MM-DD (NLBWB only), /cited-by-decisions (NLBWB only)
  /related, /related/counts, /attachment/<FILE>

Direct law fetch (NLBWB only):
  cdx-nl://law/NL/<BWB_ID>[/meta|text|toc|parts|versions|at|cited-by-decisions|related|citations]

Resolvers:
  cdx-nl://afkorting/<ABBR>[/...]           # NLBWB
  cdx-nl://ecli/<ECLI>[/...]                # NLUIT
  cdx-nl://publication/<PUB_ID>/resolve     # NLBWB";

#[derive(Parser, Debug)]
#[command(
    name = "cdx-nl",
    version,
    about = "CDX-NL CLI for search, cdx-nl:// resource fetches, and endpoint schemas",
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
        about = "Search one Dutch data source",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a cdx-nl:// resource", arg_required_else_help = true)]
    Get {
        #[command(flatten)]
        args: GetArgs,
    },

    #[command(
        about = "Print cdx-nl-oriented output schema and query parameters for get endpoints",
        arg_required_else_help = true,
        subcommand_value_name = "ENDPOINT",
        subcommand_help_heading = "Schema endpoints"
    )]
    Schema {
        #[command(subcommand)]
        endpoint: ResourceSchemaCommand,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ResourceSchemaCommand {
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

    #[command(name = "versions", about = "Output schema for /versions (NLBWB only)")]
    Versions,

    #[command(name = "at", about = "Output schema for /at?date=YYYY-MM-DD (NLBWB only)")]
    At,

    #[command(
        name = "cited-by-decisions",
        about = "Output schema for /cited-by-decisions (NLBWB only)"
    )]
    CitedByDecisions,

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
        about = "Output schema for /related/counts",
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Schema sources"
    )]
    RelatedCounts {
        #[command(subcommand)]
        source: Option<SchemaSourceCommand>,
    },

    #[command(about = "Output schema for /citations (NLBWB only)")]
    Citations,

    #[command(
        name = "publication-resolve",
        about = "Output schema for /publication/<PUB_ID>/resolve (NLBWB only)"
    )]
    PublicationResolve,

    #[command(
        name = "bwbid",
        about = "Output schema for NLBWB resolver response (afkorting/publication)"
    )]
    BwbId,
}

impl ResourceSchemaCommand {
    pub(crate) fn kind(&self) -> ResourceSchemaKind {
        match self {
            Self::Meta { .. } => ResourceSchemaKind::Meta,
            Self::Text => ResourceSchemaKind::Text,
            Self::Toc { .. } => ResourceSchemaKind::Toc,
            Self::Parts { .. } => ResourceSchemaKind::Parts,
            Self::Versions => ResourceSchemaKind::Versions,
            Self::At => ResourceSchemaKind::At,
            Self::CitedByDecisions => ResourceSchemaKind::CitedByDecisions,
            Self::Related { .. } => ResourceSchemaKind::Related,
            Self::RelatedCounts { .. } => ResourceSchemaKind::RelatedCounts,
            Self::Citations => ResourceSchemaKind::Citations,
            Self::PublicationResolve => ResourceSchemaKind::PublicationResolve,
            Self::BwbId => ResourceSchemaKind::BwbId,
        }
    }

    pub(crate) fn schema_source(&self) -> Option<SchemaSource> {
        match self {
            Self::Meta { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Toc { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Parts { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Related { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::RelatedCounts { source } => source.as_ref().map(SchemaSourceCommand::kind),
            Self::Text
            | Self::Versions
            | Self::At
            | Self::CitedByDecisions
            | Self::Citations
            | Self::PublicationResolve
            | Self::BwbId => None,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SchemaSourceCommand {
    #[command(name = "NLBWB", alias = "nlbwb", about = "NLBWB schema")]
    Nlbwb,

    #[command(name = "NLUIT", alias = "nluit", about = "NLUIT schema")]
    Nluit,
}

impl SchemaSourceCommand {
    pub(crate) fn kind(&self) -> SchemaSource {
        match self {
            Self::Nlbwb => SchemaSource::Nlbwb,
            Self::Nluit => SchemaSource::Nluit,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum SearchSource {
    /// Search Dutch national legislation (Basiswettenbestand).
    #[command(name = "NLBWB", alias = "nlbwb", after_help = SEARCH_NLBWB_HELP)]
    Nlbwb(SearchNlbwbArgs),

    /// Search Dutch case law (Rechtspraak.nl).
    #[command(name = "NLUIT", alias = "nluit", after_help = SEARCH_NLUIT_HELP)]
    Nluit(SearchNluitArgs),
}

impl SearchSource {
    fn source_code(&self) -> &'static str {
        match self {
            Self::Nlbwb(_) => "NLBWB",
            Self::Nluit(_) => "NLUIT",
        }
    }

    fn build_payload(&self) -> Result<String, CliError> {
        match self {
            Self::Nlbwb(args) => args.build_payload("NLBWB"),
            Self::Nluit(args) => args.build_payload("NLUIT"),
        }
    }

    fn dry_run(&self) -> bool {
        match self {
            Self::Nlbwb(args) => args.dry_run(),
            Self::Nluit(args) => args.dry_run(),
        }
    }

    fn sort(&self) -> Option<&str> {
        match self {
            Self::Nlbwb(args) => args.sort(),
            Self::Nluit(args) => args.sort(),
        }
    }

    fn order(&self) -> Option<&str> {
        match self {
            Self::Nlbwb(args) => args.order(),
            Self::Nluit(args) => args.order(),
        }
    }
}

pub(crate) fn run(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Search { source } => execute_search(source),
        Commands::Get { args } => execute_get(args),
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
    fn parses_uppercase_nlbwb_search() {
        let cli = Cli::try_parse_from(["cdx-nl", "search", "NLBWB", "--query", "BW", "--bwb-id", "BWBR0001827"]).unwrap();
        match cli.command {
            Commands::Search { source: SearchSource::Nlbwb(args) } => {
                assert_eq!(args.base.query.as_deref(), Some("BW"));
                assert_eq!(args.bwb_id.as_deref(), Some("BWBR0001827"));
            }
            _ => panic!("expected NLBWB search"),
        }
    }

    #[test]
    fn parses_lowercase_alias_nluit_search() {
        let cli = Cli::try_parse_from(["cdx-nl", "search", "nluit", "--query", "huurrecht", "--court", "HR"]).unwrap();
        match cli.command {
            Commands::Search { source: SearchSource::Nluit(args) } => {
                assert_eq!(args.base.query.as_deref(), Some("huurrecht"));
                assert_eq!(args.court, vec!["HR".to_string()]);
            }
            _ => panic!("expected NLUIT search"),
        }
    }

    #[test]
    fn parses_get_with_dry_run() {
        let cli = Cli::try_parse_from(["cdx-nl", "get", "cdx-nl://doc/NLBWB1/meta", "--dry-run"]).unwrap();
        match cli.command {
            Commands::Get { args } => {
                assert_eq!(args.resource, "cdx-nl://doc/NLBWB1/meta");
                assert!(args.dry_run);
            }
            _ => panic!("expected Get"),
        }
    }

    #[test]
    fn parses_get_with_ecli_colons() {
        let cli = Cli::try_parse_from(["cdx-nl", "get", "cdx-nl://ecli/ECLI:NL:HR:2024:1234"]).unwrap();
        match cli.command {
            Commands::Get { args } => {
                assert_eq!(args.resource, "cdx-nl://ecli/ECLI:NL:HR:2024:1234");
            }
            _ => panic!("expected Get"),
        }
    }

    #[test]
    fn parses_schema_subcommands() {
        // Source-aware endpoints accept either uppercase or lowercase alias.
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "meta", "NLBWB"]).is_ok());
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "meta", "nlbwb"]).is_ok());
        // Source is optional — generic-source intro mode.
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "meta"]).is_ok());
        // Shared endpoints take no source.
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "text"]).is_ok());
        // NLBWB-only endpoints take no source.
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "citations"]).is_ok());
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "at"]).is_ok());
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "cited-by-decisions"]).is_ok());
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "publication-resolve"]).is_ok());
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "bwbid"]).is_ok());
        // related/counts is source-aware.
        assert!(Cli::try_parse_from(["cdx-nl", "schema", "related/counts", "NLUIT"]).is_ok());
    }

    #[test]
    fn root_help_mentions_nlbwb_and_nluit() {
        let mut cmd = Cli::command();
        let help = cmd.render_help().to_string();
        assert!(help.contains("NLBWB") || help.to_lowercase().contains("legislation"));
        assert!(help.contains("NLUIT") || help.to_lowercase().contains("case law"));
    }
}

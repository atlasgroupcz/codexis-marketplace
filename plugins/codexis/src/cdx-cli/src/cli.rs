use clap::{Parser, Subcommand};

use crate::core::config::Config;
use crate::core::error::CliError;
use crate::core::http::{execute_get_request, execute_search_request, SearchFacetMode};
use crate::core::schema::{
    render_resource_schema_with_source, MetaSchemaSource, ResourceSchemaKind,
};
use crate::get::GetArgs;
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

const ROOT_HELP: &str = "\
Data sources:
  CR       Czech Legislation    Czech laws, decrees, regulations, and municipal documents
  SK       Slovak Legislation   Slovak laws and regulations
  JD       Czech Case Law       Judicial decisions from Czech courts
  ES       EU Court Decisions   EU Court of Justice and ECHR rulings
  EU       EU Legislation       EU regulations, directives, and decisions
  LT       Legal Literature     Legal publications and articles
  VS       Contract Templates   Contract specimens and templates
  COMMENT  Legal Commentaries   LIBERIS legal commentaries on Czech legislation
  ALL      Global Search        Exploratory search across all sources; use only for orientation

Examples:
  cdx-cli search CR --query \"náhrada škody\"
  cdx-cli search CR --query \"občanský zákoník\" --current --limit 5
  cdx-cli search JD --query \"náhrada škody\" --court \"Nejvyšší soud\" --limit 5
  cdx-cli get cdx://doc/CR10_2025_01_01/meta
  cdx-cli get cdx://doc/CR10_2025_01_01/text
  cdx-cli get 'cdx://doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=3'
  cdx-cli get cdx://cz_law/89/2012/meta
  cdx-cli get cdx://cz_law/89/2012/text?part=paragraf1

Detailed source help:
  cdx-cli search <DATA_SOURCE> --help
  cdx-cli search [JD|CR|EU|ES|SK|LT|VS|COMMENT|ALL] --help

Endpoint schema help:
  cdx-cli schema [meta|toc|versions|text|related|related/counts]
  cdx-cli schema meta [CR|SK|JD|ES|EU|LT|VS|COMMENT]

Common get resource suffixes:
  /meta, /toc, /text
  /versions (supported for CR documents only)
  /related, /related/counts

Direct Czech law fetches:
  cdx://cz_law/<NUM>/<YEAR>/meta
  cdx://cz_law/<NUM>/<YEAR>/toc
  cdx://cz_law/<NUM>/<YEAR>/text[?part=PART]
  cdx://cz_law/<NUM>/<YEAR>/versions
  cdx://cz_law/<NUM>/<YEAR>/related[?type=TYPE&part=PART&offset=N&limit=N&sort=FIELD&order=ORDER]
  cdx://cz_law/<NUM>/<YEAR>/related/counts[?part=PART]";

#[derive(Parser, Debug)]
#[command(
    name = "cdx-cli",
    version,
    about = "CODEXIS CLI for search, cdx:// resource fetches, and endpoint schemas",
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
        about = "Search one CODEXIS data source",
        arg_required_else_help = true,
        subcommand_value_name = "DATA_SOURCE",
        subcommand_help_heading = "Data sources"
    )]
    Search {
        #[command(subcommand)]
        source: SearchSource,
    },

    #[command(about = "Fetch a cdx:// resource", arg_required_else_help = true)]
    Get(GetArgs),

    #[command(
        about = "Print cdx-cli-oriented output schema and query parameters for get endpoints",
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
        subcommand_help_heading = "Metadata sources"
    )]
    Meta {
        #[command(subcommand)]
        source: Option<MetaSchemaSourceCommand>,
    },

    #[command(about = "Output schema for /text")]
    Text,

    #[command(about = "Output schema for /toc")]
    Toc,

    #[command(about = "Output schema for /versions")]
    Versions,

    #[command(about = "Output schema for /related")]
    Related,

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
            Self::Toc => ResourceSchemaKind::Toc,
            Self::Versions => ResourceSchemaKind::Versions,
            Self::Related => ResourceSchemaKind::Related,
            Self::RelatedCounts => ResourceSchemaKind::RelatedCounts,
        }
    }

    fn meta_source(&self) -> Option<MetaSchemaSource> {
        match self {
            Self::Meta { source } => source.as_ref().map(MetaSchemaSourceCommand::kind),
            _ => None,
        }
    }
}

#[derive(Subcommand, Debug)]
enum MetaSchemaSourceCommand {
    #[command(
        name = "COMMENT",
        visible_alias = "comment",
        about = "COMMENT metadata schema"
    )]
    Comment,

    #[command(name = "CR", visible_alias = "cr", about = "CR metadata schema")]
    Cr,

    #[command(name = "ES", visible_alias = "es", about = "ES metadata schema")]
    Es,

    #[command(name = "EU", visible_alias = "eu", about = "EU metadata schema")]
    Eu,

    #[command(name = "JD", visible_alias = "jd", about = "JD metadata schema")]
    Jd,

    #[command(name = "LT", visible_alias = "lt", about = "LT metadata schema")]
    Lt,

    #[command(name = "SK", visible_alias = "sk", about = "SK metadata schema")]
    Sk,

    #[command(name = "VS", visible_alias = "vs", about = "VS metadata schema")]
    Vs,
}

impl MetaSchemaSourceCommand {
    fn kind(&self) -> MetaSchemaSource {
        match self {
            Self::Comment => MetaSchemaSource::Comment,
            Self::Cr => MetaSchemaSource::Cr,
            Self::Es => MetaSchemaSource::Es,
            Self::Eu => MetaSchemaSource::Eu,
            Self::Jd => MetaSchemaSource::Jd,
            Self::Lt => MetaSchemaSource::Lt,
            Self::Sk => MetaSchemaSource::Sk,
            Self::Vs => MetaSchemaSource::Vs,
        }
    }
}

#[derive(Subcommand, Debug)]
enum SearchSource {
    #[command(
        name = "ALL",
        visible_alias = "all",
        about = "Global Search: exploratory search across all sources; use only for orientation",
        after_help = SEARCH_ALL_HELP,
        arg_required_else_help = true
    )]
    All(SearchAllArgs),

    #[command(
        name = "COMMENT",
        visible_alias = "comment",
        about = "Legal Commentaries: LIBERIS legal commentaries on Czech legislation",
        after_help = SEARCH_COMMENT_HELP,
        arg_required_else_help = true
    )]
    Comment(SearchCommentArgs),

    #[command(
        name = "CR",
        visible_alias = "cr",
        about = "Czech Legislation: Czech laws, decrees, regulations, and municipal documents",
        after_help = SEARCH_CR_HELP,
        arg_required_else_help = true
    )]
    Cr(SearchCrArgs),

    #[command(
        name = "ES",
        visible_alias = "es",
        about = "EU Court Decisions: EU Court of Justice and ECHR rulings",
        after_help = SEARCH_ES_HELP,
        arg_required_else_help = true
    )]
    Es(SearchEsArgs),

    #[command(
        name = "EU",
        visible_alias = "eu",
        about = "EU Legislation: EU regulations, directives, and decisions",
        after_help = SEARCH_EU_HELP,
        arg_required_else_help = true
    )]
    Eu(SearchEuArgs),

    #[command(
        name = "JD",
        visible_alias = "jd",
        about = "Czech Case Law: judicial decisions from Czech courts",
        after_help = SEARCH_JD_HELP,
        arg_required_else_help = true
    )]
    Jd(SearchJdArgs),

    #[command(
        name = "LT",
        visible_alias = "lt",
        about = "Legal Literature: legal publications and articles",
        after_help = SEARCH_LT_HELP,
        arg_required_else_help = true
    )]
    Lt(SearchLtArgs),

    #[command(
        name = "SK",
        visible_alias = "sk",
        about = "Slovak Legislation: Slovak laws and regulations",
        after_help = SEARCH_SK_HELP,
        arg_required_else_help = true
    )]
    Sk(SearchSkArgs),

    #[command(
        name = "VS",
        visible_alias = "vs",
        about = "Contract Templates: contract specimens and templates",
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
        &config.auth_header,
        source.source_code(),
        &payload,
        source.dry_run(),
        source.facet_mode(),
    )
}

fn execute_get(args: GetArgs) -> Result<(), CliError> {
    let config = Config::load()?;
    execute_get_request(
        &config.base_url,
        &config.auth_header,
        &args.resource,
        args.dry_run,
    )
}

fn execute_schema(endpoint: ResourceSchemaCommand) -> Result<(), CliError> {
    println!(
        "{}",
        render_resource_schema_with_source(endpoint.kind(), endpoint.meta_source())?
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
    fn cli_parses_get_command() {
        let cli =
            Cli::try_parse_from(["cdx-cli", "get", "--dry-run", "cdx://doc/JD1/meta"]).unwrap();

        match cli.command {
            Commands::Get(args) => {
                assert!(args.dry_run);
                assert_eq!(args.resource, "cdx://doc/JD1/meta");
            }
            _ => panic!("expected get command"),
        }
    }

    #[test]
    fn cli_parses_schema_related_counts_command() {
        let cli = Cli::try_parse_from(["cdx-cli", "schema", "related/counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
        }
    }

    #[test]
    fn cli_parses_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-cli", "schema", "meta"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::Meta { source: None },
            } => {}
            _ => panic!("expected schema meta command"),
        }
    }

    #[test]
    fn cli_parses_typed_schema_meta_command() {
        let cli = Cli::try_parse_from(["cdx-cli", "schema", "meta", "JD"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint:
                    ResourceSchemaCommand::Meta {
                        source: Some(MetaSchemaSourceCommand::Jd),
                    },
            } => {}
            _ => panic!("expected schema meta JD command"),
        }
    }

    #[test]
    fn cli_accepts_schema_related_counts_alias() {
        let cli = Cli::try_parse_from(["cdx-cli", "schema", "related-counts"]).unwrap();

        match cli.command {
            Commands::Schema {
                endpoint: ResourceSchemaCommand::RelatedCounts,
            } => {}
            _ => panic!("expected schema related/counts command"),
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
    fn cli_rejects_removed_search_schema_flags() {
        for flag in ["--schema-input", "--schema-output"] {
            let error = Cli::try_parse_from(["cdx-cli", "search", "JD", flag]).unwrap_err();
            let rendered = error.to_string();

            assert!(rendered.contains(flag));
            assert!(rendered.contains("unexpected"));
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

    #[test]
    fn cli_rejects_removed_cz_law_command() {
        let error = Cli::try_parse_from(["cdx-cli", "cz_law", "meta", "89/2012"]).unwrap_err();
        let rendered = error.to_string();
        assert!(rendered.contains("unrecognized subcommand"));
        assert!(rendered.contains("cz_law"));
    }

    #[test]
    fn root_help_mentions_direct_cz_law_fetches() {
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CR       Czech Legislation"));
        assert!(help.contains("ALL      Global Search"));
        assert!(help.contains("cdx-cli search CR --query \"náhrada škody\""));
        assert!(help.contains(
            "cdx-cli search JD --query \"náhrada škody\" --court \"Nejvyšší soud\" --limit 5"
        ));
        assert!(help.contains("cdx-cli search <DATA_SOURCE> --help"));
        assert!(help.contains("cdx-cli search [JD|CR|EU|ES|SK|LT|VS|COMMENT|ALL] --help"));
        assert!(help.contains("cdx-cli schema [meta|toc|versions|text|related|related/counts]"));
        assert!(help.contains("cdx://cz_law/<NUM>/<YEAR>/meta"));
        assert!(help.contains("/versions (supported for CR documents only)"));
        assert!(!help.contains("cz_law  Fetch Czech law resources by number/year"));
    }

    #[test]
    fn get_help_mentions_document_and_cz_law_resources() {
        let mut command = Cli::command();
        let get = command.find_subcommand_mut("get").unwrap();
        let help = get.render_long_help().to_string();

        assert!(help.contains("cdx://doc/<DOC_ID>/versions"));
        assert!(help.contains("CR documents only"));
        assert!(help.contains("cdx://cz_law/<NUM>/<YEAR>/related/counts"));
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
        assert!(help.contains("versions"));
        assert!(help.contains("related"));
        assert!(help.contains("related/counts"));
    }

    #[test]
    fn schema_meta_help_lists_metadata_source_subcommands() {
        let mut command = Cli::command();
        let schema = command.find_subcommand_mut("schema").unwrap();
        let meta = schema.find_subcommand_mut("meta").unwrap();
        let help = meta.render_long_help().to_string();

        assert!(help.contains("[DATA_SOURCE]"));
        assert!(help.contains("Metadata sources:"));
        assert!(help.contains("CR"));
        assert!(help.contains("JD"));
        assert!(help.contains("COMMENT"));
    }

    #[test]
    fn search_help_uses_data_source_placeholder_and_heading() {
        let mut command = Cli::command();
        let search = command.find_subcommand_mut("search").unwrap();
        let help = search.render_long_help().to_string();

        assert!(help.contains("<DATA_SOURCE>"));
        assert!(help.contains("Data sources:"));
        assert!(help.contains("CR       Czech Legislation:"));
        assert!(help.contains("COMMENT  Legal Commentaries:"));
    }
}

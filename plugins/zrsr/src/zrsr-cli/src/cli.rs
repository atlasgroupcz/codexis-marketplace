use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::Value;

use crate::error::normalize_ico;
use crate::http::ZrsrClient;
use crate::sources;

const ROOT_HELP: &str = "\
zrsr-cli — Slovak trade & business register (ZRSR / RPO) lookup.

Wraps the public RPO API of the Slovak Statistical Office (Register právnických
osôb a podnikateľov), which aggregates the source registers — the Živnostenský
register SR (zrsr.sk) and the Obchodný register among them.

Output: JSON on stdout (consume with `jq`); on failure, an `ERROR:` line on stderr
and a non-zero exit code. Pass-through of the RPO response body — only --limit
rewrites the `results` array.

Examples:
  zrsr-cli detail 686930
  zrsr-cli detail --id 4445617 --history
  zrsr-cli search --name \"Tatra banka\" --limit 5
  zrsr-cli search --name \"Kováč\" --obec \"Žilina\" --register zr --only-active
  zrsr-cli api get \"/entity/4445617\"

The RPO API caps every search at 500 records and an identifier lookup can take
tens of seconds. Override base URL with ZRSR_BASE_URL
(default: https://api.statistics.sk/rpo/v1).
";

#[derive(Parser, Debug)]
#[command(
    name = "zrsr-cli",
    version,
    about = "Slovak trade & business register (ZRSR / RPO) CLI",
    long_about = ROOT_HELP,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Fetch full detail of a single subject by IČO (or by RPO id via --id).
    Detail(DetailArgs),
    /// Search subjects by name / IČO / municipality / street / legal form.
    Search(SearchArgs),
    /// Raw HTTP access (escape hatch).
    #[command(subcommand)]
    Api(ApiCommand),
}

#[derive(Args, Debug)]
pub struct DetailArgs {
    /// IČO (1–8 digits; padded to 8 with leading zeros for the API call).
    #[arg(required_unless_present = "id", conflicts_with = "id")]
    pub ico: Option<String>,

    /// Internal RPO entity id (skips the slow IČO search hop).
    #[arg(long)]
    pub id: Option<u64>,

    /// Request historical records (showHistoricalData=true).
    #[arg(long)]
    pub history: bool,

    /// Request organization units (showOrganizationUnits=true).
    #[arg(long)]
    pub units: bool,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Business / person name (substring; matches historical names too).
    #[arg(long)]
    pub name: Option<String>,

    /// IČO (server-side exact match; padded to 8 digits).
    #[arg(long)]
    pub ico: Option<String>,

    /// Obec (municipality of the registered address).
    #[arg(long, visible_alias = "municipality")]
    pub obec: Option<String>,

    /// Street of the registered address.
    #[arg(long)]
    pub street: Option<String>,

    /// Právna forma text (e.g. "Spoločnosť s ručením obmedzeným").
    #[arg(long = "pravna-forma", visible_alias = "legal-form")]
    pub pravna_forma: Option<String>,

    /// Source register (server-side sourceRegister filter).
    #[arg(long, value_enum, default_value_t = RegisterFilter::All)]
    pub register: RegisterFilter,

    /// Only subjects without a termination date (server-side onlyActive).
    #[arg(long)]
    pub only_active: bool,

    /// Truncate `results` to N records (client-side; the API itself caps at 500).
    #[arg(long)]
    pub limit: Option<usize>,
}

#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    /// Raw GET against the RPO base URL.
    Get { path: String },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum RegisterFilter {
    /// Živnostenský register.
    Zr,
    /// Obchodný register.
    Or,
    /// No source-register filtering.
    All,
}

impl RegisterFilter {
    fn source_register(self) -> Option<&'static str> {
        match self {
            RegisterFilter::Zr => Some("Živnostenský register"),
            RegisterFilter::Or => Some("Obchodný register"),
            RegisterFilter::All => None,
        }
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = ZrsrClient::new()?;

    let (body, source_entries) = match cli.command {
        Command::Detail(args) => {
            let body = detail(&client, args)?;
            let entries = sources::detail_source(&body);
            (body, entries)
        }
        Command::Search(args) => {
            let body = search(&client, args)?;
            let entries = sources::search_sources(&body, sources::SEARCH_SOURCES_CAP);
            (body, entries)
        }
        Command::Api(ApiCommand::Get { path }) => {
            let body = client.get(&path)?;
            let entries = sources::sniff_sources(&body);
            (body, entries)
        }
    };

    print!("{body}");
    if !body.ends_with('\n') {
        println!();
    }
    sources::try_publish(&source_entries);
    Ok(())
}

fn detail(client: &ZrsrClient, args: DetailArgs) -> Result<String> {
    let id = match (args.id, args.ico) {
        (Some(id), _) => id,
        (None, Some(ico)) => {
            let ico = normalize_ico(&ico)?;
            let body = client.get_with_query("/search", &[("identifier", ico.clone())])?;
            let parsed: Value = serde_json::from_str(&body)
                .map_err(|e| anyhow!("RPO search response is not valid JSON: {e}"))?;
            parsed
                .pointer("/results/0/id")
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow!("no RPO entity found for IČO {ico}"))?
        }
        (None, None) => unreachable!("clap requires either ICO or --id"),
    };
    let mut query: Vec<(&str, String)> = Vec::new();
    if args.history {
        query.push(("showHistoricalData", "true".to_string()));
    }
    if args.units {
        query.push(("showOrganizationUnits", "true".to_string()));
    }
    client.get_with_query(&format!("/entity/{id}"), &query)
}

fn search(client: &ZrsrClient, args: SearchArgs) -> Result<String> {
    if args.limit == Some(0) {
        return Err(anyhow!("--limit must be > 0"));
    }
    let query = build_search_query(&args)?;
    let body = client.get_with_query("/search", &query)?;
    match args.limit {
        Some(limit) => truncate_results(&body, limit),
        None => Ok(body),
    }
}

fn build_search_query(args: &SearchArgs) -> Result<Vec<(&'static str, String)>> {
    let mut query: Vec<(&'static str, String)> = Vec::new();
    if let Some(name) = trimmed(&args.name) {
        query.push(("fullName", name));
    }
    if let Some(ico) = trimmed(&args.ico) {
        query.push(("identifier", normalize_ico(&ico)?));
    }
    if let Some(obec) = trimmed(&args.obec) {
        query.push(("addressMunicipality", obec));
    }
    if let Some(street) = trimmed(&args.street) {
        query.push(("addressStreet", street));
    }
    if let Some(pf) = trimmed(&args.pravna_forma) {
        query.push(("legalForm", pf));
    }
    if query.is_empty() {
        return Err(anyhow!(
            "search needs at least one of --name, --ico, --obec, --street or --pravna-forma"
        ));
    }
    if let Some(register) = args.register.source_register() {
        query.push(("sourceRegister", register.to_string()));
    }
    if args.only_active {
        query.push(("onlyActive", "true".to_string()));
    }
    Ok(query)
}

fn trimmed(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn truncate_results(body: &str, limit: usize) -> Result<String> {
    let mut parsed: Value = serde_json::from_str(body)
        .map_err(|e| anyhow!("RPO search response is not valid JSON: {e}"))?;
    if let Some(results) = parsed.get_mut("results").and_then(Value::as_array_mut) {
        results.truncate(limit);
    }
    serde_json::to_string(&parsed).map_err(|e| anyhow!("could not serialize truncated results: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_args() -> SearchArgs {
        SearchArgs {
            name: None,
            ico: None,
            obec: None,
            street: None,
            pravna_forma: None,
            register: RegisterFilter::All,
            only_active: false,
            limit: None,
        }
    }

    #[test]
    fn search_requires_a_value_filter() {
        assert!(build_search_query(&empty_args()).is_err());
        let mut args = empty_args();
        args.register = RegisterFilter::Zr;
        args.only_active = true;
        assert!(build_search_query(&args).is_err());
    }

    #[test]
    fn search_params_map_to_rpo_names() {
        let mut args = empty_args();
        args.ico = Some("123".to_string());
        args.obec = Some(" Žilina ".to_string());
        args.register = RegisterFilter::Zr;
        args.only_active = true;

        let query = build_search_query(&args).unwrap();

        assert_eq!(
            query,
            vec![
                ("identifier", "00000123".to_string()),
                ("addressMunicipality", "Žilina".to_string()),
                ("sourceRegister", "Živnostenský register".to_string()),
                ("onlyActive", "true".to_string()),
            ]
        );
    }

    #[test]
    fn limit_truncates_results() {
        let body = r#"{"results":[{"id":1},{"id":2},{"id":3}],"license":"x"}"#;
        let truncated = truncate_results(body, 2).unwrap();
        let parsed: Value = serde_json::from_str(&truncated).unwrap();
        assert_eq!(parsed["results"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["license"], "x");
    }

    #[test]
    fn limit_passes_through_bodies_without_results() {
        assert_eq!(
            truncate_results(r#"{"message":"x"}"#, 1).unwrap(),
            r#"{"message":"x"}"#
        );
    }
}

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};

use serde_json::Value;

use crate::http::RegistryClient;
use crate::ico::{normalize_ico, validate_date};
use crate::orsr::resolve_links;
use crate::sources;

const ROOT_HELP: &str = "\
orsr-cli - Slovak commercial register / RPO lookup.

Output: JSON on stdout (consume with `jq`); on failure, an `ERROR:` line on stderr
and a non-zero exit code. RPO responses are passed through verbatim, never reformatted.
Successful RPO/RUZ responses are cached by URL for 24 hours; use --no-cache for a
fresh read (TTL override: ORSR_CLI_CACHE_TTL_SECONDS).

Examples:
  orsr-cli search --ico 31333532
  orsr-cli search --name \"ESET\" --municipality Bratislava --active
  orsr-cli search --legal-form \"Akciová spoločnosť\" --establishment-after 2020-01-01
  orsr-cli detail 937053
  orsr-cli detail 937053 --history --units
  orsr-cli links 31333532
  orsr-cli ruz 31333532
  orsr-cli api \"search?identifier=31333532\"

Data source: RPO open API of the Štatistický úrad SR (refreshed nightly, CC-BY 4.0).
Override base URLs with RPO_BASE_URL (default: https://api.statistics.sk/rpo/v1)
and RUZ_BASE_URL (default: https://www.registeruz.sk/cruz-public/api).
";

#[derive(Parser, Debug)]
#[command(
    name = "orsr-cli",
    version,
    about = "Slovak commercial register (RPO / ORSR) CLI",
    long_about = ROOT_HELP,
    arg_required_else_help = true
)]
pub struct Cli {
    /// Bypass the 24h URL cache.
    #[arg(long, global = true)]
    pub no_cache: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search legal entities in RPO by IČO, name or other filters.
    Search(Box<SearchArgs>),
    /// Fetch the full RPO record by RPO entity id (from search results, not IČO).
    Detail(DetailArgs),
    /// Resolve orsr.sk extract URLs (aktuálny + úplný výpis) for an IČO.
    Links { ico: String },
    /// Fetch the accounting-entity record (registeruz.sk) for an IČO.
    Ruz { ico: String },
    /// Raw GET against the RPO base URL (escape hatch).
    Api { path: String },
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// IČO (1-8 digits; padded to 8 with leading zeros for the API call).
    #[arg(long, visible_alias = "identifier")]
    pub ico: Option<String>,

    /// Name fragment (matches historical names too; filter results yourself).
    #[arg(long, visible_alias = "full-name")]
    pub name: Option<String>,

    /// Seat municipality.
    #[arg(long, visible_alias = "address-municipality")]
    pub municipality: Option<String>,

    /// Seat street.
    #[arg(long, visible_alias = "address-street")]
    pub street: Option<String>,

    /// Legal form text.
    #[arg(long = "legal-form")]
    pub legal_form: Option<String>,

    /// Legal status text.
    #[arg(long = "legal-status")]
    pub legal_status: Option<String>,

    /// Source register text.
    #[arg(long = "source-register")]
    pub source_register: Option<String>,

    /// Main economic activity text.
    #[arg(long = "main-activity")]
    pub main_activity: Option<String>,

    /// Only active entities.
    #[arg(long)]
    pub active: bool,

    /// Established on or after YYYY-MM-DD.
    #[arg(long = "establishment-after")]
    pub establishment_after: Option<String>,

    /// Established on or before YYYY-MM-DD.
    #[arg(long = "establishment-before")]
    pub establishment_before: Option<String>,

    /// Terminated on or after YYYY-MM-DD.
    #[arg(long = "termination-after")]
    pub termination_after: Option<String>,

    /// Terminated on or before YYYY-MM-DD.
    #[arg(long = "termination-before")]
    pub termination_before: Option<String>,

    /// Database modification on or after YYYY-MM-DD.
    #[arg(long = "db-modification-after")]
    pub db_modification_after: Option<String>,

    /// Database modification on or before YYYY-MM-DD.
    #[arg(long = "db-modification-before")]
    pub db_modification_before: Option<String>,
}

#[derive(Args, Debug)]
pub struct DetailArgs {
    /// RPO entity id (`id` from search results, not IČO).
    pub id: u64,

    /// Request historical records (showHistoricalData=true).
    #[arg(long)]
    pub history: bool,

    /// Request organization units (showOrganizationUnits=true).
    #[arg(long)]
    pub units: bool,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = RegistryClient::new(!cli.no_cache)?;

    let (body, source_entries) = match cli.command {
        Command::Search(args) => (search(&client, &args)?, Vec::new()),
        Command::Detail(args) => {
            let body = detail(&client, args)?;
            let entries = detail_source_entries(&client, &body);
            (body, entries)
        }
        Command::Links { ico } => {
            let ico = normalize_ico(&ico)?;
            let links = resolve_links(&client, &ico)?;
            let entries = sources::links_sources(&links, &ico);
            (serde_json::to_string_pretty(&links)?, entries)
        }
        Command::Ruz { ico } => {
            let ico = normalize_ico(&ico)?;
            let (body, entity_id) = ruz(&client, &ico)?;
            let entries = sources::ruz_source(&body, &ico, entity_id);
            (body, entries)
        }
        Command::Api { path } => (client.rpo_get(&path, &[])?, Vec::new()),
    };

    print!("{body}");
    if !body.ends_with('\n') {
        println!();
    }
    sources::try_publish(&source_entries);
    Ok(())
}

fn detail_source_entries(client: &RegistryClient, detail_body: &str) -> Vec<Value> {
    let Some(ico) = sources::current_ico(detail_body) else {
        return Vec::new();
    };
    let Ok(links) = resolve_links(client, &ico) else {
        return Vec::new();
    };
    sources::detail_source(detail_body, &links, &ico)
}

fn search(client: &RegistryClient, args: &SearchArgs) -> Result<String> {
    validate_search_dates(args)?;
    let params = build_search_params(args)?;
    let query: Vec<(&str, &str)> = params
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    client.rpo_get("search", &query)
}

fn detail(client: &RegistryClient, args: DetailArgs) -> Result<String> {
    let mut query: Vec<(&str, &str)> = Vec::new();
    if args.history {
        query.push(("showHistoricalData", "true"));
    }
    if args.units {
        query.push(("showOrganizationUnits", "true"));
    }
    client.rpo_get(&format!("entity/{}", args.id), &query)
}

fn build_search_params(args: &SearchArgs) -> Result<Vec<(String, String)>> {
    let mut params = Vec::new();
    push_param(
        &mut params,
        "identifier",
        normalized_ico_option(args.ico.as_deref())?,
    );
    push_param(&mut params, "fullName", args.name.as_deref());
    push_param(
        &mut params,
        "addressMunicipality",
        args.municipality.as_deref(),
    );
    push_param(&mut params, "addressStreet", args.street.as_deref());
    push_param(&mut params, "legalForm", args.legal_form.as_deref());
    push_param(&mut params, "legalStatus", args.legal_status.as_deref());
    push_param(
        &mut params,
        "sourceRegister",
        args.source_register.as_deref(),
    );
    push_param(&mut params, "mainActivity", args.main_activity.as_deref());
    push_param(
        &mut params,
        "establishmentAfter",
        args.establishment_after.as_deref(),
    );
    push_param(
        &mut params,
        "establishmentBefore",
        args.establishment_before.as_deref(),
    );
    push_param(
        &mut params,
        "terminationAfter",
        args.termination_after.as_deref(),
    );
    push_param(
        &mut params,
        "terminationBefore",
        args.termination_before.as_deref(),
    );
    push_param(
        &mut params,
        "dbModificationDateAfter",
        args.db_modification_after.as_deref(),
    );
    push_param(
        &mut params,
        "dbModificationDateBefore",
        args.db_modification_before.as_deref(),
    );
    if args.active {
        params.push(("onlyActive".to_string(), "true".to_string()));
    }
    if params.is_empty() {
        return Err(anyhow!(
            "search needs at least one filter - use --ico, --name, --municipality, --street, --legal-form, --legal-status, --source-register, --main-activity, --active or a date filter"
        ));
    }
    Ok(params)
}

fn validate_search_dates(args: &SearchArgs) -> Result<()> {
    for (name, value) in [
        ("--establishment-after", args.establishment_after.as_deref()),
        (
            "--establishment-before",
            args.establishment_before.as_deref(),
        ),
        ("--termination-after", args.termination_after.as_deref()),
        ("--termination-before", args.termination_before.as_deref()),
        (
            "--db-modification-after",
            args.db_modification_after.as_deref(),
        ),
        (
            "--db-modification-before",
            args.db_modification_before.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            validate_date(value, name)?;
        }
    }
    Ok(())
}

fn normalized_ico_option(value: Option<&str>) -> Result<Option<String>> {
    value.map(normalize_ico).transpose()
}

fn push_param(params: &mut Vec<(String, String)>, name: &str, value: Option<impl AsRef<str>>) {
    if let Some(value) = value {
        let trimmed = value.as_ref().trim();
        if !trimmed.is_empty() {
            params.push((name.to_string(), trimmed.to_string()));
        }
    }
}

fn ruz(client: &RegistryClient, ico: &str) -> Result<(String, i64)> {
    let ids_body = client.ruz_get(
        "uctovne-jednotky",
        &[
            ("zmenene-od", "2000-01-01"),
            ("ico", ico),
            ("max-zaznamov", "10"),
        ],
    )?;
    let ids: serde_json::Value = serde_json::from_str(&ids_body)?;
    let first_id = ids["id"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow!("no accounting entity found for IČO {ico} in registeruz.sk"))?;
    let body = client.ruz_get("uctovna-jednotka", &[("id", first_id.to_string().as_str())])?;
    Ok((body, first_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_args() -> SearchArgs {
        SearchArgs {
            ico: None,
            name: None,
            municipality: None,
            street: None,
            legal_form: None,
            legal_status: None,
            source_register: None,
            main_activity: None,
            active: false,
            establishment_after: None,
            establishment_before: None,
            termination_after: None,
            termination_before: None,
            db_modification_after: None,
            db_modification_before: None,
        }
    }

    #[test]
    fn search_requires_a_filter() {
        assert!(build_search_params(&empty_args()).is_err());
    }

    #[test]
    fn search_params_map_to_rpo_names() {
        let mut args = empty_args();
        args.ico = Some("123".to_string());
        args.municipality = Some("Bratislava".to_string());
        args.street = Some("Einsteinova".to_string());
        args.active = true;
        args.establishment_after = Some("2020-01-01".to_string());

        let params = build_search_params(&args).unwrap();

        assert_eq!(
            params,
            vec![
                ("identifier".to_string(), "00000123".to_string()),
                ("addressMunicipality".to_string(), "Bratislava".to_string()),
                ("addressStreet".to_string(), "Einsteinova".to_string()),
                ("establishmentAfter".to_string(), "2020-01-01".to_string()),
                ("onlyActive".to_string(), "true".to_string()),
            ]
        );
    }

    #[test]
    fn search_rejects_malformed_dates() {
        let mut args = empty_args();
        args.termination_before = Some("10.06.2026".to_string());
        assert!(validate_search_dates(&args).is_err());
    }
}

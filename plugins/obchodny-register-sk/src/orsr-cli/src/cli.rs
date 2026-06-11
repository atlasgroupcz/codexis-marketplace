use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};

use serde_json::Value;

use crate::http::RegistryClient;
use crate::ico::normalize_ico;
use crate::orsr::resolve_links;
use crate::sources;

const ROOT_HELP: &str = "\
orsr-cli — Slovak commercial register / RPO lookup.

Output: JSON on stdout (consume with `jq`); on failure, an `ERROR:` line on stderr
and a non-zero exit code. RPO responses are passed through verbatim, never reformatted.

Examples:
  orsr-cli search --ico 31333532
  orsr-cli search --name \"ESET\"
  orsr-cli detail 937053
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
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search legal entities in RPO by IČO or name.
    Search(SearchArgs),
    /// Fetch the full RPO record by RPO entity id (from search results, not IČO).
    Detail { id: u64 },
    /// Resolve orsr.sk extract URLs (aktuálny + úplný výpis) for an IČO.
    Links { ico: String },
    /// Fetch the accounting-entity record (registeruz.sk) for an IČO.
    Ruz { ico: String },
    /// Raw GET against the RPO base URL (escape hatch).
    Api { path: String },
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// IČO (1–8 digits; padded to 8 with leading zeros for the API call).
    #[arg(long)]
    pub ico: Option<String>,

    /// Name fragment (matches historical names too; filter results yourself).
    #[arg(long)]
    pub name: Option<String>,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = RegistryClient::new()?;

    let (body, source_entries) = match cli.command {
        Command::Search(args) => (search(&client, args)?, Vec::new()),
        Command::Detail { id } => {
            let body = client.rpo_get(&format!("entity/{id}"), &[])?;
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

fn search(client: &RegistryClient, args: SearchArgs) -> Result<String> {
    match (args.ico, args.name) {
        (Some(ico), None) => {
            let ico = normalize_ico(&ico)?;
            client.rpo_get("search", &[("identifier", ico.as_str())])
        }
        (None, Some(name)) => client.rpo_get("search", &[("fullName", name.as_str())]),
        _ => Err(anyhow!("search needs exactly one of --ico or --name")),
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

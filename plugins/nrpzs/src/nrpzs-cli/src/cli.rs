use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::http::NrpzsClient;
use crate::transform;
use crate::util::normalize_ico;

const VALID_CISELNIKY: &[&str] = &[
    "druhy-poskytovatelu",
    "druhy-pece",
    "obory-pece",
    "formy-pece",
    "druhy-zarizeni",
    "oddeleni",
    "pracoviste",
    "zdravotnicka-zarizeni-ciselnik",
    "mista-poskytovani-ciselnik",
    "poskytovatele-ciselnik",
];

const ROOT_HELP: &str = "\
nrpzs-cli - Národní registr poskytovatelů zdravotních služeb (ÚZIS) lookup.

Output: JSON on stdout (consume with `jq`); on failure an `ERROR:` line on stderr
and a non-zero exit code. Read-only, no API key needed.

Override base URL with NRPZS_BASE_URL (default: https://nrpzs-api.uzis.cz/api/v1).
";

#[derive(Parser, Debug)]
#[command(
    name = "nrpzs-cli",
    version,
    about = "NRPZS healthcare-provider registry CLI",
    long_about = ROOT_HELP,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Detail of one místo poskytování by its MistoPoskytovaniId.
    Detail { misto_poskytovani_id: String },
    /// Oddělení (departments) of a zdravotnické zařízení by ZdravotnickeZarizeniId.
    Oddeleni { zdravotnicke_zarizeni_id: String },
    /// Ordinační doba of a zdravotnické zařízení by ZdravotnickeZarizeniId.
    #[command(name = "ordinacni-doba")]
    OrdinacniDoba { zdravotnicke_zarizeni_id: String },
    /// Print a code list (číselník) by name.
    Ciselnik { name: String },
    /// Search providers by IČO / name / obor péče.
    Search(SearchArgs),
    /// Aggregate all places of one provider (by IČO) into a single profile.
    Profil { ico: String },
    /// Raw GET access (escape hatch).
    #[command(subcommand)]
    Api(ApiCommand),
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// IČO (1–8 digits).
    #[arg(long)]
    pub ico: Option<String>,
    /// Name or part of name (min. 3 characters).
    #[arg(long)]
    pub nazev: Option<String>,
    /// Obor péče - code (e.g. L42) or name (e.g. urologie). Only with --level mista.
    #[arg(long)]
    pub obor: Option<String>,
    /// Kraj - code (e.g. CZ020) or name. Narrows --obor/--nazev locally; --level mista only.
    #[arg(long)]
    pub kraj: Option<String>,
    /// Okres - code (e.g. CZ020B) or name. Narrows --obor/--nazev locally; --level mista only.
    #[arg(long)]
    pub okres: Option<String>,
    /// Granularity of the result.
    #[arg(long, value_enum, default_value_t = Level::Mista)]
    pub level: Level,
    /// Page offset (ignored when --kraj/--okres is set; those page through all).
    #[arg(long, default_value_t = 0)]
    pub offset: u32,
    /// Page size (max 100; ignored when --kraj/--okres is set).
    #[arg(long, default_value_t = 100)]
    pub limit: u32,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq)]
pub enum Level {
    /// Místa poskytování (richer: address, contacts, obor). Default.
    Mista,
    /// Zdravotnická zařízení (facility level).
    Zarizeni,
}

#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    /// Raw GET against the NRPZS base URL.
    Get { path: String },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = NrpzsClient::new()?;

    let body = match cli.command {
        Command::Detail { misto_poskytovani_id } => {
            client.get_raw(&format!("/mista-poskytovani/{misto_poskytovani_id}"), &[])?
        }
        Command::Oddeleni { zdravotnicke_zarizeni_id } => client.get_raw_allow_empty(
            &format!("/zdravotnicka-zarizeni/{zdravotnicke_zarizeni_id}/oddeleni"),
            &[],
        )?,
        Command::OrdinacniDoba { zdravotnicke_zarizeni_id } => client.get_raw_allow_empty(
            &format!("/zdravotnicka-zarizeni/{zdravotnicke_zarizeni_id}/ordinacni-doba"),
            &[],
        )?,
        Command::Ciselnik { name } => {
            if !VALID_CISELNIKY.contains(&name.as_str()) {
                return Err(anyhow::anyhow!(
                    "unknown číselník {name:?} - valid names: {}",
                    VALID_CISELNIKY.join(", ")
                ));
            }
            client.get_raw(&format!("/{name}"), &[])?
        }
        Command::Search(args) => search(&client, args)?,
        Command::Profil { ico } => {
            let ico8 = normalize_ico(&ico)?;
            let rows = client.fetch_all("/mista-poskytovani", &[("ico", &ico8)])?;
            if rows.is_empty() {
                return Err(anyhow::anyhow!("no provider found for IČO {ico8}"));
            }
            serde_json::to_string_pretty(&transform::profil(&ico8, &rows))?
        }
        Command::Api(ApiCommand::Get { path }) => client.get_raw(&path, &[])?,
    };

    print!("{body}");
    if !body.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn search(client: &NrpzsClient, a: SearchArgs) -> Result<String> {
    if a.ico.is_none() && a.nazev.is_none() && a.obor.is_none() {
        return Err(anyhow::anyhow!(
            "search needs at least one of --ico, --nazev, --obor"
        ));
    }
    if let Some(n) = &a.nazev {
        if n.trim().chars().count() < 3 {
            return Err(anyhow::anyhow!(
                "--nazev must be at least 3 characters (NRPZS API limit)"
            ));
        }
    }
    if a.limit > 100 {
        return Err(anyhow::anyhow!("--limit must be ≤ 100 (NRPZS API page limit)"));
    }
    let region = a.kraj.is_some() || a.okres.is_some();
    if a.level == Level::Zarizeni && (a.obor.is_some() || region) {
        return Err(anyhow::anyhow!(
            "--level zarizeni supports only --ico/--nazev (that endpoint has no obor/kraj/okres)"
        ));
    }
    if region && a.obor.is_none() && a.nazev.is_none() {
        return Err(anyhow::anyhow!(
            "--kraj/--okres can only narrow an --obor or --nazev search, not stand alone"
        ));
    }

    let ico8 = match &a.ico {
        Some(i) => Some(normalize_ico(i)?),
        None => None,
    };
    let obor_kod = match &a.obor {
        Some(o) => Some(resolve_obor(client, o)?),
        None => None,
    };

    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(i) = &ico8 {
        params.push(("ico", i));
    }
    if let Some(n) = &a.nazev {
        params.push(("nazev", n));
    }
    if let Some(c) = &obor_kod {
        params.push(("oborPece", c));
    }

    let path = match a.level {
        Level::Mista => "/mista-poskytovani",
        Level::Zarizeni => "/zdravotnicka-zarizeni",
    };

    if region {
        let mut rows = client.fetch_all(path, &params)?;
        if let Some(k) = &a.kraj {
            rows = transform::filter_region(rows, "KrajCode", "Kraj", k);
        }
        if let Some(o) = &a.okres {
            rows = transform::filter_region(rows, "OkresCode", "Okres", o);
        }
        Ok(serde_json::to_string_pretty(&serde_json::Value::Array(rows))?)
    } else {
        let off = a.offset.to_string();
        let lim = a.limit.to_string();
        let mut p = params.clone();
        p.push(("offset", &off));
        p.push(("limit", &lim));
        client.get_raw(path, &p)
    }
}

fn resolve_obor(client: &NrpzsClient, input: &str) -> Result<String> {
    let codelist = client.get_array("/obory-pece", &[])?;
    transform::obor_code(&codelist, input).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown obor péče {input:?} - run `nrpzs-cli ciselnik obory-pece` to list codes/names"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_ciselnik_names_are_accepted() {
        assert!(VALID_CISELNIKY.contains(&"obory-pece"));
        assert!(VALID_CISELNIKY.contains(&"druhy-zarizeni"));
        assert!(!VALID_CISELNIKY.contains(&"neexistuje"));
    }
}

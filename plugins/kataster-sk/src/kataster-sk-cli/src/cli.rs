use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::http::KatasterClient;
use crate::sources;

const ROOT_HELP: &str = "\
kataster-sk-cli - Slovak land cadastre (kataster nehnuteľností) lookup.

Output: JSON on stdout (consume with `jq`); on failure, an `ERROR:` line on stderr
and a non-zero exit code. API responses are passed through verbatim.

Examples:
  kataster-sk-cli ku \"Ruzinov\"
  kataster-sk-cli parcela 805556 \"1234/9\"
  kataster-sk-cli detail 1070374091
  kataster-sk-cli detail 123456 --register e --geometry

Anonymous parcel data only - owner names and title-deed (LV) contents are not
available without a verified ESKN login; hand the user a deep link instead.
Override base URLs with ZBGIS_BASE_URL and ESKN_BASE_URL.
";

#[derive(Parser, Debug)]
#[command(
    name = "kataster-sk-cli",
    version,
    about = "Slovak land cadastre (ZBGIS / ESKN) CLI",
    long_about = ROOT_HELP,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Geocode cadastral units, municipalities and places by name.
    Ku { query: String },
    /// Find parcels by number within a cadastral unit (KU code from `ku`).
    Parcela {
        ku_code: u32,
        parcel_number: String,
    },
    /// Fetch parcel attributes by object id (from `parcela` results).
    Detail(DetailArgs),
}

#[derive(Args, Debug)]
pub struct DetailArgs {
    /// Parcel object id (`data.id` from `parcela` output).
    pub object_id: u64,

    /// Cadastre register the parcel belongs to (`parcela-c` → c, `parcela-e` → e).
    #[arg(long, value_enum, default_value_t = Register::C)]
    pub register: Register,

    /// Include the parcel polygon (WGS84).
    #[arg(long)]
    pub geometry: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Register {
    C,
    E,
}

impl Register {
    fn layer(self) -> &'static str {
        match self {
            Register::C => "parcels_c_view",
            Register::E => "parcels_e_view",
        }
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = KatasterClient::new()?;

    let mut source_entries = Vec::new();
    let body = match cli.command {
        Command::Ku { query } => {
            let query = require_nonempty(&query, "query")?;
            client.zbgis_get("", &[("q", query)])?
        }
        Command::Parcela {
            ku_code,
            parcel_number,
        } => {
            let parcel_number = require_nonempty(&parcel_number, "parcel number")?;
            let ku = ku_code.to_string();
            let body = client.zbgis_get(&format!("{ku}/mu/{ku}"), &[("q", parcel_number)])?;
            source_entries = sources::parcela_sources(&body, sources::PARCEL_SOURCES_CAP);
            body
        }
        Command::Detail(args) => {
            let path = format!("{}/MapServer/0/query", args.register.layer());
            let object_id = args.object_id.to_string();
            let geometry = if args.geometry { "true" } else { "false" };
            let mut query = vec![
                ("objectIds", object_id.as_str()),
                ("outFields", "*"),
                ("returnGeometry", geometry),
                ("f", "json"),
            ];
            if args.geometry {
                query.push(("outSR", "4326"));
            }
            crate::land_use::enrich_detail(&client.eskn_get(&path, &query)?)
        }
    };

    print!("{body}");
    if !body.ends_with('\n') {
        println!();
    }
    sources::try_publish(&source_entries);
    Ok(())
}

fn require_nonempty<'a>(value: &'a str, what: &str) -> Result<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("{what} is empty"));
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_nonempty_trims_and_rejects_blank() {
        assert_eq!(require_nonempty(" 1234/9 ", "parcel number").unwrap(), "1234/9");
        assert!(require_nonempty("   ", "query").is_err());
    }

    #[test]
    fn register_maps_to_eskn_layers() {
        assert_eq!(Register::C.layer(), "parcels_c_view");
        assert_eq!(Register::E.layer(), "parcels_e_view");
    }
}

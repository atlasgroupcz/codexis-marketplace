// Pulled in only to force vendored OpenSSL to be statically linked on Linux
// (see Cargo.toml). On macOS, native-tls uses Secure Transport, so OpenSSL is
// neither linked nor needed.
#[cfg(target_os = "linux")]
use openssl as _;

mod db;
mod dataset;
mod http;
mod manifest;
mod paths;
mod text;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(
    name = "rzpro-cli",
    version,
    about = "Czech Registry of Medical Devices (RZPRO) — local lookup over the KSRZIS OpenData CSV datasets."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Search devices by name, trade name, or manufacturer (with optional filters).
    Search {
        /// Free text; all words must match (order-independent, diacritics-insensitive).
        #[arg(num_args = 0..)]
        query: Vec<String>,
        /// Filter by device kind, e.g. "in vitro", "implantabilní" (substring).
        #[arg(long)]
        druh: Option<String>,
        /// Filter by risk class, exact, e.g. "III", "IIa", "IVDR C".
        #[arg(long)]
        riziko: Option<String>,
        /// Filter by manufacturer country, e.g. "Německo" (substring).
        #[arg(long)]
        stat: Option<String>,
        /// Only devices expiring on or before this ISO date, e.g. 2027-01-01.
        #[arg(long)]
        expiruje_do: Option<String>,
        /// Max number of results to return (total match count is still reported).
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// List the distinct values (with counts) of a filterable column.
    Values {
        #[arg(value_enum)]
        sloupec: ValueColumn,
    },
    /// Full record of one device by Evidenční číslo.
    Detail { evid: String },
    /// Variants / catalogue numbers of a device by Evidenční číslo.
    Varianty { evid: String },
    /// Reverse: find the device a catalogue number belongs to.
    Varianta { katalog: String },
    /// Persons (manufacturer / distributor) linked to a device by Evidenční číslo.
    Osoby { evid: String },
    /// A registered person, its activities, and devices, by Registrační číslo or IČ.
    Osoba { key: String },
    /// Download datasets if a newer publication is available.
    Refresh {
        /// Re-download even if the local copy is current.
        #[arg(long)]
        force: bool,
    },
    /// Index metadata (publication date and row counts).
    Index {
        #[command(subcommand)]
        sub: IndexCmd,
    },
}

#[derive(Subcommand)]
enum IndexCmd {
    /// Show local dataset version and row counts.
    Status,
}

#[derive(Clone, ValueEnum)]
enum ValueColumn {
    Druh,
    Riziko,
    Stat,
}

impl ValueColumn {
    fn key(&self) -> &'static str {
        match self {
            ValueColumn::Druh => "druh",
            ValueColumn::Riziko => "riziko",
            ValueColumn::Stat => "stat",
        }
    }
}

fn print_json(v: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

/// Stamp a query result with the local data's publication date (provenance),
/// so callers can cite "RZPRO as of <datum>".
fn with_datum(v: Value) -> Value {
    match (v, manifest::load()) {
        (Value::Object(mut m), Some(man)) => {
            m.insert("datum".to_string(), json!(man.datum));
            Value::Object(m)
        }
        (v, _) => v,
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Search {
            query,
            druh,
            riziko,
            stat,
            expiruje_do,
            limit,
        } => {
            dataset::ensure_data()?;
            let filters = db::Filters {
                druh,
                riziko,
                stat,
                expiruje_do,
            };
            print_json(&with_datum(db::search(&query.join(" "), &filters, limit)?))
        }
        Command::Values { sloupec } => {
            dataset::ensure_data()?;
            print_json(&with_datum(db::values(sloupec.key())?))
        }
        Command::Detail { evid } => {
            dataset::ensure_data()?;
            print_json(&with_datum(db::detail(&evid)?))
        }
        Command::Varianty { evid } => {
            dataset::ensure_data()?;
            print_json(&with_datum(db::varianty(&evid)?))
        }
        Command::Varianta { katalog } => {
            dataset::ensure_data()?;
            print_json(&with_datum(db::varianta(&katalog)?))
        }
        Command::Osoby { evid } => {
            dataset::ensure_data()?;
            print_json(&with_datum(db::osoby(&evid)?))
        }
        Command::Osoba { key } => {
            dataset::ensure_data()?;
            print_json(&with_datum(db::osoba(&key)?))
        }
        Command::Refresh { force } => {
            let out = dataset::refresh(force)?;
            let v = match out {
                dataset::RefreshOutcome::UpToDate { datum } => json!({
                    "status": "up-to-date", "datum": datum
                }),
                dataset::RefreshOutcome::Updated { datum, counts } => json!({
                    "status": "updated", "datum": datum, "counts": counts
                }),
            };
            print_json(&v)
        }
        Command::Index { sub } => match sub {
            IndexCmd::Status => {
                let m = manifest::load()
                    .ok_or_else(|| anyhow!("no local index; run `rzpro-cli refresh`"))?;
                print_json(&json!({
                    "datum": m.datum,
                    "downloadedAt": m.downloaded_at,
                    "counts": m.counts,
                }))
            }
        },
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e:#}");
        std::process::exit(1);
    }
}

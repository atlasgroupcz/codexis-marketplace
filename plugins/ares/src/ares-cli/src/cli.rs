use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};

use crate::error::normalize_ico;
use crate::http::AresClient;

const ROOT_HELP: &str = "\
ares-cli — Czech business registry (ARES) lookup.

Output: JSON on stdout (consume with `jq`); on failure, an `ERROR:` line on stderr
and a non-zero exit code. Pass-through of the ARES response body, never reformatted.

Examples:
  ares-cli detail 27082440
  ares-cli detail 27082440 --register vr
  ares-cli detail 27082440 --register rzp
  ares-cli search --name \"ATLAS\" --pocet 5
  ares-cli search --obec \"Praha\" --pravni-forma 112
  ares-cli api get \"/ekonomicke-subjekty/27082440\"
  ares-cli api post \"/ekonomicke-subjekty/vyhledat\" '{\"obchodniJmeno\":\"ATLAS\",\"pocet\":5}'

Override base URL with ARES_BASE_URL (default: https://ares.gov.cz/ekonomicke-subjekty-v-be/rest).
";

#[derive(Parser, Debug)]
#[command(
    name = "ares-cli",
    version,
    about = "Czech business registry (ARES) CLI",
    long_about = ROOT_HELP,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Fetch detail of a single ekonomický subjekt by IČO.
    Detail(DetailArgs),
    /// Search ekonomické subjekty by name / address / legal form.
    Search(SearchArgs),
    /// Raw HTTP access (escape hatch).
    #[command(subcommand)]
    Api(ApiCommand),
}

#[derive(Args, Debug)]
pub struct DetailArgs {
    /// IČO (1–8 digits; padded to 8 with leading zeros for the API call).
    pub ico: String,

    /// Which register to read.
    ///
    /// Note: ARES v3 has no separate VAT-payer (RPDPH) endpoint — DIČ + VAT
    /// registration history are returned by `basic` under `dic` and
    /// `seznamRegistraci`.
    #[arg(long, value_enum, default_value_t = Register::Basic)]
    pub register: Register,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Obchodní jméno (substring; ARES does its own matching).
    #[arg(long)]
    pub name: Option<String>,

    /// Obec (city/town).
    #[arg(long)]
    pub obec: Option<String>,

    /// PSČ (postal code).
    #[arg(long)]
    pub psc: Option<String>,

    /// Právní forma — ARES code (e.g. 112 for s.r.o.).
    #[arg(long = "pravni-forma")]
    pub pravni_forma: Option<String>,

    /// Okres (district code or name as accepted by ARES).
    #[arg(long)]
    pub okres: Option<String>,

    /// Specific IČO (server-side filter; useful only with paging).
    #[arg(long)]
    pub ico: Option<String>,

    /// Page offset (records to skip).
    #[arg(long, default_value_t = 0)]
    pub start: u32,

    /// Page size (records per response).
    #[arg(long, default_value_t = 10)]
    pub pocet: u32,
}

#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    /// Raw GET against the ARES base URL.
    Get { path: String },
    /// Raw POST with JSON body against the ARES base URL.
    Post { path: String, body: String },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Register {
    /// Core ekonomický subjekt record (incl. DIČ and seznamRegistraci).
    Basic,
    /// Veřejný / obchodní rejstřík (statutory bodies, partners, etc.).
    Vr,
    /// Registr ekonomických subjektů (statistical).
    Res,
    /// Živnostenský rejstřík.
    Rzp,
}

impl Register {
    fn endpoint_prefix(self) -> &'static str {
        match self {
            Register::Basic => "/ekonomicke-subjekty",
            Register::Vr => "/ekonomicke-subjekty-vr",
            Register::Res => "/ekonomicke-subjekty-res",
            Register::Rzp => "/ekonomicke-subjekty-rzp",
        }
    }
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let client = AresClient::new()?;

    let body = match cli.command {
        Command::Detail(args) => detail(&client, args)?,
        Command::Search(args) => search(&client, args)?,
        Command::Api(ApiCommand::Get { path }) => client.get(&path)?,
        Command::Api(ApiCommand::Post { path, body }) => {
            let payload: Value = serde_json::from_str(&body)
                .map_err(|e| anyhow!("--body is not valid JSON: {e}"))?;
            client.post_json(&path, &payload)?
        }
    };

    print!("{body}");
    if !body.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn detail(client: &AresClient, args: DetailArgs) -> Result<String> {
    let ico = normalize_ico(&args.ico)?;
    let path = format!("{}/{}", args.register.endpoint_prefix(), ico);
    client.get(&path)
}

fn search(client: &AresClient, args: SearchArgs) -> Result<String> {
    if args.pocet == 0 {
        return Err(anyhow!("--pocet must be > 0"));
    }
    if args.pocet > 1000 {
        return Err(anyhow!("--pocet must be ≤ 1000 (ARES limit)"));
    }

    let mut payload = serde_json::Map::new();
    if let Some(name) = args.name {
        payload.insert("obchodniJmeno".into(), json!(name));
    }
    if let Some(ico) = args.ico {
        let normalized = normalize_ico(&ico)?;
        payload.insert("ico".into(), json!([normalized]));
    }
    if let Some(pf) = args.pravni_forma {
        payload.insert("pravniForma".into(), json!([pf]));
    }

    let mut sidlo = serde_json::Map::new();
    if let Some(obec) = args.obec {
        sidlo.insert("obec".into(), json!(obec));
    }
    if let Some(psc) = args.psc {
        sidlo.insert("psc".into(), json!(psc));
    }
    if let Some(okres) = args.okres {
        sidlo.insert("okres".into(), json!(okres));
    }
    if !sidlo.is_empty() {
        payload.insert("sidlo".into(), Value::Object(sidlo));
    }

    payload.insert("start".into(), json!(args.start));
    payload.insert("pocet".into(), json!(args.pocet));

    if payload.len() == 2 {
        return Err(anyhow!(
            "search needs at least one filter — use --name, --ico, --obec, --psc, --pravni-forma or --okres"
        ));
    }

    client.post_json("/ekonomicke-subjekty/vyhledat", &Value::Object(payload))
}

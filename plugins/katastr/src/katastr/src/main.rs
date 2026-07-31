//! katastr-cli - unified CLI for ČÚZK lookups, tracking, settings and
//! notifications.
//!
//! Port of the Python `bin/katastr-cli`. All business logic lives in the
//! `katastr-core` crate shared with `katastr-cgi` - this binary is just an
//! argparse + pretty-printing shell around it.

use std::process::ExitCode;

use clap::{ArgGroup, Parser, Subcommand};
use katastr_core::state::ProceedingState;
use katastr_core::tracking::CheckResult;
use katastr_core::{
    api_client, geom, lookup, notify, notify_settings, pravo_stavby, ruian, settings, tracking,
    KatastrError,
};

#[derive(Parser)]
#[command(
    name = "katastr-cli",
    about = "Katastr nemovitostí: dotazy do ČÚZK, sledování řízení, nastavení a notifikace.",
    version
)]
struct Cli {
    #[command(subcommand)]
    namespace: Namespace,
}

// Doc komentáře na clap typech nejsou komentáře pro programátora - clap je
// vypisuje jako text `--help`. Proto jsou česky, stejně jako chybové hlášky:
// čte je uživatel. Všechno ostatní v této crate je anglicky.
#[derive(Subcommand)]
enum Namespace {
    /// Sledování katastrálních řízení
    Tracking {
        #[command(subcommand)]
        cmd: TrackingCmd,
    },
    /// Přímé volání API ČÚZK
    Api {
        #[command(subcommand)]
        cmd: ApiCmd,
    },
    /// Správa API klíče k ČÚZK
    Settings {
        #[command(subcommand)]
        cmd: SettingsCmd,
    },
    /// Nastavení notifikací
    Notify {
        #[command(subcommand)]
        cmd: NotifyCmd,
    },
    /// Přeloží adresu nebo název území na kód RÚIAN
    #[command(group(ArgGroup::new("search_vstup").required(true).args(["text", "klic"])))]
    Search {
        /// Adresa, obec, katastrální území, okres, kraj nebo ulice
        text: Option<String>,
        /// Druh hledané věci, např. Obec nebo KatastralniUzemi. S ním se
        /// hledá přímo v příslušném rejstříku a vrátí se všechny nálezy;
        /// bez něj se vrátí návrhy i s druhem, ze kterých se dá vybrat.
        #[arg(long)]
        typ: Option<String>,
        /// Klíč z návrhu (pole `klic`). Dohledá přesně ten jeden záznam.
        #[arg(long, conflicts_with_all = ["text", "typ"])]
        klic: Option<String>,
    },
    /// Parcela podle katastrálního území a čísla, nebo podle ID
    #[command(group(ArgGroup::new("parcela_vstup").required(true).args(["id", "ku"])))]
    Parcela {
        #[arg(long, requires = "cislo", conflicts_with = "id",
              value_parser = clap::value_parser!(u32).range(meze::KOD_KU))]
        ku: Option<u32>,
        #[arg(long, requires = "ku", conflicts_with = "id",
              value_parser = clap::value_parser!(u32).range(meze::CISLO_PARCELY))]
        cislo: Option<u32>,
        #[arg(long, requires = "cislo", conflicts_with = "id",
              value_parser = clap::value_parser!(u32).range(meze::PODDELENI))]
        poddeleni: Option<u32>,
        /// 1 = stavební, 2 = pozemková
        #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=2))]
        druh: u8,
        #[arg(long, value_enum, default_value_t = TypParcely::Pkn)]
        typ: TypParcely,
        /// Původ parcely ZE. Povinný u --typ PZE.
        #[arg(long = "puvod-ze", value_enum, required_if_eq("typ", "PZE"))]
        puvod_ze: Option<PuvodZe>,
        #[arg(long)]
        id: Option<String>,
    },
    /// Co je kolem parcely, stavby nebo adresy
    #[command(group(ArgGroup::new("okoli_stred").required(true).args(["parcela", "stavba", "adresa"])))]
    Okoli {
        #[arg(long)]
        parcela: Option<String>,
        #[arg(long)]
        stavba: Option<String>,
        #[arg(long)]
        adresa: Option<String>,
        /// Poloměr v metrech. ČÚZK omezuje plochu polygonu na 5000 m2,
        /// takže prakticky nejvýš zhruba 35.
        #[arg(long, value_parser = kladny_polomer)]
        metru: f64,
    },
    /// Stavba podle adresního místa, části obce a čísla, nebo podle ID
    #[command(group(ArgGroup::new("stavba_vstup").required(true).args(["id", "adresni_misto", "cast_obce"])))]
    #[command(group(ArgGroup::new("stavba_cislo").args(["cp", "ce"]).requires("cast_obce")))]
    Stavba {
        #[arg(long = "adresni-misto")]
        adresni_misto: Option<u32>,
        #[arg(long = "cast-obce", requires = "stavba_cislo",
              value_parser = clap::value_parser!(u32).range(meze::KOD_CASTI_OBCE))]
        cast_obce: Option<u32>,
        /// Číslo popisné
        #[arg(long, value_parser = clap::value_parser!(u32).range(meze::CISLO_DOMOVNI))]
        cp: Option<u32>,
        /// Číslo evidenční
        #[arg(long, value_parser = clap::value_parser!(u32).range(meze::CISLO_DOMOVNI))]
        ce: Option<u32>,
        #[arg(long)]
        id: Option<String>,
    },
    /// Jednotka (byt nebo nebytový prostor)
    #[command(group(ArgGroup::new("jednotka_vstup").required(true).args(["id", "cast_obce"])))]
    Jednotka {
        #[arg(long = "cast-obce", requires_all = ["cp", "cj"],
              value_parser = clap::value_parser!(u32).range(meze::KOD_CASTI_OBCE))]
        cast_obce: Option<u32>,
        #[arg(long, conflicts_with = "id",
              value_parser = clap::value_parser!(u32).range(meze::CISLO_DOMOVNI))]
        cp: Option<u32>,
        #[arg(long, conflicts_with = "id",
              value_parser = clap::value_parser!(u32).range(meze::CISLO_JEDNOTKY))]
        cj: Option<u32>,
        #[arg(long)]
        id: Option<String>,
    },
    /// Řízení podle čísla, ID, nebo seznam přijatých za den
    #[command(group(ArgGroup::new("rizeni_vstup").required(true).args(["cislo", "id", "prijate_dne"])))]
    Rizeni {
        /// Číslo řízení, např. V-123/2026-701
        cislo: Option<String>,
        #[arg(long)]
        id: Option<String>,
        #[arg(long = "prijate-dne", requires_all = ["pracoviste", "typ"])]
        prijate_dne: Option<String>,
        #[arg(long, requires = "prijate_dne",
              value_parser = clap::value_parser!(u32).range(meze::KOD_PRACOVISTE))]
        pracoviste: Option<u32>,
        #[arg(long, value_enum, requires = "prijate_dne")]
        typ: Option<TypRizeni>,
    },
    /// Právo stavby podle ID, nebo podle parcely či stavby
    #[command(group(ArgGroup::new("pravo_vstup").required(true).args(["id", "parcela", "stavba"])))]
    PravoStavby {
        id: Option<String>,
        #[arg(long)]
        parcela: Option<String>,
        #[arg(long)]
        stavba: Option<String>,
    },
    /// Sousední parcely
    Sousedni { id: String },
    /// Katastrální pracoviště: adresář nebo jedno podle kódu
    Pracoviste {
        #[arg(long, value_parser = clap::value_parser!(u32).range(meze::KOD_PRACOVISTE))]
        kod: Option<u32>,
    },
}

/// `TypParcely` as the swagger declares it - exactly these two.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum TypParcely {
    #[value(name = "PKN")]
    Pkn,
    #[value(name = "PZE")]
    Pze,
}

impl TypParcely {
    fn as_str(self) -> &'static str {
        match self {
            Self::Pkn => "PKN",
            Self::Pze => "PZE",
        }
    }
}

/// Ranges the swagger declares for these query parameters. Enforcing them at
/// parse time means a value that ČÚZK would reject never costs a call from the
/// 500-per-period quota.
mod meze {
    pub const KOD_KU: std::ops::RangeInclusive<i64> = 1..=999_999;
    pub const CISLO_PARCELY: std::ops::RangeInclusive<i64> = 1..=99_999;
    pub const PODDELENI: std::ops::RangeInclusive<i64> = 1..=999;
    pub const KOD_CASTI_OBCE: std::ops::RangeInclusive<i64> = 1..=999_999;
    pub const CISLO_DOMOVNI: std::ops::RangeInclusive<i64> = 1..=99_999;
    pub const CISLO_JEDNOTKY: std::ops::RangeInclusive<i64> = 0..=99_999_999;
    pub const KOD_PRACOVISTE: std::ops::RangeInclusive<i64> = 1..=999;
}

/// Source of record for a ZE parcel. The swagger marks the parameter optional,
/// but the server rejects `TypParcely=PZE` without it - verified live, which is
/// why `--typ PZE` never worked before this flag existed.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum PuvodZe {
    /// 3 = evidence nemovitostí
    #[value(name = "3")]
    EvidenceNemovitosti,
    /// 4 = pozemkový katastr
    #[value(name = "4")]
    PozemkovyKatastr,
    /// 6 = přídělový plán nebo jiný podklad
    #[value(name = "6")]
    PridelovyPlan,
}

impl PuvodZe {
    fn as_u8(self) -> u8 {
        match self {
            Self::EvidenceNemovitosti => 3,
            Self::PozemkovyKatastr => 4,
            Self::PridelovyPlan => 6,
        }
    }
}

/// The proceeding types `Rizeni/PrijateDne` actually accepts. Its swagger
/// declares all five, but the server rejects `PD` and `ZPV` with a 400 and
/// "Must be one of [V,Z,PGP]" - verified live. Only `--prijate-dne` takes this
/// flag, so the narrow set is the right one; a number like `PD-1/2026-701`
/// still parses fine as a proceeding number.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum TypRizeni {
    #[value(name = "V")]
    V,
    #[value(name = "Z")]
    Z,
    #[value(name = "PGP")]
    Pgp,
}

impl TypRizeni {
    fn as_str(self) -> &'static str {
        match self {
            Self::V => "V",
            Self::Z => "Z",
            Self::Pgp => "PGP",
        }
    }
}

/// Rejects a radius clap's own numeric parsing would accept. Infinity and NaN
/// have no JSON spelling, so they would reach ČÚZK as `null` coordinates and
/// come back as an opaque error.
fn kladny_polomer(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| format!("{s} není číslo"))?;
    if !v.is_finite() || v <= 0.0 {
        return Err(String::from("musí být kladné konečné číslo v metrech"));
    }
    Ok(v)
}

#[derive(Subcommand)]
enum TrackingCmd {
    /// Začne sledovat řízení
    Add {
        /// Číslo řízení, např. V-123/2026-701
        cislo: String,
        /// Volitelné vlastní označení
        #[arg(long, default_value = "")]
        label: String,
    },
    /// Vypíše všechna sledovaná řízení
    List,
    /// Úplný stav jednoho sledovaného řízení
    Show { cislo: String },
    /// Zkontroluje sledovaná řízení, jestli se něco změnilo
    Check {
        /// Konkrétní řízení (výchozí: všechna)
        cislo: Option<String>,
    },
    /// Označí změny jako přečtené
    Confirm {
        cislo: String,
        /// Označí všechny nepotvrzené změny (výchozí, když není --change)
        #[arg(long)]
        all: bool,
        /// Označí jen tuhle jednu změnu (index od nuly)
        #[arg(long, value_name = "INDEX")]
        change: Option<usize>,
    },
    /// Přestane řízení sledovat
    Remove { cislo: String },
    /// Nastaví nebo změní vlastní označení sledovaného řízení
    #[command(name = "set-label")]
    SetLabel {
        cislo: String,
        /// Nové označení (prázdný řetězec ho smaže)
        label: String,
    },
}

#[derive(Subcommand)]
enum ApiCmd {
    /// GET dotaz na ČÚZK
    Get {
        /// Cesta v API ČÚZK, např. "/api/v1/Parcely/12345"
        path: String,
    },
}

#[derive(Subcommand)]
enum SettingsCmd {
    /// Stav API klíče, kvóta a platnost
    Show,
    /// Ověří API klíč u ČÚZK a uloží ho
    Set { key: String },
    /// Vyzkouší uložený API klíč proti ČÚZK
    Test,
}

#[derive(Subcommand)]
enum NotifyCmd {
    /// Aktuální nastavení notifikací
    Show,
    /// Změní nastavení notifikací
    Set {
        /// Zapne e-mail
        #[arg(long, conflicts_with = "no_email")]
        email: bool,
        /// Vypne e-mail
        #[arg(long = "no-email")]
        no_email: bool,
        /// Zapne zvoneček v aplikaci
        #[arg(long, conflicts_with = "no_inapp")]
        inapp: bool,
        /// Vypne zvoneček v aplikaci
        #[arg(long = "no-inapp")]
        no_inapp: bool,
    },
    /// Správa příjemců e-mailů
    Recipients {
        #[command(subcommand)]
        cmd: RecipientsCmd,
    },
}

#[derive(Subcommand)]
enum RecipientsCmd {
    /// Přidá e-mail příjemce
    Add { email: String },
    /// Odebere e-mail příjemce (nezáleží na velikosti písmen)
    Remove { email: String },
    /// Nahradí celý seznam příjemců
    Set { emails: Vec<String> },
    /// Smaže všechny příjemce
    Clear,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    // Keep the user settings file present + migrated, like the Python CLI's
    // `notify_settings.ensure_settings()` at startup.
    let _ = notify_settings::ensure_settings();

    let result = match cli.namespace {
        Namespace::Tracking { cmd } => run_tracking(cmd),
        Namespace::Api { cmd } => run_api(cmd),
        Namespace::Settings { cmd } => run_settings(cmd),
        Namespace::Notify { cmd } => run_notify(cmd),
        Namespace::Search { text, typ, klic } => cmd_search(text, typ, klic),
        Namespace::Parcela { ku, cislo, poddeleni, druh, typ, puvod_ze, id } => {
            cmd_parcela(ku, cislo, poddeleni, druh, typ, puvod_ze, id)
        }
        Namespace::Okoli { parcela, stavba, adresa, metru } => {
            cmd_okoli(parcela, stavba, adresa, metru)
        }
        Namespace::Stavba { adresni_misto, cast_obce, cp, ce, id } => {
            cmd_stavba(adresni_misto, cast_obce, cp, ce, id)
        }
        Namespace::Jednotka { cast_obce, cp, cj, id } => cmd_jednotka(cast_obce, cp, cj, id),
        Namespace::Rizeni { cislo, id, prijate_dne, pracoviste, typ } => {
            cmd_rizeni(cislo, id, prijate_dne, pracoviste, typ)
        }
        Namespace::PravoStavby { id, parcela, stavba } => cmd_pravo_stavby(id, parcela, stavba),
        Namespace::Sousedni { id } => print_get(&lookup::sousedni_parcely(&id)),
        Namespace::Pracoviste { kod } => match kod {
            Some(k) => print_get(&lookup::pracoviste_detail(k)),
            None => print_get(&lookup::pracoviste_list()),
        },
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Katastr(e)) => {
            eprintln!("ERROR: {e}");
            ExitCode::from(1)
        }
        Err(CliError::Usage(msg)) => {
            eprintln!("ERROR: {msg}");
            ExitCode::from(1)
        }
    }
}

// ── tracking commands ────────────────────────────────────────────────────────

fn run_tracking(cmd: TrackingCmd) -> CliResult {
    match cmd {
        TrackingCmd::Add { cislo, label } => cmd_add(&cislo, &label),
        TrackingCmd::List => cmd_list(),
        TrackingCmd::Show { cislo } => cmd_show(&cislo),
        TrackingCmd::Check { cislo } => cmd_check(cislo.as_deref()),
        TrackingCmd::Confirm { cislo, all, change } => cmd_confirm(&cislo, all, change),
        TrackingCmd::Remove { cislo } => cmd_remove(&cislo),
        TrackingCmd::SetLabel { cislo, label } => cmd_set_label(&cislo, &label),
    }
}

fn cmd_add(cislo: &str, label: &str) -> CliResult {
    match tracking::add(cislo, label) {
        Ok(state) => {
            eprintln!("OK: Řízení {} přidáno ke sledování.", state.cislo_rizeni);
            eprintln!("  Stav: {}", state.stav);
            eprintln!("  Operací: {}", state.provedene_operace.len());
            if !state.label.is_empty() {
                eprintln!("  Označení: {}", state.label);
            }
            Ok(())
        }
        // Idempotent add: already tracking is a success, not an error.
        Err(KatastrError::ProceedingAlreadyTracked(_)) => {
            let canonical = tracking::normalize_number(cislo).unwrap_or_else(|_| cislo.to_string());
            eprintln!("OK: Řízení {canonical} je již sledováno.");
            if !label.is_empty() {
                eprintln!(
                    "  Pozn.: --label platí jen při prvním přidání; \
                     štítek změníš příkazem set-label."
                );
            }
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

fn cmd_list() -> CliResult {
    let proceedings = tracking::list_all();
    if proceedings.is_empty() {
        println!("Žádná sledovaná řízení.");
        return Ok(());
    }
    println!("Sledovaná řízení ({}):\n", proceedings.len());
    for state in &proceedings {
        print_state_short(state);
        println!();
    }
    Ok(())
}

fn cmd_show(cislo: &str) -> CliResult {
    let state = tracking::show(cislo)?;
    let json = serde_json::to_string_pretty(&state).map_err(KatastrError::from)?;
    println!("{json}");
    Ok(())
}

fn cmd_check(cislo: Option<&str>) -> CliResult {
    let results: Vec<CheckResult> = match cislo {
        Some(c) => vec![tracking::check_one(c)?],
        None => tracking::check_all(),
    };

    if results.is_empty() {
        println!("Žádná sledovaná řízení ke kontrole.");
        return Ok(());
    }

    let mut total_changes = 0usize;
    for r in &results {
        if !r.ok {
            println!("  {}: CHYBA - {}", r.display_name, r.error.as_deref().unwrap_or(""));
            continue;
        }
        let Some(change) = r.change.as_ref() else {
            println!("  {}: beze změn", r.display_name);
            continue;
        };
        total_changes += 1;
        println!("  {}: 1 nová změna", r.display_name);
        for line in tracking::describe_change(change) {
            println!("    {line}");
        }
    }
    println!(
        "\nKontrola dokončena. Celkem {} nových změn u {} řízení.",
        total_changes,
        results.len()
    );

    // Push notifications per the stored settings - mirrors the Python CLI so a
    // change is notified regardless of which route first catches it. The routing
    // lives in notify::dispatch; here we only render its outcome.
    let dispatched = notify::dispatch(&results);
    if let Some(email) = dispatched.email {
        match email {
            notify::EmailOutcome::Sent(sent) => {
                eprintln!("E-mailem odeslán souhrn ({sent} řízení se změnou).")
            }
            notify::EmailOutcome::Skipped => eprintln!("Žádné nové změny - e-mail neodeslán."),
            notify::EmailOutcome::Failed(e) => eprintln!("{e}"),
        }
    }
    if dispatched.in_app_sent {
        eprintln!("Upozornění odesláno do aplikace (zvoneček).");
    }
    Ok(())
}

fn cmd_confirm(cislo: &str, all: bool, change_index: Option<usize>) -> CliResult {
    if all && change_index.is_some() {
        return Err(CliError::Usage(
            "Použij buď --all NEBO --change <INDEX>, ne obojí.".into(),
        ));
    }
    let marked = tracking::confirm(cislo, change_index)?;
    if marked == 0 {
        eprintln!("Žádné nepřečtené změny.");
    } else {
        eprintln!("OK: Označeno jako přečteno: {marked} změn");
    }
    Ok(())
}

fn cmd_remove(cislo: &str) -> CliResult {
    let canonical = tracking::normalize_number(cislo)?;
    match tracking::remove(&canonical) {
        Ok(()) => {
            eprintln!("OK: Řízení {canonical} odstraněno ze sledování.");
            Ok(())
        }
        // Idempotent remove: nothing to remove is a success.
        Err(KatastrError::ProceedingNotTracked(_)) => {
            eprintln!("OK: Řízení {canonical} není sledováno (nic k odebrání).");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

fn cmd_set_label(cislo: &str, label: &str) -> CliResult {
    let state = tracking::set_label(cislo, label)?;
    if label.is_empty() {
        eprintln!("OK: Označení řízení {} odstraněno.", state.cislo_rizeni);
    } else {
        eprintln!("OK: Označení řízení {} nastaveno na: {label}", state.cislo_rizeni);
    }
    Ok(())
}

// ── api commands ─────────────────────────────────────────────────────────────

fn run_api(cmd: ApiCmd) -> CliResult {
    match cmd {
        ApiCmd::Get { path } => print_json(&api_client::get(&path)?),
    }
}

// ── domain lookup commands ───────────────────────────────────────────────────
// Named subcommands over `lookup`, `ruian`, `geom` and `pravo_stavby` -
// `api get` stays the escape hatch for whatever did not get its own command.

/// The one place a response is serialized, so output has the same shape across
/// every command.
fn print_json(v: &serde_json::Value) -> CliResult {
    let json = serde_json::to_string_pretty(v).map_err(KatastrError::from)?;
    println!("{json}");
    Ok(())
}

/// Calls a path and prints the response verbatim, as the rest of the repo does.
fn print_get(path: &str) -> CliResult {
    print_json(&api_client::get(path)?)
}

fn cmd_search(text: Option<String>, typ: Option<String>, klic: Option<String>) -> CliResult {
    // clap's `search_vstup` group guarantees exactly one of the two.
    match (klic.as_deref(), text.as_deref()) {
        (Some(k), _) => print_json(&ruian::by_key(k)?),
        (None, Some(t)) => print_json(&ruian::search(t, typ.as_deref())?),
        _ => unreachable!("clap group search_vstup"),
    }
}

fn cmd_parcela(
    ku: Option<u32>,
    cislo: Option<u32>,
    poddeleni: Option<u32>,
    druh: u8,
    typ: TypParcely,
    puvod_ze: Option<PuvodZe>,
    id: Option<String>,
) -> CliResult {
    // clap's `parcela_vstup` group guarantees exactly one of these shapes.
    let path = match (id.as_deref(), ku, cislo) {
        (Some(i), _, _) => lookup::parcela_detail(i),
        (None, Some(k), Some(c)) => lookup::parcela_search(k, c, poddeleni, druh, typ.as_str(), puvod_ze.map(PuvodZe::as_u8)),
        _ => unreachable!("clap group parcela_vstup"),
    };
    print_get(&path)
}

/// The property's definition point, however it was reached.
///
/// `Vyhledani` returns `data` as an array, a detail lookup as an object, so
/// both are tried.
fn definicni_bod(data: &serde_json::Value) -> Option<(f64, f64)> {
    let bod = data
        .pointer("/data/definicniBod")
        .or_else(|| data.pointer("/data/0/definicniBod"))?;
    Some((bod.get("x")?.as_f64()?, bod.get("y")?.as_f64()?))
}

fn cmd_okoli(
    parcela: Option<String>,
    stavba: Option<String>,
    adresa: Option<String>,
    metru: f64,
) -> CliResult {
    // clap's `okoli_stred` group guarantees exactly one centre.
    let zdroj = match (parcela.as_deref(), stavba.as_deref(), adresa.as_deref()) {
        (Some(id), None, None) => api_client::get(&lookup::parcela_detail(id))?,
        (None, Some(id), None) => api_client::get(&lookup::stavba_detail(id))?,
        (None, None, Some(text)) => {
            // The address resolves to a code first, the code to a building,
            // whose definition point becomes the centre.
            let nalez = ruian::search(text, Some("AdresniMisto"))?;
            let Some(kod) = nalez.get("kod").and_then(|k| k.as_u64()) else {
                return Err(CliError::Usage(format!(
                    "Adresu {text} se nepodařilo jednoznačně přeložit. \
                     Zkus `katastr-cli search` a použij --stavba s konkrétním ID."
                )));
            };
            api_client::get(&lookup::stavba_adresni_misto(kod as u32))?
        }
        _ => unreachable!("clap group okoli_stred"),
    };

    let Some((x, y)) = definicni_bod(&zdroj) else {
        return Err(CliError::Usage(
            "Odpověď neobsahuje definiční bod, okolí nelze spočítat.".into(),
        ));
    };

    let coords = geom::square_around(x, y, metru);
    let parcely = api_client::get(&lookup::parcely_polygon(&coords))?;
    let stavby = api_client::get(&lookup::stavby_polygon(&coords))?;
    print_json(&serde_json::json!({
        "stred": { "x": x, "y": y },
        "metru": metru,
        "parcely": parcely,
        "stavby": stavby,
    }))
}

fn cmd_stavba(
    adresni_misto: Option<u32>,
    cast_obce: Option<u32>,
    cp: Option<u32>,
    ce: Option<u32>,
    id: Option<String>,
) -> CliResult {
    // clap's `stavba_vstup` and `stavba_cislo` groups guarantee exactly one
    // entry point, and exactly one of --cp/--ce alongside --cast-obce.
    let path = match (id.as_deref(), adresni_misto, cast_obce, cp, ce) {
        (Some(i), ..) => lookup::stavba_detail(i),
        (None, Some(kod), ..) => lookup::stavba_adresni_misto(kod),
        (None, None, Some(co), Some(n), None) => lookup::stavba_search(co, 1, n),
        (None, None, Some(co), None, Some(n)) => lookup::stavba_search(co, 2, n),
        _ => unreachable!("clap groups stavba_vstup + stavba_cislo"),
    };
    print_get(&path)
}

fn cmd_jednotka(
    cast_obce: Option<u32>,
    cp: Option<u32>,
    cj: Option<u32>,
    id: Option<String>,
) -> CliResult {
    // clap's `jednotka_vstup` group guarantees exactly one of these shapes.
    let path = match (id.as_deref(), cast_obce, cp, cj) {
        (Some(i), ..) => lookup::jednotka_detail(i),
        (None, Some(co), Some(n), Some(j)) => lookup::jednotka_search(co, 1, n, j),
        _ => unreachable!("clap group jednotka_vstup"),
    };
    print_get(&path)
}

fn cmd_rizeni(
    cislo: Option<String>,
    id: Option<String>,
    prijate_dne: Option<String>,
    pracoviste: Option<u32>,
    typ: Option<TypRizeni>,
) -> CliResult {
    // clap's `rizeni_vstup` group guarantees exactly one entry point, and
    // --prijate-dne requires both --pracoviste and --typ.
    if let Some(datum) = prijate_dne {
        let (Some(kp), Some(t)) = (pracoviste, typ) else {
            unreachable!("clap requires_all on prijate_dne");
        };
        return print_get(&lookup::rizeni_prijate_dne(t.as_str(), kp, &datum));
    }
    if let Some(i) = id.as_deref() {
        return print_get(&lookup::rizeni_detail(i));
    }
    let Some(c) = cislo else {
        unreachable!("clap group rizeni_vstup");
    };
    let parsed = tracking::parse_proceeding_number(&c)?;
    print_get(&lookup::rizeni_search(
        &parsed.typ_rizeni,
        parsed.poradove_cislo,
        parsed.rok,
        parsed.kod_pracoviste,
    ))
}

fn cmd_pravo_stavby(
    id: Option<String>,
    parcela: Option<String>,
    stavba: Option<String>,
) -> CliResult {
    // `dotaz` remembers which identifier was actually checked, so a
    // "no right" answer can name it instead of coming back as a bare null
    // nobody can trace back to a parcel or building.
    let (path, dotaz) = match (id.as_deref(), parcela.as_deref(), stavba.as_deref()) {
        (Some(i), None, None) => (lookup::pravo_stavby_detail(i), serde_json::json!({ "id": i })),
        (None, Some(p), None) => {
            (lookup::pravo_stavby_parcela(p), serde_json::json!({ "parcela": p }))
        }
        (None, None, Some(s)) => {
            (lookup::pravo_stavby_stavba(s), serde_json::json!({ "stavba": s }))
        }
        // clap's `pravo_vstup` group guarantees exactly one of the three.
        _ => unreachable!("clap group pravo_vstup"),
    };
    match api_client::get(&path) {
        Ok(data) => print_json(&data),
        // A 404 means two things here, and only one of them is an error.
        Err(KatastrError::ApiHttp { status: 404, message }) => {
            match pravo_stavby::interpret_404(&message) {
                pravo_stavby::Missing::NoRight => print_json(&serde_json::json!({
                    "pravoStavby": null,
                    "dotaz": dotaz,
                })),
                pravo_stavby::Missing::NotFound => {
                    Err(KatastrError::ApiHttp { status: 404, message }.into())
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}

// ── settings commands ────────────────────────────────────────────────────────

/// Prints ČÚZK's operational messages. Among other things they carry an
/// approaching key expiry, so they must not be swallowed.
///
/// Its own call, even though `zpravy` rides along elsewhere: the endpoints
/// under `AplikacniSluzby` carry no ordinary envelope. Neither `StavUctu` nor
/// `AktualnostDat` nor `Health` returns `zpravy` - verified live - so it cannot
/// be read off a response already in hand.
///
/// Best-effort: with no network, whatever is known locally still gets printed.
fn print_provozni_informace() {
    let Ok(v) = api_client::get(lookup::PROVOZNI_INFORMACE) else {
        return;
    };
    for z in v.get("zpravy").and_then(|z| z.as_array()).into_iter().flatten() {
        if let Some(text) = z.get("text").and_then(|t| t.as_str()) {
            println!("ČÚZK hlásí: {text}");
        }
    }
}

/// ČÚZK's `status` is an undocumented integer, so printing it raw told nobody
/// anything. Every entry carries its own `description` in plain words - print
/// those instead of guessing what the number means. Falls back to the raw
/// status rather than printing nothing at all.
fn print_health(v: &serde_json::Value) {
    let mut printed = false;
    for e in v.get("entries").and_then(|e| e.as_array()).into_iter().flatten() {
        if let (Some(check), Some(desc)) = (
            e.get("check").and_then(serde_json::Value::as_str),
            e.get("description").and_then(serde_json::Value::as_str),
        ) {
            println!("{check}: {desc}");
            printed = true;
        }
    }
    if !printed {
        if let Some(s) = v.get("status") {
            println!("Health: {s}");
        }
    }
}

fn run_settings(cmd: SettingsCmd) -> CliResult {
    match cmd {
        SettingsCmd::Show => {
            let status = settings::key_status();
            if !status.configured {
                println!("API klíč není nastaven.");
                return Ok(());
            }
            println!("API klíč: {} (configured)", status.masked);

            // Account status is a supplement, not a precondition: with no
            // network, whatever is known locally still gets printed.
            match api_client::get(lookup::STAV_UCTU) {
                Ok(v) => {
                    if let (Some(pouzito), Some(limit)) = (
                        v.get("provedenoVolani").and_then(|x| x.as_u64()),
                        v.get("limitVolani").and_then(|x| x.as_u64()),
                    ) {
                        println!("Volání: {pouzito} / {limit} za období {}",
                            v.get("aktualniObdobi").and_then(|x| x.as_str()).unwrap_or("?"));
                    }
                    if let Some(exp) = v.get("expiraceApiKey").and_then(|x| x.as_str()) {
                        println!("Platnost klíče do: {exp}");
                    }
                }
                Err(e) => eprintln!("Stav účtu se nepodařilo zjistit: {e}"),
            }

            print_provozni_informace();
            Ok(())
        }
        SettingsCmd::Set { key } => {
            settings::set_api_key(&key)?;
            eprintln!(
                "OK: API klíč byl ověřen u ČÚZK a uložen ({}).",
                settings::mask_key(key.trim())
            );
            Ok(())
        }
        SettingsCmd::Test => {
            if !settings::is_configured() {
                return Err(CliError::Katastr(KatastrError::ApiKeyMissing(
                    "API klíč není nastaven.".into(),
                )));
            }
            settings::validate_key(&settings::read_api_key())?;
            eprintln!("OK: API klíč funguje (ČÚZK odpovědělo).");

            if let Ok(v) = api_client::get(lookup::AKTUALNOST_DAT) {
                if let Some(k) = v.get("aktualnostDatK").and_then(|x| x.as_str()) {
                    println!("Data aktuální k: {k}");
                }
            }
            if let Ok(v) = api_client::get(lookup::HEALTH) {
                print_health(&v);
            }
            print_provozni_informace();
            Ok(())
        }
    }
}

// ── notify commands ──────────────────────────────────────────────────────────

fn run_notify(cmd: NotifyCmd) -> CliResult {
    match cmd {
        NotifyCmd::Show => {
            let json = serde_json::to_string_pretty(&notify_settings::get_notify())
                .map_err(KatastrError::from)?;
            println!("{json}");
            Ok(())
        }
        NotifyCmd::Set {
            email,
            no_email,
            inapp,
            no_inapp,
        } => {
            let email = tri_state(email, no_email);
            let inapp = tri_state(inapp, no_inapp);
            let eff = notify_settings::set_notify(email, inapp, None)?;
            eprintln!(
                "OK: Notifikace - e-mail: {}, in-app: {}.",
                eff.email, eff.in_app
            );
            Ok(())
        }
        NotifyCmd::Recipients { cmd } => cmd_recipients(cmd),
    }
}

/// Collapse an on/off flag pair into the `Option<bool>` the setter expects:
/// `--flag` → `Some(true)`, `--no-flag` → `Some(false)`, neither → `None`.
fn tri_state(on: bool, off: bool) -> Option<bool> {
    if on {
        Some(true)
    } else if off {
        Some(false)
    } else {
        None
    }
}

fn cmd_recipients(cmd: RecipientsCmd) -> CliResult {
    let mut current = notify_settings::get_notify().recipients;
    match cmd {
        RecipientsCmd::Add { email } => current.push(email.trim().to_string()),
        RecipientsCmd::Remove { email } => {
            let key = email.trim().to_lowercase();
            current.retain(|r| r.to_lowercase() != key);
        }
        RecipientsCmd::Set { emails } => current = emails,
        RecipientsCmd::Clear => current.clear(),
    }
    // The signed-in user always receives the e-mail, so drop their own address.
    let current = notify::without_self(&current);
    let eff = notify_settings::set_notify(None, None, Some(current))?;
    let joined = if eff.recipients.is_empty() {
        "(žádní; e-mail chodí přihlášenému uživateli)".to_string()
    } else {
        eff.recipients.join(", ")
    };
    eprintln!("OK: Příjemci - {joined}.");
    Ok(())
}

// ── output helpers ───────────────────────────────────────────────────────────

fn print_state_short(state: &ProceedingState) {
    let label_str = if state.label.is_empty() {
        String::new()
    } else {
        format!(" ({})", state.label)
    };
    let stav = if state.stav.is_empty() { "?" } else { &state.stav };
    let ops = state.provedene_operace.len();
    let changes = state.changes.len();
    // Same "what counts as unread" rule the overview/CGI use, via the shared
    // cdx-watch helper - so the CLI listing can't drift from the dashboard.
    let unconfirmed = tracking::unconfirmed_count(&state.changes);
    let badge = if unconfirmed > 0 {
        format!(" • {unconfirmed} nepřečtených změn")
    } else {
        String::new()
    };

    println!("  {}{label_str}", state.cislo_rizeni);
    println!("    Stav: {stav}");
    println!("    Operací: {ops}, Změn: {changes}{badge}");
}

// ── error plumbing ───────────────────────────────────────────────────────────

type CliResult = Result<(), CliError>;

enum CliError {
    Katastr(KatastrError),
    Usage(String),
}

impl From<KatastrError> for CliError {
    fn from(e: KatastrError) -> Self {
        CliError::Katastr(e)
    }
}

//! katastr-cli - unified CLI for ČÚZK lookups, tracking, settings and
//! notifications.
//!
//! Port of the Python `bin/katastr-cli`. All business logic lives in the
//! `katastr-core` crate shared with `katastr-cgi` - this binary is just an
//! argparse + pretty-printing shell around it.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use katastr_core::state::ProceedingState;
use katastr_core::tracking::CheckResult;
use katastr_core::{
    api_client, notify, notify_settings, settings, tracking, KatastrError,
};

#[derive(Parser)]
#[command(
    name = "katastr-cli",
    about = "ČÚZK lookups, tracking, settings and notifications.",
    version
)]
struct Cli {
    #[command(subcommand)]
    namespace: Namespace,
}

#[derive(Subcommand)]
enum Namespace {
    /// Track cadastral proceedings
    Tracking {
        #[command(subcommand)]
        cmd: TrackingCmd,
    },
    /// Raw ČÚZK API access
    Api {
        #[command(subcommand)]
        cmd: ApiCmd,
    },
    /// Manage ČÚZK API key
    Settings {
        #[command(subcommand)]
        cmd: SettingsCmd,
    },
    /// Manage notification settings
    Notify {
        #[command(subcommand)]
        cmd: NotifyCmd,
    },
}

#[derive(Subcommand)]
enum TrackingCmd {
    /// Start tracking a proceeding
    Add {
        /// Proceeding number, e.g. V-123/2026-701
        cislo: String,
        /// Optional user-friendly label
        #[arg(long, default_value = "")]
        label: String,
    },
    /// List all tracked proceedings
    List,
    /// Show full state of a tracked proceeding
    Show { cislo: String },
    /// Check tracked proceedings for changes
    Check {
        /// Specific proceeding (default: all)
        cislo: Option<String>,
    },
    /// Mark changes as read
    Confirm {
        cislo: String,
        /// Mark all unconfirmed changes (default if --change is not given)
        #[arg(long)]
        all: bool,
        /// Mark only this specific change (0-based index)
        #[arg(long, value_name = "INDEX")]
        change: Option<usize>,
    },
    /// Stop tracking a proceeding
    Remove { cislo: String },
    /// Set or change the user-friendly label of a tracked proceeding
    #[command(name = "set-label")]
    SetLabel {
        cislo: String,
        /// New label string (use "" to clear)
        label: String,
    },
}

#[derive(Subcommand)]
enum ApiCmd {
    /// GET request to ČÚZK
    Get {
        /// ČÚZK API path, e.g. "/api/v1/Parcely/12345"
        path: String,
    },
}

#[derive(Subcommand)]
enum SettingsCmd {
    /// Show masked API key status
    Show,
    /// Validate and store API key
    Set { key: String },
    /// Test current API key against ČÚZK
    Test,
}

#[derive(Subcommand)]
enum NotifyCmd {
    /// Show current notification settings
    Show,
    /// Change notification settings
    Set {
        /// Turn e-mail on
        #[arg(long, conflicts_with = "no_email")]
        email: bool,
        /// Turn e-mail off
        #[arg(long = "no-email")]
        no_email: bool,
        /// Turn the in-app bell on
        #[arg(long, conflicts_with = "no_inapp")]
        inapp: bool,
        /// Turn the in-app bell off
        #[arg(long = "no-inapp")]
        no_inapp: bool,
    },
    /// Manage e-mail recipients
    Recipients {
        #[command(subcommand)]
        cmd: RecipientsCmd,
    },
}

#[derive(Subcommand)]
enum RecipientsCmd {
    /// Add a recipient e-mail
    Add { email: String },
    /// Remove a recipient e-mail (case-insensitive)
    Remove { email: String },
    /// Replace the whole recipient list
    Set { emails: Vec<String> },
    /// Remove all recipients
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
        ApiCmd::Get { path } => {
            let result = api_client::get(&path)?;
            let json = serde_json::to_string_pretty(&result).map_err(KatastrError::from)?;
            println!("{json}");
            Ok(())
        }
    }
}

// ── settings commands ────────────────────────────────────────────────────────

fn run_settings(cmd: SettingsCmd) -> CliResult {
    match cmd {
        SettingsCmd::Show => {
            let status = settings::key_status();
            if !status.configured {
                println!("API klíč není nastaven.");
            } else {
                println!("API klíč: {} (configured)", status.masked);
            }
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

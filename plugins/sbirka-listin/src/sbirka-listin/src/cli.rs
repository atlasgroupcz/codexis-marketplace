//! Command surface.
//!
//! Output: JSON on stdout - built from the scraped register pages (the source is
//! HTML, so we serialize our own shape). Errors go to stderr as `ERROR: …` with
//! exit code 2.

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;

use cdx_common::ico::normalize_ico;
use cdx_common::time::now_utc;
use cdx_notify::notify::without_self;
use cdx_notify::notify_settings::NotifySettings;
use cdx_watch::change::{confirm_changes, unconfirmed_count};
use cdx_watch::track::{check_all, check_one};

use crate::api;
use crate::dokument;
use crate::http::PortalClient;
use crate::notify;
use crate::sources;
use crate::watch::{self, SbirkaListinWatcher};

const ROOT_HELP: &str = "\
Examples:
  sbirka-listin-cli listiny 27082440
  sbirka-listin-cli listiny 27082440 | jq '.listiny[] | select(.typ | contains(\"účetní závěrka [2025]\"))'
  sbirka-listin-cli dokument 27082440 'B 8573/SL91/MSPH'
  sbirka-listin-cli sledovani add 27082440
  sbirka-listin-cli sledovani check
";

#[derive(Parser)]
#[command(
    name = "sbirka-listin-cli",
    version,
    about = "Sbírka listin veřejných rejstříků ČR - seznam listin subjektu, stažení dokumentu, hlídání přírůstků.",
    disable_version_flag = true,
    disable_help_subcommand = true,
    subcommand_required = true,
    arg_required_else_help = true,
    after_help = ROOT_HELP
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Seznam listin subjektu podle IČO.
    Listiny {
        #[arg(value_parser = ico_value_parser, value_name = "ICO")]
        ico: String,
    },
    /// Stáhne jeden dokument podle celého čísla listiny ("B 8573/SL95/MSPH" -
    /// dej ho do uvozovek, má mezeru). Už stažený vrátí bez dotazu na portál.
    Dokument {
        #[arg(value_parser = ico_value_parser, value_name = "ICO")]
        ico: String,
        #[arg(value_name = "CISLO")]
        cislo: String,
    },
    /// Hlídání přírůstků ve sbírce listin.
    Sledovani {
        #[command(subcommand)]
        command: SledovaniCommand,
    },
    /// Nastavení upozornění (e-mail, in-app zvoneček, příjemci).
    Notify {
        #[command(subcommand)]
        command: NotifyCommand,
    },
}

#[derive(Subcommand)]
enum SledovaniCommand {
    /// Začne sledovat subjekt. Současný seznam se stane výchozím stavem, takže
    /// první kontrola hlásí jen to, co přibude potom.
    Add {
        #[arg(value_parser = ico_value_parser, value_name = "ICO")]
        ico: String,
    },
    /// Vypíše sledované subjekty; ty s nepotvrzenými změnami první.
    List,
    /// Vypíše celý uložený stav jednoho subjektu včetně historie změn.
    Show {
        #[arg(value_parser = ico_value_parser, value_name = "ICO")]
        ico: String,
    },
    /// Přestane sledovat subjekt.
    Remove {
        #[arg(value_parser = ico_value_parser, value_name = "ICO")]
        ico: String,
    },
    /// Zkontroluje přírůstky. Bez IČO projde všechny sledované subjekty.
    Check {
        #[arg(value_parser = ico_value_parser, value_name = "ICO")]
        ico: Option<String>,
    },
    /// Označí změny jako přečtené.
    Confirm {
        #[arg(value_parser = ico_value_parser, value_name = "ICO")]
        ico: String,
        /// Jen změna na tomto indexu; bez něj všechny nepotvrzené.
        #[arg(long)]
        index: Option<usize>,
    },
}

#[derive(Subcommand)]
enum NotifyCommand {
    /// Vypíše aktuální nastavení upozornění.
    Show,
    /// Změní nastavení upozornění.
    Set {
        /// Zapne e-mail.
        #[arg(long, conflicts_with = "no_email")]
        email: bool,
        /// Vypne e-mail.
        #[arg(long = "no-email")]
        no_email: bool,
        /// Zapne in-app zvoneček.
        #[arg(long, conflicts_with = "no_inapp")]
        inapp: bool,
        /// Vypne in-app zvoneček.
        #[arg(long = "no-inapp")]
        no_inapp: bool,
    },
    /// Správa e-mailových příjemců.
    Recipients {
        #[command(subcommand)]
        command: RecipientsCommand,
    },
}

#[derive(Subcommand)]
enum RecipientsCommand {
    /// Přidá příjemce.
    Add {
        #[arg(value_name = "EMAIL")]
        email: String,
    },
    /// Odebere příjemce (bez ohledu na velikost písmen).
    Remove {
        #[arg(value_name = "EMAIL")]
        email: String,
    },
    /// Nahradí celý seznam příjemců.
    Set {
        #[arg(value_name = "EMAIL")]
        emails: Vec<String>,
    },
    /// Odebere všechny příjemce.
    Clear,
}

fn ico_value_parser(raw: &str) -> Result<String, String> {
    // Canonicalize at parse time, not at request time: the portal keys on the
    // zero-padded eight-digit form and the store slugs on whatever we hold.
    normalize_ico(raw).map_err(|e| e.to_string())
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    // Keep the user settings file present and migrated, like the Python watchers'
    // `ensure_settings()` at startup. Best-effort: a settings write failure must
    // not block a read-only lookup.
    let _ = NotifySettings::new(watch::APP_ID).ensure_settings();
    match cli.command {
        Command::Listiny { ico } => listiny(&ico),
        Command::Dokument { ico, cislo } => stahni(&ico, &cislo),
        Command::Sledovani { command } => match command {
            SledovaniCommand::Add { ico } => sledovani_add(&ico),
            SledovaniCommand::List => sledovani_list(),
            SledovaniCommand::Show { ico } => sledovani_show(&ico),
            SledovaniCommand::Remove { ico } => sledovani_remove(&ico),
            SledovaniCommand::Check { ico } => sledovani_check(ico.as_deref()),
            SledovaniCommand::Confirm { ico, index } => sledovani_confirm(&ico, index),
        },
        Command::Notify { command } => match command {
            NotifyCommand::Show => notify_show(),
            NotifyCommand::Set {
                email,
                no_email,
                inapp,
                no_inapp,
            } => notify_set(tri_state(email, no_email), tri_state(inapp, no_inapp)),
            NotifyCommand::Recipients { command } => notify_recipients(command),
        },
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

fn listiny(ico8: &str) -> Result<()> {
    let client = PortalClient::new()?;
    let detail = api::fetch_detail(&client, ico8)?;
    sources::try_publish(&sources::listiny_source(&detail));
    print_json(&serde_json::to_value(&detail)?);
    Ok(())
}

fn stahni(ico8: &str, cislo: &str) -> Result<()> {
    let client = PortalClient::new()?;
    let fetched = dokument::fetch(&client, ico8, cislo)?;
    print_json(&json!({
        "ico": ico8,
        "cislo": fetched.cislo,
        "path": fetched.path,
        "url": fetched.url,
        "reused": fetched.reused,
    }));
    Ok(())
}

fn sledovani_add(ico8: &str) -> Result<()> {
    let store = watch::store();
    if store.exists(ico8) {
        anyhow::bail!("IČO {ico8} už je sledované");
    }
    let client = PortalClient::new()?;
    let detail = api::fetch_detail(&client, ico8)?;
    let snapshot = watch::seed(&detail);
    store.save(ico8, &snapshot)?;
    print_json(&json!({
        "ico": snapshot.ico,
        "nazev": snapshot.nazev,
        "spisovaznacka": snapshot.spisovaznacka,
        "listin": snapshot.listiny.len(),
        "added_on": snapshot.added_on,
    }));
    Ok(())
}

fn sledovani_list() -> Result<()> {
    print_json(&json!(watch::overview_rows(&watch::store())));
    Ok(())
}

fn sledovani_show(ico8: &str) -> Result<()> {
    let snapshot = watch::store()
        .load::<watch::Snapshot>(ico8)
        .ok_or_else(|| anyhow::anyhow!("IČO {ico8} není sledované"))?;
    print_json(&serde_json::to_value(&snapshot)?);
    Ok(())
}

fn sledovani_remove(ico8: &str) -> Result<()> {
    let store = watch::store();
    if !store.exists(ico8) {
        anyhow::bail!("IČO {ico8} není sledované");
    }
    store.remove(ico8)?;
    print_json(&json!({ "ico": ico8, "removed": true }));
    Ok(())
}

fn sledovani_check(ico8: Option<&str>) -> Result<()> {
    let client = PortalClient::new()?;
    let watcher = SbirkaListinWatcher { client: &client };
    let store = watch::store();
    let results = match ico8 {
        Some(ico) => vec![check_one(&store, &watcher, ico)?],
        None => check_all(&store, &watcher),
    };
    // Detect-and-notify are one step: the ticket's core is "upozorní uživatele".
    // Notification is best-effort (never fails the check); its outcome rides
    // along in the JSON for the automation log.
    let notifikace = notify::dispatch(&results);
    print_json(&json!({ "results": results, "notifikace": notifikace }));
    Ok(())
}

fn sledovani_confirm(ico8: &str, index: Option<usize>) -> Result<()> {
    let store = watch::store();
    let mut snapshot = store
        .load::<watch::Snapshot>(ico8)
        .ok_or_else(|| anyhow::anyhow!("IČO {ico8} není sledované"))?;
    let confirmed = confirm_changes(&mut snapshot.changes, index, &now_utc())?;
    store.save(ico8, &snapshot)?;
    print_json(&json!({
        "ico": ico8,
        "potvrzeno": confirmed,
        "nepotvrzenych_zmen": unconfirmed_count(&snapshot.changes),
    }));
    Ok(())
}

fn notify_settings() -> NotifySettings {
    NotifySettings::new(watch::APP_ID)
}

fn notify_show() -> Result<()> {
    print_json(&serde_json::to_value(notify_settings().get_notify())?);
    Ok(())
}

fn notify_set(email: Option<bool>, in_app: Option<bool>) -> Result<()> {
    let eff = notify_settings().set_notify(email, in_app, None)?;
    print_json(&serde_json::to_value(&eff)?);
    Ok(())
}

fn notify_recipients(command: RecipientsCommand) -> Result<()> {
    let mut current = notify_settings().get_notify().recipients;
    match command {
        RecipientsCommand::Add { email } => current.push(email.trim().to_string()),
        RecipientsCommand::Remove { email } => {
            let key = email.trim().to_lowercase();
            current.retain(|r| r.to_lowercase() != key);
        }
        RecipientsCommand::Set { emails } => current = emails,
        RecipientsCommand::Clear => current.clear(),
    }
    // The signed-in user always receives the e-mail, so drop their own address -
    // listing yourself as an extra is redundant.
    let current = without_self(&current);
    let eff = notify_settings().set_notify(None, None, Some(current))?;
    print_json(&json!({ "recipients": eff.recipients }));
    Ok(())
}

fn print_json(value: &serde_json::Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn ico_is_canonicalized_at_parse_time() {
        // So the value matches the portal's zero-padded form and the store slug
        // stays identical however the user typed it.
        let cli = Cli::try_parse_from(["sbirka-listin-cli", "listiny", "27082440"]).unwrap();
        let Command::Listiny { ico } = cli.command else {
            panic!("expected listiny");
        };
        assert_eq!(ico, "27082440");

        let cli = Cli::try_parse_from(["sbirka-listin-cli", "listiny", "216208"]).unwrap();
        let Command::Listiny { ico } = cli.command else {
            panic!("expected listiny");
        };
        assert_eq!(
            ico, "00216208",
            "short IČO must be left-padded to eight digits"
        );
    }

    #[test]
    fn bad_ico_is_rejected_by_the_parser() {
        assert!(Cli::try_parse_from(["sbirka-listin-cli", "listiny", "abc"]).is_err());
        assert!(Cli::try_parse_from(["sbirka-listin-cli", "listiny", "123456789"]).is_err());
    }

    #[test]
    fn dokument_takes_ico_and_the_full_cislo_listiny() {
        // IČO is canonicalized; the číslo listiny is taken verbatim (slashes and
        // a space, quoted by the shell).
        let cli = Cli::try_parse_from([
            "sbirka-listin-cli",
            "dokument",
            "27082440",
            "B 8573/SL95/MSPH",
        ])
        .unwrap();
        let Command::Dokument { ico, cislo } = cli.command else {
            panic!("expected dokument");
        };
        assert_eq!(ico, "27082440");
        assert_eq!(cislo, "B 8573/SL95/MSPH");
    }

    #[test]
    fn sledovani_check_ico_is_optional() {
        let cli = Cli::try_parse_from(["sbirka-listin-cli", "sledovani", "check"]).unwrap();
        let Command::Sledovani {
            command: SledovaniCommand::Check { ico },
        } = cli.command
        else {
            panic!("expected sledovani check");
        };
        assert!(ico.is_none());
    }

    #[test]
    fn sledovani_show_and_remove_take_a_canonical_ico() {
        let cli = Cli::try_parse_from(["sbirka-listin-cli", "sledovani", "show", "216208"]).unwrap();
        let Command::Sledovani {
            command: SledovaniCommand::Show { ico },
        } = cli.command
        else {
            panic!("expected sledovani show");
        };
        assert_eq!(ico, "00216208");

        let cli =
            Cli::try_parse_from(["sbirka-listin-cli", "sledovani", "remove", "27082440"]).unwrap();
        let Command::Sledovani {
            command: SledovaniCommand::Remove { ico },
        } = cli.command
        else {
            panic!("expected sledovani remove");
        };
        assert_eq!(ico, "27082440");
    }

    #[test]
    fn tri_state_maps_flag_pairs() {
        assert_eq!(tri_state(true, false), Some(true));
        assert_eq!(tri_state(false, true), Some(false));
        assert_eq!(tri_state(false, false), None);
    }

    #[test]
    fn notify_set_rejects_contradictory_flags() {
        // --email and --no-email cannot both be given (clap `conflicts_with`).
        assert!(Cli::try_parse_from([
            "sbirka-listin-cli",
            "notify",
            "set",
            "--email",
            "--no-email",
        ])
        .is_err());
    }

    #[test]
    fn notify_recipients_set_takes_many_emails() {
        let cli = Cli::try_parse_from([
            "sbirka-listin-cli",
            "notify",
            "recipients",
            "set",
            "a@x.com",
            "b@x.com",
        ])
        .unwrap();
        let Command::Notify {
            command: NotifyCommand::Recipients {
                command: RecipientsCommand::Set { emails },
            },
        } = cli.command
        else {
            panic!("expected notify recipients set");
        };
        assert_eq!(emails, vec!["a@x.com".to_string(), "b@x.com".to_string()]);
    }

    #[test]
    fn a_bare_invocation_is_an_error_not_a_silent_noop() {
        assert!(Cli::try_parse_from(["sbirka-listin-cli"]).is_err());
    }

    #[test]
    fn root_help_carries_examples() {
        assert!(ROOT_HELP.contains("sledovani check"));
        assert!(ROOT_HELP.contains("jq"));
    }
}

//! Deliver a summary of newly detected changes from a check run - over the
//! daemon's plugin e-mail API and/or the in-app notification bell. Port of
//! Python's `katastr_core.notify`.
//!
//! Sends only when a change was actually detected; a quiet run sends nothing.
//! The transport itself (auth, daemon URL resolution, signed-in-user
//! resolution) lives in `cdx_notify::notify`; this module owns the
//! katastr-specific summary content and the dispatch decision (`dispatch`) that
//! both the CLI and the CGI share.

use cdx_notify::app_link::resolve_app_link;
use cdx_notify::email::email_shell;
use cdx_notify::notify::{html_escape, resolve_to, send_email, send_inapp};

use crate::error::Result;
use crate::notify_settings::get_notify;
use crate::tracking::{describe_change, CheckResult};

/// Identifies this plugin so the bell's deep link resolves to the daemon's
/// `/apps/<applicationId>` route - the only route serving a plugin app. A
/// hand-written `/katastr` matches no route and lands on a 404. The component
/// half comes from `state::APP_ID`, already the single source for this
/// component's id.
const PLUGIN_NAME: &str = "katastr";
/// Notification type tag for the bell.
const NOTIFICATION_TYPE: &str = "cadastre-changed";
/// Uppercase brand label shown as the e-mail eyebrow.
const EYEBROW: &str = "Hlídač katastrálních řízení";
const FOOTER: &str = "Otevřete aplikaci Katastr v CODEXIS pro detail.";

/// Re-export the shared transport helpers the CGI/CLI call directly.
pub use cdx_notify::notify::{query_me_email, recipients_from_body, without_self};

// ── summary building ──────────────────────────────────────────────────────────

fn changed(results: &[CheckResult]) -> Vec<&CheckResult> {
    results.iter().filter(|r| r.has_change()).collect()
}

/// Just the e-mail / bell subject line for the changed proceedings. Split out so
/// the in-app bell can reuse it without rebuilding the whole e-mail body.
fn summary_subject(changed: &[&CheckResult]) -> String {
    if changed.len() == 1 {
        format!("Katastr: změna u řízení {}", changed[0].display_name)
    } else {
        format!("Katastr: změny u {} sledovaných řízení", changed.len())
    }
}

/// `(subject, body_text, body_html)` for proceedings with a newly detected
/// change. Mirrors Python's `_change_summary`.
fn change_summary(changed: &[&CheckResult]) -> (String, String, String) {
    let subject = summary_subject(changed);

    let mut text_lines: Vec<String> = vec![
        "Hlídač katastrálních řízení zaznamenal změny:".to_string(),
        String::new(),
    ];
    let mut html_blocks = String::new();

    for r in changed {
        let detail = r.change.as_ref().map(describe_change).unwrap_or_default();
        text_lines.push(format!("{}:", r.display_name));
        for line in &detail {
            text_lines.push(format!("  {line}"));
        }
        text_lines.push(String::new());

        let items: String = detail
            .iter()
            .map(|line| format!("<li style=\"margin:2px 0\">{}</li>", html_escape(line)))
            .collect();
        html_blocks.push_str(&format!(
            "<h3 style=\"margin:18px 0 6px;font:600 15px sans-serif;color:#0a0a0a\">{}</h3>\
             <ul style=\"margin:0;padding-left:18px;font:14px/1.5 sans-serif;color:#333\">{}</ul>",
            html_escape(&r.display_name),
            items
        ));
    }
    text_lines.push(FOOTER.to_string());

    let body_html = email_shell(EYEBROW, "Změny u sledovaných řízení", &html_blocks, FOOTER);
    (subject, text_lines.join("\n"), body_html)
}

/// Send one summary e-mail for proceedings with newly detected changes.
/// Returns the number of proceedings with a new change (0 → nothing sent).
/// Pass `to` to override the recipient (default: the signed-in user).
fn email_change_summary(changed: &[&CheckResult], to: Option<Vec<String>>) -> Result<usize> {
    if changed.is_empty() {
        return Ok(0);
    }
    let (subject, text, html) = change_summary(changed);
    send_email(&subject, &text, Some(&html), to.as_deref(), "katastr")?;
    Ok(changed.len())
}

/// Send an in-app (bell) notification only when there is a real change (no
/// heartbeat bell). Returns `true` if sent.
fn inapp_change_summary(changed: &[&CheckResult]) -> bool {
    if changed.is_empty() {
        return false;
    }
    // Only the subject is needed here - no HTML body is built, unlike e-mail.
    let subject = summary_subject(changed);
    // Resolution is best-effort - an unresolvable link is omitted rather than
    // sent broken.
    let link = resolve_app_link(PLUGIN_NAME, crate::state::APP_ID);
    send_inapp(&subject, NOTIFICATION_TYPE, link.as_deref())
}

// ── dispatch ──────────────────────────────────────────────────────────────────

/// The e-mail channel's outcome for a check run.
pub enum EmailOutcome {
    /// A summary e-mail was sent covering `n` changed proceedings.
    Sent(usize),
    /// E-mail is enabled, but the run was quiet so nothing was sent.
    Skipped,
    /// E-mail is enabled, but the send failed.
    Failed(String),
}

/// What [`dispatch`] did with a check run's results, so the CLI can report it.
pub struct Dispatched {
    /// E-mail channel outcome; `None` when e-mail notifications are disabled.
    pub email: Option<EmailOutcome>,
    /// `true` when the in-app bell was enabled and a notification was sent.
    pub in_app_sent: bool,
}

/// Read the stored notify settings and route a check run's freshly detected
/// changes to the enabled channels (e-mail + in-app). The single place the CLI
/// and CGI share, so the "which channels, in what order" decision lives once.
/// Best-effort: the CGI ignores the return, the CLI renders its status lines
/// from it.
pub fn dispatch(results: &[CheckResult]) -> Dispatched {
    let cfg = get_notify();
    // Compute the changed set once and share it between both channels, rather
    // than each channel re-filtering `results`.
    let changed = changed(results);

    let email = if cfg.email {
        let to = resolve_to(&cfg.recipients);
        Some(match email_change_summary(&changed, to) {
            Ok(0) => EmailOutcome::Skipped,
            Ok(n) => EmailOutcome::Sent(n),
            Err(e) => EmailOutcome::Failed(e.to_string()),
        })
    } else {
        None
    };

    let in_app_sent = cfg.in_app && inapp_change_summary(&changed);

    Dispatched { email, in_app_sent }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Change, Operace};

    fn result_with_change() -> CheckResult {
        CheckResult {
            slug: "V-1_2026-701".into(),
            display_name: "V-1/2026-701".into(),
            ok: true,
            change: Some(Change {
                detected_on: "2026-07-10T00:00:00Z".into(),
                old_stav: "Probíhá".into(),
                new_stav: "Povoleno".into(),
                new_operations: vec![Operace {
                    nazev: "Vklad".into(),
                    datum_provedeni: "2026-07-10".into(),
                }],
                stav_uhrady_changed: false,
                old_stav_uhrady: None,
                new_stav_uhrady: None,
                confirmed_on: None,
            }),
            error: None,
        }
    }

    #[test]
    fn change_summary_none_when_no_changes() {
        let quiet = CheckResult {
            slug: "V-2_2026-701".into(),
            display_name: "V-2/2026-701".into(),
            ok: true,
            change: None,
            error: None,
        };
        // A quiet run filters down to no changed proceedings, so no summary.
        assert!(changed(&[quiet]).is_empty());
    }

    #[test]
    fn change_summary_single_subject_and_body() {
        let (subject, text, html) = change_summary(&changed(&[result_with_change()]));
        assert_eq!(subject, "Katastr: změna u řízení V-1/2026-701");
        assert!(text.contains("Stav: Probíhá → Povoleno"));
        assert!(text.contains("Nová operace: Vklad (2026-07-10)"));
        assert!(html.contains("V-1/2026-701"));
        assert!(text.trim_end().ends_with(FOOTER));
    }

    #[test]
    fn change_summary_plural_subject() {
        let (subject, _, _) =
            change_summary(&changed(&[result_with_change(), result_with_change()]));
        assert_eq!(subject, "Katastr: změny u 2 sledovaných řízení");
    }
}

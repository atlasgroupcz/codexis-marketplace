//! Deliver a summary of newly detected changes from a check run - over the
//! daemon's plugin e-mail API and/or the in-app notification bell. Port of
//! Python's `katastr_core.notify`.
//!
//! Sends only when a change was actually detected; a quiet run sends nothing.
//! The transport itself (auth, daemon URL resolution, signed-in-user
//! resolution) lives in `cdx_notify::notify`; this module owns only the
//! katastr-specific summary content.

use cdx_notify::notify::{html_escape, send_email, send_inapp};

use crate::error::Result;
use crate::tracking::{describe_change, CheckResult};

/// Deep link into the plugin's app + notification type tag for the bell.
const APP_LINK: &str = "/katastr";
const NOTIFICATION_TYPE: &str = "cadastre-changed";
const FOOTER: &str = "Otevřete aplikaci Katastr v CODEXIS pro detail.";

/// Re-export the shared transport helpers so the CGI/CLI call sites keep
/// working unchanged.
pub use cdx_notify::notify::{query_me_email, resolve_to, without_self};

// ── summary building ──────────────────────────────────────────────────────────

fn changed(results: &[CheckResult]) -> Vec<&CheckResult> {
    results.iter().filter(|r| r.has_change()).collect()
}

fn email_shell(heading: &str, inner_html: &str) -> String {
    format!(
        "<div style=\"max-width:560px;font:14px sans-serif;color:#0a0a0a\">\
         <p style=\"margin:0;font:600 12px sans-serif;letter-spacing:.14em;\
         text-transform:uppercase;color:#5ea500\">Hlídač katastrálních řízení</p>\
         <h2 style=\"margin:4px 0 2px;font:400 22px Georgia,serif\">{heading}</h2>\
         {inner_html}\
         <p style=\"margin-top:20px;font:13px sans-serif;color:#777\">{FOOTER}</p></div>"
    )
}

/// `(subject, body_text, body_html)` for proceedings with a newly detected
/// change. Mirrors Python's `_change_summary`.
fn change_summary(changed: &[&CheckResult]) -> (String, String, String) {
    let subject = if changed.len() == 1 {
        format!("Katastr: změna u řízení {}", changed[0].cislo_rizeni)
    } else {
        format!("Katastr: změny u {} sledovaných řízení", changed.len())
    };

    let mut text_lines: Vec<String> =
        vec!["Hlídač katastrálních řízení zaznamenal změny:".to_string(), String::new()];
    let mut html_blocks = String::new();

    for r in changed {
        let detail = r.change.as_ref().map(describe_change).unwrap_or_default();
        text_lines.push(format!("{}:", r.cislo_rizeni));
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
            html_escape(&r.cislo_rizeni),
            items
        ));
    }
    text_lines.push(FOOTER.to_string());

    let body_html = email_shell("Změny u sledovaných řízení", &html_blocks);
    (subject, text_lines.join("\n"), body_html)
}

/// Build `(subject, body_text, body_html)` for proceedings with a new change,
/// or `None` when nothing changed.
pub fn build_summary(results: &[CheckResult]) -> Option<(String, String, String)> {
    let changed = changed(results);
    if changed.is_empty() {
        None
    } else {
        Some(change_summary(&changed))
    }
}

/// Send one summary e-mail for proceedings with newly detected changes.
/// Returns the number of proceedings with a new change (0 → nothing sent).
/// Pass `to` to override the recipient (default: the signed-in user).
pub fn email_change_summary(results: &[CheckResult], to: Option<Vec<String>>) -> Result<usize> {
    let changed = changed(results);
    if changed.is_empty() {
        return Ok(0);
    }
    let (subject, text, html) = change_summary(&changed);
    send_email(&subject, &text, Some(&html), to.as_deref(), "katastr")?;
    Ok(changed.len())
}

/// Send an in-app (bell) notification only when there is a real change (no
/// heartbeat bell). Returns `true` if sent.
pub fn inapp_change_summary(results: &[CheckResult]) -> bool {
    let changed = changed(results);
    if changed.is_empty() {
        return false;
    }
    let subject = change_summary(&changed).0;
    // cdx-notify's send_inapp takes an optional `action` before the deep link;
    // katastr has no action button, only the app link.
    send_inapp(&subject, NOTIFICATION_TYPE, None, Some(APP_LINK))
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Change, Operace};

    fn result_with_change() -> CheckResult {
        CheckResult {
            cislo_rizeni: "V-1/2026-701".into(),
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
    fn build_summary_none_when_no_changes() {
        let quiet = CheckResult {
            cislo_rizeni: "V-2/2026-701".into(),
            ok: true,
            change: None,
            error: None,
        };
        assert!(build_summary(&[quiet]).is_none());
    }

    #[test]
    fn build_summary_single_subject_and_body() {
        let (subject, text, html) = build_summary(&[result_with_change()]).unwrap();
        assert_eq!(subject, "Katastr: změna u řízení V-1/2026-701");
        assert!(text.contains("Stav: Probíhá → Povoleno"));
        assert!(text.contains("Nová operace: Vklad (2026-07-10)"));
        assert!(html.contains("V-1/2026-701"));
        assert!(text.trim_end().ends_with(FOOTER));
    }

    #[test]
    fn build_summary_plural_subject() {
        let (subject, _, _) =
            build_summary(&[result_with_change(), result_with_change()]).unwrap();
        assert_eq!(subject, "Katastr: změny u 2 sledovaných řízení");
    }
}

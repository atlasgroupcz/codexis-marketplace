//! Deliver a summary of newly filed documents from a check run - over the
//! daemon's plugin e-mail API and/or the in-app notification bell.
//!
//! Sends only when a check actually detected new listiny; a quiet run sends
//! nothing. The transport (auth, daemon URL, signed-in-user resolution) and the
//! branded e-mail shell live in `cdx_notify`; this module owns only the
//! sbirka-listin summary content and the dispatch decision.

use cdx_notify::email::email_shell;
use cdx_notify::app_link::resolve_app_link;
use cdx_notify::notify::{html_escape, resolve_to, send_email, send_inapp};
use cdx_notify::notify_settings::NotifySettings;
use cdx_watch::track::CheckResult;
use serde_json::{json, Value};

use crate::watch::{Change, NovaListina, APP_ID};

/// Identifies this plugin so the bell's deep link resolves to the daemon's
/// `/apps/<applicationId>` route - the only route serving a plugin app. A
/// hand-written `/sbirka-listin` matches no route and lands on a 404. The
/// component half comes from `watch::APP_ID`, already the single source for
/// this component's id.
const PLUGIN_NAME: &str = "sbirka-listin";
/// The bell's notification-type tag.
const NOTIFICATION_TYPE: &str = "sbirka-listin-changed";
const EYEBROW: &str = "Hlídač sbírky listin";
const FOOTER: &str = "Otevřete aplikaci Sbírka listin pro detail.";

/// Human-readable lines describing one detected change.
pub fn describe_change(change: &Change) -> Vec<String> {
    let mut lines = Vec::new();
    for l in &change.nove_listiny {
        lines.push(describe_nova(l));
    }
    for cislo in &change.zmizele_listiny {
        lines.push(format!("zmizelá listina: {cislo}"));
    }
    lines
}

fn describe_nova(l: &NovaListina) -> String {
    match &l.zalozeno {
        Some(z) => format!("nová listina: {} ({}, založeno {z})", l.typ, l.cislo),
        None => format!("nová listina: {} ({})", l.typ, l.cislo),
    }
}

fn changed(results: &[CheckResult<Change>]) -> Vec<&CheckResult<Change>> {
    results.iter().filter(|r| r.has_change()).collect()
}

/// The notification subject line for an already-filtered changed set.
fn change_subject(changed: &[&CheckResult<Change>]) -> String {
    if changed.len() == 1 {
        format!("Sbírka listin: nové listiny u {}", changed[0].display_name)
    } else {
        format!("Sbírka listin: nové listiny u {} subjektů", changed.len())
    }
}

/// `(subject, body_text, body_html)` for subjects with newly filed documents.
fn change_summary(changed: &[&CheckResult<Change>]) -> (String, String, String) {
    let subject = change_subject(changed);

    let mut text_lines: Vec<String> = vec![
        "Hlídač sbírky listin zaznamenal nové listiny:".to_string(),
        String::new(),
    ];
    let mut html_blocks = String::new();

    for r in changed {
        let detail = r.change.as_ref().map(describe_change).unwrap_or_default();
        text_lines.push(format!("{} (IČO {}):", r.display_name, r.slug));
        for line in &detail {
            text_lines.push(format!("  {line}"));
        }
        text_lines.push(String::new());

        let items: String = detail
            .iter()
            .map(|line| format!("<li style=\"margin:2px 0\">{}</li>", html_escape(line)))
            .collect();
        html_blocks.push_str(&format!(
            "<h3 style=\"margin:18px 0 6px;font:600 15px sans-serif;color:#0a0a0a\">{} \
             <span style=\"font-weight:400;color:#777\">(IČO {})</span></h3>\
             <ul style=\"margin:0;padding-left:18px;font:14px/1.5 sans-serif;color:#333\">{}</ul>",
            html_escape(&r.display_name),
            html_escape(&r.slug),
            items
        ));
    }
    text_lines.push(FOOTER.to_string());

    let body_html = email_shell(
        EYEBROW,
        "Nové listiny u sledovaných subjektů",
        &html_blocks,
        FOOTER,
    );
    (subject, text_lines.join("\n"), body_html)
}

/// Send one summary e-mail for the already-filtered changed set. Pass `to` to
/// override the recipient (default: the signed-in user).
fn send_email_summary(
    changed: &[&CheckResult<Change>],
    to: Option<Vec<String>>,
) -> cdx_notify::Result<()> {
    let (subject, text, html) = change_summary(changed);
    send_email(&subject, &text, Some(&html), to.as_deref(), "sbirka-listin")
}

/// Route a check run's results to e-mail / in-app per the stored notify
/// settings, returning a compact summary of what was dispatched (for the check
/// command's JSON output). Best-effort: a transport failure is reported, never
/// fatal - a check must record its state regardless.
pub fn dispatch(results: &[CheckResult<Change>]) -> Value {
    let changed = changed(results);
    if changed.is_empty() {
        return json!({ "zmen": 0 });
    }
    let cfg = NotifySettings::new(APP_ID).get_notify();
    let email = if cfg.email {
        match send_email_summary(&changed, resolve_to(&cfg.recipients)) {
            Ok(()) => format!("odesláno ({})", changed.len()),
            Err(e) => format!("chyba: {e}"),
        }
    } else {
        "vypnuto".to_string()
    };
    // The bell needs only the subject line - never build the full e-mail body.
    let in_app = if cfg.in_app {
        // Resolution is best-effort - an unresolvable link is omitted rather
        // than sent broken.
        let link = resolve_app_link(PLUGIN_NAME, APP_ID);
        if send_inapp(&change_subject(&changed), NOTIFICATION_TYPE, None, link.as_deref()) {
            "odesláno"
        } else {
            "selhalo"
        }
    } else {
        "vypnuto"
    };
    json!({ "zmen": changed.len(), "email": email, "inApp": in_app })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(nove: &[(&str, &str, &str)]) -> Change {
        Change {
            detected_on: "2026-07-20T07:00:00Z".into(),
            nove_listiny: nove
                .iter()
                .map(|(c, t, z)| NovaListina {
                    cislo: c.to_string(),
                    typ: t.to_string(),
                    zalozeno: Some(z.to_string()),
                })
                .collect(),
            zmizele_listiny: vec![],
            confirmed_on: None,
        }
    }

    fn result(name: &str, ico: &str, change: Option<Change>) -> CheckResult<Change> {
        CheckResult {
            slug: ico.to_string(),
            display_name: name.to_string(),
            ok: true,
            change,
            error: None,
        }
    }

    #[test]
    fn describe_change_lists_new_then_missing_listiny() {
        let mut ch = change(&[("B 8573/SL95/MSPH", "účetní závěrka [2024]", "1.5.2025")]);
        ch.zmizele_listiny.push("B 8573/SL1/MSPH".into());
        let lines = describe_change(&ch);
        assert_eq!(
            lines[0],
            "nová listina: účetní závěrka [2024] (B 8573/SL95/MSPH, založeno 1.5.2025)"
        );
        assert_eq!(lines[1], "zmizelá listina: B 8573/SL1/MSPH");
    }

    #[test]
    fn subject_names_the_only_subject_but_counts_when_several() {
        let a = result(
            "Alza.cz a.s.",
            "27082440",
            Some(change(&[("B 8573/SL95/MSPH", "stanovy", "1.1.2026")])),
        );
        assert_eq!(
            change_summary(&[&a]).0,
            "Sbírka listin: nové listiny u Alza.cz a.s."
        );

        let b = result("B", "2", Some(change(&[("c", "t", "z")])));
        assert_eq!(
            change_summary(&[&a, &b]).0,
            "Sbírka listin: nové listiny u 2 subjektů"
        );
    }

    #[test]
    fn body_carries_ico_and_escapes_html() {
        let r = result(
            "Firma & spol.",
            "27082440",
            Some(change(&[(
                "B 8573/SL95/MSPH",
                "účetní závěrka [2024]",
                "1.5.2025",
            )])),
        );
        let (_, text, html) = change_summary(&[&r]);
        assert!(text.contains("Firma & spol. (IČO 27082440):"));
        assert!(text.contains(
            "  nová listina: účetní závěrka [2024] (B 8573/SL95/MSPH, založeno 1.5.2025)"
        ));
        assert!(text.contains(FOOTER));
        // The plain-text name keeps the ampersand; the HTML variant escapes it.
        assert!(html.contains("Firma &amp; spol."));
        assert!(html.contains(EYEBROW));
    }

    #[test]
    fn quiet_run_reports_zero_and_touches_nothing() {
        // No change → the early return must not read settings or hit the network.
        assert_eq!(dispatch(&[result("A", "1", None)]), json!({ "zmen": 0 }));
        assert!(changed(&[result("A", "1", None)]).is_empty());
    }
}

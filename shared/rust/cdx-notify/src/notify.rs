//! Daemon notification transport shared by every watcher/component plugin -
//! e-mail via the plugin e-mail API and in-app via the notification bell.
//!
//! Auth is the VM/daemon token (`~/.cdx/env/secret`), read via
//! [`cdx_common::daemon_rest::daemon_bearer`]; the daemon URL comes from
//! `CODEXIS_PUBLIC_DAEMON_URL` (via [`cdx_common::config::read_env`]).

use std::time::Duration;

use cdx_common::config::{read_env, CODEXIS_PUBLIC_DAEMON_URL_ENV};
use cdx_common::daemon_rest::daemon_bearer;
use serde_json::json;
use uuid::Uuid;

use crate::error::{NotifyError, Result};

const EMAIL_ENDPOINT: &str = "/rest/v1/plugin/email/send";

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(30))
        .build()
}

/// GraphQL URL for the daemon (`CODEXIS_PUBLIC_DAEMON_URL`, which is the
/// `…/graphql` endpoint).
fn daemon_url() -> Option<String> {
    read_env(CODEXIS_PUBLIC_DAEMON_URL_ENV)
}

/// REST base derived from the GraphQL URL by stripping a `/graphql` suffix.
fn rest_base(daemon_url: &str) -> String {
    let base = daemon_url.trim_end_matches('/');
    base.strip_suffix("/graphql")
        .unwrap_or(base)
        .trim_end_matches('/')
        .to_string()
}

// ── GraphQL ───────────────────────────────────────────────────────────────────

/// Run a query against the daemon's GraphQL endpoint and return its `data`
/// object. Best-effort: missing config or any transport/parse error yields
/// `None`, so callers can treat "no daemon" and "bad answer" alike.
pub(crate) fn graphql(query: &str) -> Option<serde_json::Value> {
    let url = daemon_url()?;
    let auth = daemon_bearer()?;
    let resp = agent()
        .post(&url)
        .set("Authorization", &auth)
        .set("Content-Type", "application/json")
        .send_json(json!({ "query": query }))
        .ok()?;
    let parsed: serde_json::Value = resp.into_json().ok()?;
    parsed.get("data").cloned()
}

// ── signed-in user resolution ─────────────────────────────────────────────────

/// Signed-in user's e-mail from the daemon (GraphQL `me`), or `None`.
/// Best-effort: any missing config or transport error yields `None`.
pub fn query_me_email() -> Option<String> {
    let email = graphql("query { me { email } }")?
        .get("me")
        .and_then(|m| m.get("email"))
        .and_then(|e| e.as_str())?
        .to_string();
    if email.is_empty() {
        None
    } else {
        Some(email)
    }
}

fn non_empty(recipients: &[String]) -> Vec<String> {
    recipients
        .iter()
        .map(|r| r.trim())
        .filter(|r| !r.is_empty())
        .map(String::from)
        .collect()
}

/// Compose the e-mail `to` list: the signed-in user is always included when
/// there are extra recipients.
///
/// - no extras → `None` (daemon delivers to the authenticated user)
/// - extras    → `[signed_in_user, *extras]` with any extra matching the
///   signed-in user (case-insensitive) dropped, or just the extras if `me`
///   can't be resolved.
pub fn resolve_to(recipients: &[String]) -> Option<Vec<String>> {
    let extras = non_empty(recipients);
    if extras.is_empty() {
        return None;
    }
    let Some(me) = query_me_email() else {
        return Some(extras);
    };
    let me_key = me.trim().to_lowercase();
    let mut out = vec![me.clone()];
    out.extend(
        extras
            .into_iter()
            .filter(|e| e.trim().to_lowercase() != me_key),
    );
    Some(out)
}

/// Extra recipients with the signed-in user's own address removed (they always
/// receive the e-mail, so listing themselves is redundant). Case-insensitive.
/// When `me` can't be resolved, the list is returned unchanged.
pub fn without_self(recipients: &[String]) -> Vec<String> {
    let items = non_empty(recipients);
    if items.is_empty() {
        return items;
    }
    let Some(me) = query_me_email() else {
        return items;
    };
    let me_key = me.trim().to_lowercase();
    items
        .into_iter()
        .filter(|r| r.trim().to_lowercase() != me_key)
        .collect()
}

/// Extract the `recipients` array from a save-notify-settings request body and
/// run it through [`without_self`]. Returns `None` when the key is absent, so
/// the "omit `recipients` to leave the stored list untouched" rule lives in one
/// place; non-string array entries are skipped.
pub fn recipients_from_body(body: &serde_json::Value) -> Option<Vec<String>> {
    body.get("recipients").and_then(|v| v.as_array()).map(|arr| {
        let raw: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
        without_self(&raw)
    })
}

/// Minimal HTML escaping equivalent to Python's `html.escape(s)` (quote=True):
/// escapes `& < > " '`.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

// ── transport ─────────────────────────────────────────────────────────────────

pub fn send_email(
    subject: &str,
    body_text: &str,
    body_html: Option<&str>,
    to: Option<&[String]>,
    boundary_tag: &str,
) -> Result<()> {
    let (Some(url), Some(auth)) = (daemon_url(), daemon_bearer()) else {
        return Err(NotifyError::Notify(
            "E-mail nelze odeslat - chybí CODEXIS_PUBLIC_DAEMON_URL nebo přihlašovací \
             token (~/.cdx/env/secret). Spouštějte přes automatizaci v CODEXIS."
                .into(),
        ));
    };

    let mut payload = json!({ "subject": subject, "bodyText": body_text });
    if let Some(html) = body_html {
        payload["bodyHtml"] = json!(html);
    }
    if let Some(to) = to {
        if !to.is_empty() {
            payload["to"] = json!(to);
        }
    }

    let boundary = format!("----{boundary_tag}-{}", Uuid::new_v4().simple());
    let body = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"payload\"\r\n\
         Content-Type: application/json; charset=utf-8\r\n\r\n\
         {payload}\r\n\
         --{boundary}--\r\n",
        boundary = boundary,
        payload = serde_json::to_string(&payload)?,
    );

    let url = format!("{}{}", rest_base(&url), EMAIL_ENDPOINT);
    match agent()
        .post(&url)
        .set("Authorization", &auth)
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .send_bytes(body.as_bytes())
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(code, resp)) => {
            let detail: String = resp
                .into_string()
                .unwrap_or_default()
                .chars()
                .take(300)
                .collect();
            Err(NotifyError::Notify(format!(
                "E-mail se nepodařilo odeslat (HTTP {code}): {detail}"
            )))
        }
        Err(ureq::Error::Transport(t)) => Err(NotifyError::Notify(format!(
            "E-mail se nepodařilo odeslat: {t}"
        ))),
    }
}

/// The GraphQL `createNotification` request body. `action` and `link` are
/// omitted (not null) when unset, mirroring the Python `_send_inapp`.
fn build_inapp_body(
    message: &str,
    notification_type: &str,
    action: Option<&str>,
    link: Option<&str>,
) -> serde_json::Value {
    let mut input = json!({ "message": message, "type": notification_type });
    if let Some(action) = action {
        input["action"] = json!(action);
    }
    if let Some(link) = link {
        input["link"] = json!(link);
    }
    json!({
        "query": "mutation Create($input: CreateNotificationInput!) \
                  { createNotification(input: $input) { id } }",
        "variables": { "input": input },
    })
}

/// Create an in-app (notification-center bell) notification via the daemon's
/// GraphQL `createNotification`. Best-effort: returns `false` on missing config
/// or a transport error, never raises (a notification must not fail the check).
pub fn send_inapp(
    message: &str,
    notification_type: &str,
    action: Option<&str>,
    link: Option<&str>,
) -> bool {
    let (Some(url), Some(auth)) = (daemon_url(), daemon_bearer()) else {
        return false;
    };
    agent()
        .post(&url)
        .set("Authorization", &auth)
        .set("Content-Type", "application/json; charset=utf-8")
        .send_json(build_inapp_body(message, notification_type, action, link))
        .is_ok()
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rest_base_strips_graphql_suffix() {
        assert_eq!(rest_base("https://d.example/graphql"), "https://d.example");
        assert_eq!(rest_base("https://d.example/graphql/"), "https://d.example");
        assert_eq!(rest_base("https://d.example"), "https://d.example");
    }

    #[test]
    fn html_escape_covers_markup_chars() {
        assert_eq!(html_escape("a<b>&\"'"), "a&lt;b&gt;&amp;&quot;&#x27;");
    }

    #[test]
    fn non_empty_trims_and_drops_blanks() {
        assert_eq!(
            non_empty(&["  a@x.com ".into(), "".into(), "  ".into()]),
            vec!["a@x.com".to_string()]
        );
    }

    #[test]
    fn inapp_body_includes_action_and_link_when_set() {
        let body = build_inapp_body("msg", "cadastre-changed", Some("open"), Some("/katastr"));
        let input = &body["variables"]["input"];
        assert_eq!(input["message"], json!("msg"));
        assert_eq!(input["type"], json!("cadastre-changed"));
        assert_eq!(input["action"], json!("open"));
        assert_eq!(input["link"], json!("/katastr"));
    }

    #[test]
    fn inapp_body_omits_action_and_link_when_none() {
        let body = build_inapp_body("msg", "t", None, None);
        let input = body["variables"]["input"].as_object().unwrap();
        assert!(!input.contains_key("action"));
        assert!(!input.contains_key("link"));
    }
}

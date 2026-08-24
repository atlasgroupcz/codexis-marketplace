//! Daemon notification transport shared by every watcher/component plugin -
//! e-mail via the plugin e-mail API and in-app via the notification bell.
//!
//! Auth is the VM/daemon token (`~/.cdx/env/secret`), read via
//! [`cdx_common::daemon_rest::daemon_bearer`]; the daemon URL comes from
//! `CODEXIS_PUBLIC_DAEMON_URL` (via [`cdx_common::config::read_env`]).

use std::sync::OnceLock;
use std::time::Duration;

use cdx_common::config::{read_env, CODEXIS_PUBLIC_DAEMON_URL_ENV};
use cdx_common::daemon_rest::daemon_bearer;
use cdx_common::http_client::{outcome, Outcome};
use serde_json::json;
use uuid::Uuid;

use crate::error::{NotifyError, Result};

const EMAIL_ENDPOINT: &str = "/rest/v1/plugin/email/send";

const USER_AGENT_VALUE: &str = concat!("cdx-notify/", env!("CARGO_PKG_VERSION"));

/// One Agent per process. A notifying run makes three to four calls to the same
/// daemon host (resolve the user, send the mail, resolve the app link, ring the
/// bell); one agent means one TCP+TLS handshake instead of one per call.
fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        cdx_common::http_client::agent(
            USER_AGENT_VALUE,
            Duration::from_secs(5),
            Duration::from_secs(30),
        )
    })
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
    let sent = agent()
        .post(&url)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json")
        .send_json(json!({ "query": query }));
    let Outcome::Ok(mut resp) = outcome(sent) else {
        return None;
    };
    let parsed: serde_json::Value = resp.body_mut().read_json().ok()?;
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
    body.get("recipients")
        .and_then(|v| v.as_array())
        .map(|arr| {
            let raw: Vec<String> = arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
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
    // Sending mail is not idempotent, so this classifies one attempt rather than
    // going through `call_with_retry` - a retry could deliver the mail twice.
    let sent = agent()
        .post(&url)
        .header("Authorization", &auth)
        .header(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .send(body.as_bytes());

    match outcome(sent) {
        Outcome::Ok(_) => Ok(()),
        Outcome::Status(code, detail) => Err(NotifyError::Notify(format!(
            "E-mail se nepodařilo odeslat (HTTP {code}): {}",
            detail.chars().take(300).collect::<String>()
        ))),
        Outcome::Transport(t) => Err(NotifyError::Notify(format!(
            "E-mail se nepodařilo odeslat: {t}"
        ))),
    }
}

/// The GraphQL `createNotification` request body. `link` is omitted (not null)
/// when unset, mirroring the Python `_send_inapp`.
///
/// There is deliberately no call-to-action field. `CreateNotificationInput`
/// defines `message`, `content`, `link`, `type` and `vars`; the daemon rejects
/// the whole mutation over an input field it does not know, and it answers that
/// rejection with HTTP 200, so the sender cannot see it from the status.
fn build_inapp_body(
    message: &str,
    notification_type: &str,
    link: Option<&str>,
) -> serde_json::Value {
    let mut input = json!({ "message": message, "type": notification_type });
    if let Some(link) = link {
        input["link"] = json!(link);
    }
    json!({
        "query": "mutation Create($input: CreateNotificationInput!) \
                  { createNotification(input: $input) { id } }",
        "variables": { "input": input },
    })
}

/// True when the daemon accepted the mutation.
///
/// The status alone does not say. A rejected mutation comes back as HTTP 200
/// with the reason in `errors`, so a sender that reads only the status reports
/// every rejection as a delivery - that is how a dropped input field went
/// unnoticed for a month of scheduled runs in the Python plugins. A body that
/// is not JSON at all, e.g. a proxy's error page, is not a delivery either.
fn accepted_body(body: &str) -> bool {
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    let Some(object) = payload.as_object() else {
        return false;
    };
    match object.get("errors") {
        None | Some(serde_json::Value::Null) => true,
        Some(serde_json::Value::Array(errors)) => errors.is_empty(),
        Some(_) => false,
    }
}

/// Create an in-app (notification-center bell) notification via the daemon's
/// GraphQL `createNotification`. Best-effort: returns `false` on missing config
/// or a transport error, never raises (a notification must not fail the check).
pub fn send_inapp(message: &str, notification_type: &str, link: Option<&str>) -> bool {
    let (Some(url), Some(auth)) = (daemon_url(), daemon_bearer()) else {
        return false;
    };
    // Creating a notification is not idempotent either, so one attempt only.
    let sent = agent()
        .post(&url)
        .header("Authorization", &auth)
        .header("Content-Type", "application/json; charset=utf-8")
        .send_json(build_inapp_body(message, notification_type, link));
    match outcome(sent) {
        Outcome::Ok(mut response) => response
            .body_mut()
            .read_to_string()
            .map(|body| accepted_body(&body))
            .unwrap_or(false),
        _ => false,
    }
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
    fn inapp_body_includes_link_when_set() {
        let body = build_inapp_body("msg", "cadastre-changed", Some("/katastr"));
        let input = &body["variables"]["input"];
        assert_eq!(input["message"], json!("msg"));
        assert_eq!(input["type"], json!("cadastre-changed"));
        assert_eq!(input["link"], json!("/katastr"));
    }

    #[test]
    fn inapp_body_omits_link_when_none() {
        let body = build_inapp_body("msg", "t", None);
        let input = body["variables"]["input"].as_object().unwrap();
        assert!(!input.contains_key("link"));
    }

    #[test]
    fn inapp_body_never_carries_an_action_field() {
        // `CreateNotificationInput` does not define one, and the daemon rejects
        // the whole mutation over an input field it does not know.
        let body = build_inapp_body("msg", "t", Some("/katastr"));
        let input = body["variables"]["input"].as_object().unwrap();
        assert!(!input.contains_key("action"));
    }

    #[test]
    fn a_response_carrying_errors_is_not_a_delivery() {
        assert!(!accepted_body(
            r#"{"errors":[{"message":"Field 'action' in type 'Notification' is undefined"}]}"#
        ));
    }

    #[test]
    fn a_body_that_is_not_json_is_not_a_delivery() {
        assert!(!accepted_body("<html>502 Bad Gateway</html>"));
    }

    #[test]
    fn a_json_array_is_not_a_delivery() {
        assert!(!accepted_body(r#"["not","an","object"]"#));
    }

    #[test]
    fn a_data_response_is_a_delivery() {
        assert!(accepted_body(
            r#"{"data":{"createNotification":{"id":"n1"}}}"#
        ));
    }

    #[test]
    fn an_empty_errors_array_is_a_delivery() {
        assert!(accepted_body(
            r#"{"data":{"createNotification":{"id":"n1"}},"errors":[]}"#
        ));
    }
}

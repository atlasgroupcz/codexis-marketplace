use std::fmt;
use std::process::{Command, Output, Stdio};

use crate::config::DaemonConfig;

/// Publishes the `sources` array from a tool response body to the CDX daemon.
///
/// A `Null` or empty array is a no-op. Anything else is POSTed to the daemon's
/// sources endpoint via `curl`.
pub fn publish_from_body(
    sources: &serde_json::Value,
    daemon: &DaemonConfig,
) -> Result<(), SourcesError> {
    let entries = match sources {
        serde_json::Value::Null => return Ok(()),
        serde_json::Value::Array(items) => items,
        other => {
            return Err(SourcesError::Parse(format!(
                "expected JSON array in `sources`, got {}",
                value_kind(other)
            )));
        }
    };
    if entries.is_empty() {
        return Ok(());
    }
    let body = build_publish_body(entries)?;
    post_to_daemon(&daemon.sources_url(), daemon.auth_header(), &body)
}

/// Publishes a slice of already-built source entries to the CDX daemon.
/// An empty slice is a no-op.
pub fn publish_entries(
    entries: &[serde_json::Value],
    daemon: &DaemonConfig,
) -> Result<(), SourcesError> {
    if entries.is_empty() {
        return Ok(());
    }
    let body = build_publish_body(entries)?;
    post_to_daemon(&daemon.sources_url(), daemon.auth_header(), &body)
}

/// Entries are forwarded to the daemon verbatim — every upstream field, including
/// ones this crate does not know about (e.g. `previewUrl`), must survive publish.
fn build_publish_body(entries: &[serde_json::Value]) -> Result<String, SourcesError> {
    serde_json::to_string(&serde_json::json!({ "sources": entries }))
        .map_err(|e| SourcesError::Parse(e.to_string()))
}

/// Best-effort publish of source entries: a no-op when empty or outside a
/// chat-VM context; logs to stderr on failure but never propagates. Used by
/// plugins that build their own source entries. `plugin_name` prefixes the log.
pub fn try_publish(plugin_name: &str, entries: &[serde_json::Value]) {
    if entries.is_empty() {
        return;
    }
    let Some(daemon) = DaemonConfig::load() else {
        return;
    };
    if let Err(err) = publish_entries(entries, &daemon) {
        eprintln!("{plugin_name}: failed to attach sources to chat: {err}");
    }
}

fn post_to_daemon(url: &str, auth_header: &str, body: &str) -> Result<(), SourcesError> {
    let output = Command::new("curl")
        .args([
            "-sS", "--fail-with-body", "-H", auth_header, "-X", "POST",
            "-H", "Content-Type: application/json", url, "-d", body,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| SourcesError::Spawn(e.to_string()))?;
    match output.status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(SourcesError::Http { code, detail: response_detail(&output) }),
        None => Err(SourcesError::Spawn("curl terminated without status".into())),
    }
}

fn response_detail(output: &Output) -> String {
    let body = String::from_utf8_lossy(&output.stdout);
    let body = body.trim();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    match (body.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{body} ({stderr})"),
        (false, true) => body.to_string(),
        (true, false) => stderr.to_string(),
        (true, true) => "no response body".to_string(),
    }
}

fn value_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[derive(Debug)]
pub enum SourcesError {
    Parse(String),
    Spawn(String),
    Http { code: i32, detail: String },
}

impl fmt::Display for SourcesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "could not parse envelope `sources`: {msg}"),
            Self::Spawn(msg) => write!(f, "could not POST sources to daemon: {msg}"),
            Self::Http { code, detail } => {
                write!(f, "daemon rejected sources (curl exit {code}): {detail}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_body_keeps_unknown_entry_fields_verbatim() {
        let entries = vec![
            serde_json::json!({
                "__kind": "url",
                "url": "https://app.codexis.cz/doc/CR10#paragraf3",
                "title": "OZ",
                "previewUrl": "https://app.codexis.cz/document/CR10/text#paragraf3"
            }),
            serde_json::json!({ "__kind": "url", "url": "https://example.com/doc" }),
        ];

        let body = build_publish_body(&entries).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

        assert_eq!(
            parsed["sources"][0]["previewUrl"],
            serde_json::json!("https://app.codexis.cz/document/CR10/text#paragraf3")
        );
        assert!(parsed["sources"][1].get("previewUrl").is_none());
        assert_eq!(parsed["sources"], serde_json::json!(entries));
    }
}

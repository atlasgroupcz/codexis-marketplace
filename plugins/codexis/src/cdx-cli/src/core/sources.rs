use std::collections::HashMap;
use std::fmt;
use std::process::{Command, Output, Stdio};

use crate::core::config::DaemonConfig;
use crate::core::error::CliError;

pub(crate) const SOURCES_HEADER_NAME: &str = "x-cdx-sources";

/// Reads the `X-Cdx-Sources` header from an upstream response and attaches it to the
/// active cdx-daemon tool-call. The contract is strict: if the header is missing or
/// empty, no daemon call is made. If it is present but malformed, an error is returned
/// (the caller — `try_publish_sources` in `http.rs` — logs and continues so the agent's
/// primary output is never blocked).
pub(crate) fn publish_from_headers(
    headers: &HashMap<String, String>,
    daemon: &DaemonConfig,
) -> Result<(), SourcesError> {
    let Some(raw) = headers.get(SOURCES_HEADER_NAME) else {
        return Ok(());
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let decoded = url_decode(trimmed).map_err(SourcesError::Decode)?;
    let parsed: serde_json::Value =
        serde_json::from_str(&decoded).map_err(|e| SourcesError::Parse(e.to_string()))?;
    let entries = match parsed {
        serde_json::Value::Array(items) => items,
        other => {
            return Err(SourcesError::Parse(format!(
                "expected JSON array, got {}",
                value_kind(&other)
            )));
        }
    };
    if entries.is_empty() {
        return Ok(());
    }
    let body = serde_json::to_string(&serde_json::json!({ "sources": entries }))
        .map_err(|e| SourcesError::Parse(e.to_string()))?;
    post_to_daemon(&daemon.sources_url(), &daemon.auth_header, &body)
}

fn post_to_daemon(url: &str, auth_header: &str, body: &str) -> Result<(), SourcesError> {
    let output = Command::new("curl")
        .args([
            "-sS",
            "--fail-with-body",
            "-H",
            auth_header,
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            url,
            "-d",
            body,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| SourcesError::Spawn(e.to_string()))?;
    match output.status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(SourcesError::Http {
            code,
            detail: response_detail(&output),
        }),
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

/// Minimal URL-decode for header values produced by `URLEncoder.encode(json, UTF_8)`
/// on the Java side. Handles `+` → space and `%XX` percent escapes; otherwise returns
/// the bytes verbatim. We do this in-process instead of pulling a crate dependency to
/// keep the cdx-cli binary small.
fn url_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' => {
                if i + 2 >= bytes.len() {
                    return Err(format!("truncated percent escape at byte {i}"));
                }
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3])
                    .map_err(|_| format!("invalid percent escape at byte {i}"))?;
                let byte = u8::from_str_radix(hex, 16)
                    .map_err(|_| format!("invalid percent escape at byte {i}: %{hex}"))?;
                out.push(byte);
                i += 3;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|e| format!("decoded bytes are not valid UTF-8: {e}"))
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
pub(crate) enum SourcesError {
    Decode(String),
    Parse(String),
    Spawn(String),
    Http { code: i32, detail: String },
}

impl fmt::Display for SourcesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(msg) => write!(f, "could not URL-decode X-Cdx-Sources: {msg}"),
            Self::Parse(msg) => write!(f, "could not parse X-Cdx-Sources JSON: {msg}"),
            Self::Spawn(msg) => write!(f, "could not POST sources to daemon: {msg}"),
            Self::Http { code, detail } => {
                write!(f, "daemon rejected sources (curl exit {code}): {detail}")
            }
        }
    }
}

impl From<SourcesError> for CliError {
    fn from(value: SourcesError) -> Self {
        CliError::Io {
            context: value.to_string(),
            source: std::io::Error::other("sources publish failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(value: &str) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert(SOURCES_HEADER_NAME.to_string(), value.to_string());
        h
    }

    #[test]
    fn missing_header_is_silent_noop() {
        let headers = HashMap::new();
        // No daemon needed: function returns Ok before any daemon access.
        assert!(publish_from_headers(&headers, &fake_daemon()).is_ok());
    }

    #[test]
    fn empty_header_is_silent_noop() {
        let headers = header("");
        assert!(publish_from_headers(&headers, &fake_daemon()).is_ok());
    }

    #[test]
    fn empty_array_is_silent_noop() {
        let headers = header("%5B%5D");
        assert!(publish_from_headers(&headers, &fake_daemon()).is_ok());
    }

    #[test]
    fn malformed_json_returns_parse_error() {
        let headers = header("not-json");
        let err = publish_from_headers(&headers, &fake_daemon()).unwrap_err();
        assert!(matches!(err, SourcesError::Parse(_)), "got {err}");
    }

    #[test]
    fn non_array_returns_parse_error() {
        // URL-encoded `{}`
        let headers = header("%7B%7D");
        let err = publish_from_headers(&headers, &fake_daemon()).unwrap_err();
        assert!(matches!(err, SourcesError::Parse(_)), "got {err}");
    }

    #[test]
    fn url_decode_handles_unicode_via_percent_escapes() {
        let decoded = url_decode("%C5%BDlu%C5%A5ou%C4%8Dk%C3%BD%20k%C5%AF%C5%88").unwrap();
        assert_eq!(decoded, "Žluťoučký kůň");
    }

    #[test]
    fn url_decode_converts_plus_to_space() {
        assert_eq!(url_decode("a+b+c").unwrap(), "a b c");
    }

    #[test]
    fn url_decode_rejects_truncated_escape() {
        assert!(url_decode("a%2").is_err());
    }

    fn fake_daemon() -> DaemonConfig {
        DaemonConfig::test_only_new(
            "https://daemon.example/graphql".to_string(),
            "Authorization: Bearer test".to_string(),
            "chat-1".to_string(),
            "tool-1".to_string(),
        )
    }
}

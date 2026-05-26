use std::fmt;
use std::process::{Command, Output, Stdio};

use crate::core::config::DaemonConfig;
use crate::core::error::CliError;

pub(crate) fn publish_from_body(
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
    Parse(String),
    Spawn(String),
    Http { code: i32, detail: String },
}

impl fmt::Display for SourcesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "could not parse v2 envelope `sources`: {msg}"),
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

    #[test]
    fn null_sources_is_silent_noop() {
        assert!(publish_from_body(&serde_json::Value::Null, &fake_daemon()).is_ok());
    }

    #[test]
    fn empty_array_is_silent_noop() {
        let sources = serde_json::json!([]);
        assert!(publish_from_body(&sources, &fake_daemon()).is_ok());
    }

    #[test]
    fn non_array_returns_parse_error() {
        let sources = serde_json::json!({});
        let err = publish_from_body(&sources, &fake_daemon()).unwrap_err();
        assert!(matches!(err, SourcesError::Parse(_)), "got {err}");
    }

    #[test]
    fn string_returns_parse_error() {
        let sources = serde_json::json!("not-an-array");
        let err = publish_from_body(&sources, &fake_daemon()).unwrap_err();
        assert!(matches!(err, SourcesError::Parse(_)), "got {err}");
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

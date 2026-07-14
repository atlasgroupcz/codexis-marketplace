//! Curl-based HTTP plumbing shared by the CODEXIS search plugins.
//!
//! These are the self-contained utilities — running curl, handling its exit
//! code, splitting the response envelope, encoding query values, redacting
//! secrets for dry-run output and shell-escaping. They were byte-identical
//! across the search plugins. The per-plugin parts (URL schemes, `cdx://`
//! resource resolvers, source-code → API-path maps) stay in each plugin.

use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

use crate::error::CliError;

/// Runs `curl` with the given arguments and captures its output.
pub fn execute_curl(curl_args: &[String]) -> Result<Output, CliError> {
    Command::new("curl")
        .args(curl_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(CliError::CurlSpawn)
}

/// Maps a curl `Output` to a result, printing the body via `print_body`.
/// A non-zero exit still prints any body, then reports the failure.
pub fn handle_curl_output<F>(output: &Output, print_body: F) -> Result<(), CliError>
where
    F: Fn(&[u8]) -> Result<(), CliError>,
{
    match output.status.code() {
        Some(0) => print_body(&output.stdout),
        Some(code) => {
            if !output.stdout.is_empty() {
                print_body(&output.stdout)?;
            }
            Err(CliError::RequestFailed { code })
        }
        None => Err(CliError::CommandTerminated { command: "curl" }),
    }
}

/// Writes rendered output to stdout, appending a trailing newline if missing.
pub fn write_stdout(rendered: &str) -> Result<(), CliError> {
    if rendered.is_empty() {
        return Ok(());
    }

    let mut stdout = io::stdout().lock();
    stdout
        .write_all(rendered.as_bytes())
        .map_err(|source| CliError::Io {
            context: "failed to write response to stdout".to_string(),
            source,
        })?;
    if !rendered.ends_with('\n') {
        stdout.write_all(b"\n").map_err(|source| CliError::Io {
            context: "failed to write trailing newline to stdout".to_string(),
            source,
        })?;
    }
    Ok(())
}

/// Splits a daemon response envelope into `(content, sources)`. A JSON object
/// with a `content` key is unwrapped (string content loses its JSON quotes);
/// anything else is passed through untouched with no sources.
pub fn split_envelope_str(body: &str) -> (String, Option<serde_json::Value>) {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json::Value::Object(mut map)) if map.contains_key("content") => {
            let sources = map.remove("sources");
            let content = map.remove("content").unwrap_or(serde_json::Value::Null);
            let printed = match &content {
                serde_json::Value::String(s) => s.clone(),
                other => serde_json::to_string(other).unwrap_or_default(),
            };
            (printed, sources)
        }
        _ => (body.to_string(), None),
    }
}

/// Percent-encodes non-ASCII bytes in query parameter values, preserving query
/// structure characters (`?`, `&`, `=`) and ASCII values.
pub fn encode_query_values(query: &str) -> String {
    let query = match query.strip_prefix('?') {
        Some(q) => q,
        None => return query.to_string(),
    };

    let mut result = String::with_capacity(query.len() + 16);
    result.push('?');

    for (i, pair) in query.split('&').enumerate() {
        if i > 0 {
            result.push('&');
        }
        match pair.split_once('=') {
            Some((key, value)) => {
                result.push_str(key);
                result.push('=');
                percent_encode_utf8(&mut result, value);
            }
            None => result.push_str(pair),
        }
    }

    result
}

fn percent_encode_utf8(out: &mut String, input: &str) {
    for byte in input.bytes() {
        if byte.is_ascii() {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(to_hex_digit(byte >> 4));
            out.push(to_hex_digit(byte & 0x0F));
        }
    }
}

fn to_hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + nibble - 10) as char,
        _ => unreachable!(),
    }
}

/// Redacts `Authorization:` headers in curl args, for safe dry-run output.
pub fn redact_curl_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            if is_authorization_header(arg) {
                redact_authorization_header(arg)
            } else {
                arg.clone()
            }
        })
        .collect()
}

fn is_authorization_header(value: &str) -> bool {
    value
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("authorization:")
}

fn redact_authorization_header(header: &str) -> String {
    let value = header
        .split_once(':')
        .map(|(_, value)| value.trim())
        .unwrap_or_default();

    if value.to_ascii_lowercase().starts_with("bearer ") {
        "Authorization: Bearer <redacted>".to_string()
    } else if let Some((scheme, _)) = value.split_once(' ') {
        format!("Authorization: {scheme} <redacted>")
    } else {
        "Authorization: <redacted>".to_string()
    }
}

/// Renders a program + args as a copy-pasteable shell command (for `--dry-run`).
pub fn format_command(program: &str, args: &[String]) -> String {
    let mut rendered = Vec::with_capacity(args.len() + 1);
    rendered.push(shell_escape(program));
    rendered.extend(args.iter().map(|arg| shell_escape(arg)));
    rendered.join(" ")
}

/// Quotes a value for safe use in a POSIX shell command.
pub fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    if value.chars().all(is_shell_safe_char) {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn is_shell_safe_char(ch: char) -> bool {
    matches!(
        ch,
        'a'..='z'
            | 'A'..='Z'
            | '0'..='9'
            | '-'
            | '_'
            | '.'
            | '/'
            | ':'
            | '='
            | '@'
            | '%'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_envelope_extracts_content_and_sources() {
        let (content, sources) =
            split_envelope_str(r#"{"content":"hello","sources":[{"id":1}]}"#);
        assert_eq!(content, "hello");
        assert!(sources.is_some());
    }

    #[test]
    fn split_envelope_passthrough_when_no_content_key() {
        let (content, sources) = split_envelope_str(r#"{"other":"x"}"#);
        assert_eq!(content, r#"{"other":"x"}"#);
        assert!(sources.is_none());
    }

    #[test]
    fn shell_escape_quotes_values_with_specials() {
        assert_eq!(shell_escape("plain"), "plain");
        assert_eq!(shell_escape("a b"), "'a b'");
        assert_eq!(shell_escape(""), "''");
    }

    #[test]
    fn redact_hides_authorization_bearer() {
        let args = vec!["-H".to_string(), "Authorization: Bearer secret".to_string()];
        let out = redact_curl_args(&args);
        assert_eq!(out[1], "Authorization: Bearer <redacted>");
    }

    #[test]
    fn encode_query_percent_encodes_non_ascii() {
        assert_eq!(encode_query_values("?q=%C3"), "?q=%C3");
        assert_eq!(encode_query_values("?q=caf\u{e9}"), "?q=caf%C3%A9");
    }
}

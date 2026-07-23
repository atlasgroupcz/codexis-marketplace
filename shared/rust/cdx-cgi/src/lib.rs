//! Minimal CGI runtime shared by the hosted-component handlers: query parsing,
//! body reading, and JSON/error response emission. No plugin specifics - a pure
//! capability crate, independent of the notify and watcher capabilities.

use std::env;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse<'a> {
    ok: bool,
    error: &'a str,
}

/// Write a JSON CGI response (200) with no-store caching.
pub fn emit_json<T: Serialize>(value: &T) -> ExitCode {
    let mut out = io::stdout().lock();
    let _ = write!(
        out,
        "Content-Type: application/json; charset=utf-8\r\n\
         Cache-Control: no-store\r\n\
         \r\n"
    );
    match serde_json::to_writer(&mut out, value) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

/// Write a JSON error CGI response with an explicit `Status:` line.
pub fn emit_error(status: u16, message: &str) -> ExitCode {
    let mut out = io::stdout().lock();
    let _ = write!(
        out,
        "Status: {status}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Cache-Control: no-store\r\n\
         \r\n"
    );
    let _ = serde_json::to_writer(&mut out, &ErrorResponse { ok: false, error: message });
    ExitCode::SUCCESS
}

/// Upper bound on an accepted request body. `CONTENT_LENGTH` is attacker- (or
/// bug-) controlled input; without a cap a bogus value makes the per-request
/// CGI process allocate the whole claimed size up front.
pub const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;

/// Read the request body (`CONTENT_LENGTH` bytes, at most [`MAX_BODY_BYTES`])
/// from stdin.
pub fn read_body() -> io::Result<String> {
    let content_length: usize = env::var("CONTENT_LENGTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if content_length == 0 {
        return Ok(String::new());
    }
    if content_length > MAX_BODY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("request body of {content_length} bytes exceeds the {MAX_BODY_BYTES}-byte limit"),
        ));
    }
    let mut buf = vec![0u8; content_length];
    io::stdin().read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Parse a `QUERY_STRING` into decoded `(key, value)` pairs.
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => (percent_decode(k), percent_decode(v)),
            None => (percent_decode(pair), String::new()),
        })
        .collect()
}

/// Percent- and `+`-decode a single query token.
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                    out.push((hi << 4) | lo);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + b - b'a'),
        b'A'..=b'F' => Some(10 + b - b'A'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_body_rejects_content_length_over_cap() {
        // 64 MiB + 1 - over the cap, but small enough that the pre-fix code
        // fails by hitting EOF on stdin rather than by refusing the length.
        std::env::set_var("CONTENT_LENGTH", (64 * 1024 * 1024 + 1).to_string());
        let result = read_body();
        std::env::remove_var("CONTENT_LENGTH");
        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput, "got: {err}");
    }

    #[test]
    fn percent_decode_handles_plus_and_hex() {
        assert_eq!(percent_decode("a+b%2Fc"), "a b/c");
        assert_eq!(percent_decode("plain"), "plain");
        assert_eq!(percent_decode("V-1%2F2026-701"), "V-1/2026-701");
    }

    #[test]
    fn parse_query_splits_pairs_and_decodes() {
        let got = parse_query("view=notify&uuid=V-1%2F2026&flag");
        assert_eq!(
            got,
            vec![
                ("view".to_string(), "notify".to_string()),
                ("uuid".to_string(), "V-1/2026".to_string()),
                ("flag".to_string(), String::new()),
            ]
        );
    }
}

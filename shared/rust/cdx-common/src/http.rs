//! Small reqwest helpers shared by the reqwest-based lookup plugins (ares,
//! zrsr, kataster-sk, obchodny-register-sk, …).
//!
//! Only the byte-identical, self-contained helpers live here - status
//! formatting, body reading, retry/backoff and URL joining. Each plugin keeps
//! its own `Client` struct and request logic, which genuinely diverge (single
//! vs dual base URL, plain GET vs retry loop vs retry+cache, cookies, etc.).

use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::Response;
use reqwest::StatusCode;

/// `"<code> <reason>"`, or just the code when there is no canonical reason.
pub fn format_status(status: StatusCode) -> String {
    match status.canonical_reason() {
        Some(reason) => format!("{} {}", status.as_u16(), reason),
        None => status.as_u16().to_string(),
    }
}

/// Reads a response body, returning it on success or an error carrying the
/// status, method, URL and body on failure.
pub fn read_body(resp: Response, url: &str, method: &str) -> Result<String> {
    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if status.is_success() {
        Ok(body)
    } else {
        Err(anyhow!(
            "HTTP {status} on {method} {url}\n{body}",
            status = format_status(status),
            method = method,
            url = url,
            body = body.trim()
        ))
    }
}

/// Reads an env var (trimmed of a trailing slash), falling back to `default`.
pub fn base_url_from_env(var: &str, default: &str) -> String {
    std::env::var(var)
        .unwrap_or_else(|_| default.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Joins a base URL and a path; an empty path yields the base unchanged.
pub fn join_url(base: &str, path: &str) -> String {
    if path.is_empty() {
        return base.to_string();
    }
    format!("{base}/{}", path.trim_start_matches('/'))
}

/// Whether a status is worth retrying (429 or any 5xx).
pub fn retryable_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

/// Exponential backoff: 250ms · 2^attempt.
pub fn backoff(attempt: usize) -> Duration {
    Duration::from_millis(250 * 2_u64.pow(attempt as u32))
}

/// Resolves a request path against a base URL: absolute `http(s)://` paths pass
/// through; otherwise the path is appended to the base (a leading `/` is kept).
pub fn resolve_url(base_url: &str, path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }
    if path.starts_with('/') {
        format!("{base_url}{path}")
    } else {
        format!("{base_url}/{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_formats_with_reason() {
        assert_eq!(format_status(StatusCode::NOT_FOUND), "404 Not Found");
    }

    #[test]
    fn join_and_env_base() {
        assert_eq!(join_url("https://x/api", "a/b"), "https://x/api/a/b");
        assert_eq!(join_url("https://x/api", "/a"), "https://x/api/a");
        assert_eq!(join_url("https://x/api", ""), "https://x/api");
    }

    #[test]
    fn retry_and_backoff() {
        assert!(retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(retryable_status(StatusCode::BAD_GATEWAY));
        assert!(!retryable_status(StatusCode::NOT_FOUND));
        assert_eq!(backoff(0), Duration::from_millis(250));
        assert_eq!(backoff(2), Duration::from_millis(1000));
    }
}

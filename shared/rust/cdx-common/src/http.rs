//! URL, retry and prefix-table helpers shared by the lookup plugins.
//!
//! These once lived here in a reqwest flavour. When the plugins moved to ureq
//! they each re-implemented them locally rather than porting the shared module,
//! leaving `(was cdx_common::http::…)` breadcrumbs behind and this module dead
//! behind a feature nobody enabled. This is the ureq-era replacement.
//!
//! This module holds the rules; [`crate::http_client`] holds the plumbing that
//! applies them - the configured agent and the retry loop. What stays with each
//! plugin is its own `Client` struct and the search/get flows on top: those
//! really do differ service by service, and folding them together would trade a
//! real difference for a false one.
//!
//! Nothing here needs an HTTP crate, so the module is std-only and the `http`
//! feature carries no dependencies. Statuses are plain `u16` rather than a
//! `StatusCode`, which is what keeps it that way.

use std::time::Duration;

/// Resolves a request path against a base URL. An absolute `http(s)://` path
/// passes through, so callers can mix full URLs and paths freely.
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

/// Joins a path onto a base with exactly one separator. Unlike [`resolve_url`]
/// this always treats `path` as relative; an empty path yields the base.
pub fn join_url(base: &str, path: &str) -> String {
    if path.is_empty() {
        return base.to_string();
    }
    format!("{base}/{}", path.trim_start_matches('/'))
}

/// Base URL from an environment override, falling back to `default`. The
/// trailing slash is stripped so callers can concatenate without doubling it.
pub fn base_url_from_env(var: &str, default: &str) -> String {
    std::env::var(var)
        .unwrap_or_else(|_| default.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// `"404 Not Found"`, or just `"404"` when the reason phrase is empty.
///
/// Not public: ureq 3 does not report the server's own phrase, so every caller
/// wants [`crate::http_client::status_text`], which derives it from the code.
pub(crate) fn format_status(code: u16, reason: &str) -> String {
    if reason.is_empty() {
        code.to_string()
    } else {
        format!("{code} {reason}")
    }
}

/// Whether a status means the request failed: 400 and above. The boundary lives
/// here rather than as a literal at each call site, next to the other status
/// rules.
pub fn is_failure(code: u16) -> bool {
    code >= 400
}

/// Whether a body is an HTML page rather than the JSON that was asked for.
///
/// Some services answer a block or an outage with HTTP 200 and an error page,
/// so the status alone does not catch it. Two plugins test for this and both
/// phrase the resulting error differently, which is theirs to keep - the test
/// itself is not.
pub fn looks_like_html(body: &str) -> bool {
    body.trim_start().starts_with('<')
}

/// Whether a status is worth retrying: rate limiting or a server-side fault.
pub fn retryable_status(code: u16) -> bool {
    code == 429 || (500..=599).contains(&code)
}

/// Exponential backoff for retry `attempt` (0-based): 250ms, 500ms, 1s, …
pub fn backoff(attempt: usize) -> Duration {
    Duration::from_millis(250 * 2_u64.pow(attempt as u32))
}

/// Resolves a display id to its API path by matching the id's leading prefix.
///
/// The table stays with the plugin - each has its own document kinds - while the
/// lookup itself was identical in nine of them.
pub fn lookup_display_id_prefix<'a>(
    id: &str,
    prefixes: &[(&str, &'a str)],
) -> Result<&'a str, String> {
    for &(prefix, api_path) in prefixes {
        if id.starts_with(prefix) {
            return Ok(api_path);
        }
    }
    Err(format!("Unknown display ID prefix in: {id}"))
}

/// Resolves a search prefix code to its API path by exact match.
///
/// Note the difference from [`lookup_display_id_prefix`]: that one matches the
/// start of a longer id, this one compares the whole code.
pub fn lookup_search_prefix<'a>(
    code: &str,
    prefixes: &[(&str, &'a str)],
) -> Result<&'a str, String> {
    for &(prefix, api_path) in prefixes {
        if prefix == code {
            return Ok(api_path);
        }
    }
    Err(format!("Unknown search prefix: {code}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREFIXES: &[(&str, &str)] = &[("SB", "/sbirka"), ("ZZ", "/zakon")];

    #[test]
    fn resolve_url_passes_an_absolute_url_through() {
        assert_eq!(
            resolve_url("https://base.example", "https://other.example/x"),
            "https://other.example/x"
        );
    }

    #[test]
    fn resolve_url_inserts_exactly_one_slash() {
        assert_eq!(
            resolve_url("https://b.example", "/a"),
            "https://b.example/a"
        );
        assert_eq!(resolve_url("https://b.example", "a"), "https://b.example/a");
    }

    #[test]
    fn join_url_treats_the_path_as_relative_even_when_it_looks_absolute() {
        // The difference from resolve_url: no pass-through, no doubled slash.
        assert_eq!(join_url("https://b.example", "/a"), "https://b.example/a");
        assert_eq!(join_url("https://b.example", "a"), "https://b.example/a");
    }

    #[test]
    fn join_url_returns_the_base_for_an_empty_path() {
        assert_eq!(join_url("https://b.example", ""), "https://b.example");
    }

    #[test]
    fn base_url_from_env_falls_back_and_strips_a_trailing_slash() {
        let var = "CDX_COMMON_TEST_BASE_URL_UNSET";
        std::env::remove_var(var);
        assert_eq!(
            base_url_from_env(var, "https://d.example/"),
            "https://d.example"
        );
    }

    #[test]
    fn format_status_omits_an_empty_reason() {
        assert_eq!(format_status(404, "Not Found"), "404 Not Found");
        assert_eq!(format_status(404, ""), "404");
    }

    #[test]
    fn a_status_of_400_and_up_is_a_failure() {
        assert!(is_failure(400));
        assert!(is_failure(404));
        assert!(is_failure(500));
        assert!(!is_failure(200));
        assert!(!is_failure(302));
    }

    #[test]
    fn html_is_recognised_regardless_of_leading_space() {
        assert!(looks_like_html("<!DOCTYPE html>"));
        assert!(looks_like_html("\n  <html>"));
        assert!(!looks_like_html(r#"{"ico":"27082440"}"#));
        assert!(!looks_like_html(""));
    }

    #[test]
    fn only_rate_limits_and_server_faults_are_retryable() {
        assert!(retryable_status(429));
        assert!(retryable_status(500));
        assert!(retryable_status(599));
        assert!(!retryable_status(400));
        assert!(!retryable_status(404));
        assert!(!retryable_status(200));
    }

    #[test]
    fn backoff_doubles_from_250ms() {
        assert_eq!(backoff(0), Duration::from_millis(250));
        assert_eq!(backoff(1), Duration::from_millis(500));
        assert_eq!(backoff(3), Duration::from_millis(2000));
    }

    #[test]
    fn display_id_prefix_matches_the_start_of_the_id() {
        assert_eq!(
            lookup_display_id_prefix("SB123/2026", PREFIXES).unwrap(),
            "/sbirka"
        );
        assert!(lookup_display_id_prefix("XX1", PREFIXES)
            .unwrap_err()
            .contains("XX1"));
    }

    #[test]
    fn search_prefix_needs_the_whole_code_to_match() {
        assert_eq!(lookup_search_prefix("SB", PREFIXES).unwrap(), "/sbirka");
        // Would match under lookup_display_id_prefix; must not match here.
        assert!(lookup_search_prefix("SB123", PREFIXES).is_err());
    }
}

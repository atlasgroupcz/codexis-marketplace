//! Thin authenticated HTTP wrapper around the ČÚZK REST API.
//! Port of Python's `katastr_core.api_client`.
//!
//! Everything here is synchronous (blocking) - matches the Python port,
//! keeps the CGI model of "start → do work → exit" intact, and lets us
//! avoid a tokio dependency.

use std::sync::OnceLock;
use std::time::Duration;

use crate::error::{KatastrError, Result};
use crate::settings::{cuzk_base, read_api_key};

/// One Agent per process. ureq keeps a connection pool inside it, keyed by
/// (scheme, host, port), so a second call to the same host reuses an
/// established connection. `search` benefits most: it calls `ags` twice.
///
/// Note: a connection only returns to the pool once its response body has
/// been read in full.
pub fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(Duration::from_secs(30))
            .build()
    })
}

/// Which transport failures are worth retrying.
/// `cdx_common::http::retryable_status` covers HTTP statuses (429, 5xx); this
/// is its counterpart for failures that carry no status at all - a TLS reset,
/// a timeout, a refused connection.
pub fn is_retryable_transport(kind: ureq::ErrorKind) -> bool {
    matches!(
        kind,
        ureq::ErrorKind::Io | ureq::ErrorKind::ConnectionFailed | ureq::ErrorKind::Dns
    )
}

/// How many times a call is retried before giving up.
const MAX_ATTEMPTS: usize = 3;

/// How a call ended once retrying stopped making sense.
pub enum Outcome {
    Ok(ureq::Response),
    /// The HTTP status and the body, already read. The body is always read so
    /// the connection returns to the pool even when the caller discards it.
    Status(u16, String),
    Transport(ureq::Transport),
}

/// Issues the request and retries while that makes sense: `retryable_status`
/// for HTTP statuses, [`is_retryable_transport`] for failures that carry no
/// status, `backoff` between attempts.
///
/// `build` rebuilds the request for every attempt, because calling a
/// `ureq::Request` consumes it.
///
/// When to retry is one rule for all three callers (ČÚZK, RÚIAN, key
/// validation), which is why it lives here. What the outcome is called for the
/// user stays with the caller - each of those services speaks differently.
pub fn call_with_retry(build: impl Fn() -> ureq::Request) -> Outcome {
    let mut attempt = 0;
    loop {
        match build().call() {
            Ok(resp) => return Outcome::Ok(resp),
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                if cdx_common::http::retryable_status(code) && attempt + 1 < MAX_ATTEMPTS {
                    std::thread::sleep(cdx_common::http::backoff(attempt));
                    attempt += 1;
                    continue;
                }
                return Outcome::Status(code, body);
            }
            Err(ureq::Error::Transport(t)) => {
                if is_retryable_transport(t.kind()) && attempt + 1 < MAX_ATTEMPTS {
                    std::thread::sleep(cdx_common::http::backoff(attempt));
                    attempt += 1;
                    continue;
                }
                return Outcome::Transport(t);
            }
        }
    }
}

/// Authenticated GET. `path` may start with `/` or not - both cases
/// produce the same URL. Returns the parsed JSON body.
///
/// Retries through [`call_with_retry`], so callers do not have to handle a
/// transient 429/5xx or a dropped connection themselves.
///
/// Error mapping matches the Python version so the CGI lookups stay in
/// sync:
/// - no key on disk → [`KatastrError::ApiKeyMissing`]
/// - 401/403 → [`KatastrError::ApiKeyInvalid`]
/// - other HTTP → [`KatastrError::ApiHttp`] (status + body snippet)
/// - connect/read failure → [`KatastrError::ApiNetwork`]
pub fn get(path: &str) -> Result<serde_json::Value> {
    let api_key = read_api_key();
    if api_key.is_empty() {
        return Err(KatastrError::ApiKeyMissing(
            "API klíč pro Katastr není nastaven. \
             Nastavte ho v UI (Doplňky → Katastr → ⚙) nebo přes \
             `katastr-cli settings set <KEY>`."
                .into(),
        ));
    }

    let base = cuzk_base();
    let url = cdx_common::http::resolve_url(&base, path);

    match call_with_retry(|| {
        agent()
            .get(&url)
            .set("ApiKey", &api_key)
            .set("Accept", "application/json")
    }) {
        Outcome::Ok(resp) => resp
            .into_json::<serde_json::Value>()
            .map_err(|e| KatastrError::ApiHttp {
                status: 500,
                message: format!("ČÚZK API vrátilo neplatný JSON: {e}"),
            }),
        Outcome::Status(code, _) if code == 401 || code == 403 => {
            Err(KatastrError::ApiKeyInvalid(format!(
                "API klíč pro Katastr byl odmítnut (HTTP {code})."
            )))
        }
        Outcome::Status(code, body) => Err(KatastrError::ApiHttp {
            status: code,
            message: crate::cuzk_error::human_message(code, &body, path),
        }),
        Outcome::Transport(t) => Err(KatastrError::ApiNetwork(format!(
            "Nepodařilo se kontaktovat ČÚZK API: {t}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_and_io_failures_are_retryable() {
        // The TLS reset this rewrite came from arrives as Io.
        assert!(is_retryable_transport(ureq::ErrorKind::Io));
        assert!(is_retryable_transport(ureq::ErrorKind::ConnectionFailed));
        assert!(is_retryable_transport(ureq::ErrorKind::Dns));
    }

    #[test]
    fn bad_url_is_not_retryable() {
        assert!(!is_retryable_transport(ureq::ErrorKind::InvalidUrl));
        assert!(!is_retryable_transport(ureq::ErrorKind::UnknownScheme));
    }

    #[test]
    fn agent_is_reused_across_calls() {
        let a = agent();
        let b = agent();
        assert!(std::ptr::eq(a, b), "agent must be one per process");
    }
}

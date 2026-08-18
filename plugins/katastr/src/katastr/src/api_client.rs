//! Thin authenticated HTTP wrapper around the ČÚZK REST API.
//! Port of Python's `katastr_core.api_client`.
//!
//! Everything here is synchronous (blocking) - matches the Python port,
//! keeps the CGI model of "start → do work → exit" intact, and lets us
//! avoid a tokio dependency.

use std::sync::OnceLock;
use std::time::Duration;

use crate::error::{KatastrError, Result};
use cdx_common::http_client::{call_with_retry, Outcome};

use crate::settings::{cuzk_base, read_api_key};

/// Sent with every ČÚZK request. The API authenticates by `ApiKey`, so this only
/// names the caller - the same shape the other lookup plugins send.
const USER_AGENT_VALUE: &str = concat!("katastr-cli/", env!("CARGO_PKG_VERSION"));

/// One Agent per process. ureq keeps a connection pool inside it, keyed by
/// (scheme, host, port), so a second call to the same host reuses an
/// established connection. `search` benefits most: it calls `ags` twice.
///
/// Note: a connection only returns to the pool once its response body has
/// been read in full.
pub fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        cdx_common::http_client::agent(
            USER_AGENT_VALUE,
            Duration::from_secs(5),
            Duration::from_secs(30),
        )
    })
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
            .header("ApiKey", &api_key)
            .header("Accept", "application/json")
            .call()
    }) {
        Outcome::Ok(mut resp) => resp
            .body_mut()
            .read_json::<serde_json::Value>()
            .map_err(|e| KatastrError::ApiHttp {
                status: 500,
                message: format!("ČÚZK API vrátilo neplatný JSON: {e}"),
            }),
        Outcome::Status(code, _) if code == 401 || code == 403 => Err(KatastrError::ApiKeyInvalid(
            format!("API klíč pro Katastr byl odmítnut (HTTP {code})."),
        )),
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

    // What is worth retrying is `cdx_common::http_client`'s rule now, and is
    // tested there. What stays here is that this plugin holds one agent.

    #[test]
    fn agent_is_reused_across_calls() {
        let a = agent();
        let b = agent();
        assert!(std::ptr::eq(a, b), "agent must be one per process");
    }
}

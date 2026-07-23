//! Thin authenticated HTTP wrapper around the ČÚZK REST API.
//! Port of Python's `katastr_core.api_client`.
//!
//! Everything here is synchronous (blocking) - matches the Python port,
//! keeps the CGI model of "start → do work → exit" intact, and lets us
//! avoid a tokio dependency.

use std::time::Duration;

use crate::error::{KatastrError, Result};
use crate::settings::{read_api_key, CUZK_BASE};

/// Authenticated GET. `path` may start with `/` or not - both cases
/// produce the same URL. Returns the parsed JSON body.
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

    let url = if path.starts_with('/') {
        format!("{CUZK_BASE}{path}")
    } else {
        format!("{CUZK_BASE}/{path}")
    };

    // Build a single-use agent per call - cheap compared to the network RTT
    // and we don't need connection pooling for a one-shot CGI process.
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(30))
        .build();

    match agent
        .get(&url)
        .set("ApiKey", &api_key)
        .set("Accept", "application/json")
        .call()
    {
        Ok(resp) => resp
            .into_json::<serde_json::Value>()
            .map_err(|e| KatastrError::ApiHttp {
                status: 500,
                message: format!("ČÚZK API vrátilo neplatný JSON: {e}"),
            }),
        Err(ureq::Error::Status(code, resp)) if code == 401 || code == 403 => {
            // Consume the body so the socket is released; content is not
            // interesting for auth errors.
            let _ = resp.into_string();
            Err(KatastrError::ApiKeyInvalid(format!(
                "API klíč pro Katastr byl odmítnut (HTTP {code})."
            )))
        }
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            let detail = if body.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", body.trim())
            };
            Err(KatastrError::ApiHttp {
                status: code,
                message: format!("ČÚZK API vrátilo HTTP {code} pro {path}{detail}"),
            })
        }
        Err(ureq::Error::Transport(t)) => Err(KatastrError::ApiNetwork(format!(
            "Nepodařilo se kontaktovat ČÚZK API: {t}"
        ))),
    }
}

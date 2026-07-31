//! The four settings operations, shared by both binaries.
//!
//! `datify-cli settings …` and the component's CGI expose the same four things,
//! and the panel is pinned to the payload shape - so a key added on one side and
//! not the other is a live drift. They live here once and both binaries call in.

use serde_json::{json, Value};

use crate::config;
use crate::{DatifyClient, Result};

/// What the settings page renders, and what `settings show` prints.
///
/// Deliberately infallible: this is the diagnostic view, so a malformed stored
/// URL must not stop it rendering - it falls back to the default and lets the
/// user see and fix the state. The token is never included, only a mask and
/// which layer supplied it.
pub fn payload() -> Value {
    let key = config::read_api_key();
    json!({
        "configured": !key.is_empty(),
        "maskedKey": if key.is_empty() { String::new() } else { config::mask_key(&key) },
        "keySource": config::read_api_key_source(),
        "apiUrl": config::read_api_url().unwrap_or_else(|_| config::DEFAULT_API_URL.to_string()),
        "settingsFile": config::env_path().display().to_string(),
        "generateUrl": config::read_api_key_url()
            .unwrap_or_else(|_| config::DEFAULT_API_KEY_URL.to_string()),
    })
}

/// Store a token only once Datify has accepted it, so a typo fails here instead
/// of on the agent's next call.
pub fn set_key(token: &str) -> Result<Value> {
    let collections = DatifyClient::with_token(token)?.collections()?;
    config::write_api_key(token)?;
    Ok(json!({
        "ok": true,
        "maskedKey": config::mask_key(token.trim()),
        "collectionCount": collection_count(&collections),
    }))
}

pub fn clear_key() -> Result<Value> {
    config::write_api_key("")?;
    Ok(json!({ "ok": true }))
}

/// Confirm Datify still accepts the stored token.
pub fn test() -> Result<Value> {
    let collections = DatifyClient::new()?.collections()?;
    Ok(json!({
        "ok": true,
        "collectionCount": collection_count(&collections),
    }))
}

fn collection_count(collections: &Value) -> Option<usize> {
    collections.as_array().map(|a| a.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdx_common::test_support::home_guard;

    #[test]
    fn the_payload_never_contains_the_token_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        config::write_api_key("abcdefghijkl").unwrap();

        let payload = payload();

        assert_eq!(payload["configured"], true);
        assert_eq!(payload["maskedKey"], "abcd****ijkl");
        assert!(!payload.to_string().contains("abcdefghijkl"), "{payload}");
    }

    #[test]
    fn an_unset_token_reports_as_not_configured() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        let payload = payload();
        assert_eq!(payload["configured"], false);
        assert_eq!(payload["maskedKey"], "");
        assert_eq!(payload["keySource"], "");
    }

    /// The diagnostic view has to survive the state it exists to diagnose.
    #[test]
    fn a_malformed_stored_url_falls_back_instead_of_failing() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        cdx_common::env_file::write_env_file(
            &config::env_path(),
            &[("DATIFY_API_URL", "not-a-url")],
        )
        .unwrap();

        assert_eq!(payload()["apiUrl"], config::DEFAULT_API_URL);
    }

    #[test]
    fn clearing_the_key_leaves_nothing_configured() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        config::write_api_key("abcdefghijkl").unwrap();

        assert_eq!(clear_key().unwrap()["ok"], true);

        assert_eq!(payload()["configured"], false);
    }
}

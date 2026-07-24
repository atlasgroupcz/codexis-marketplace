//! ČÚZK API key management. Port of Python's `katastr_core.settings`.
//!
//! The key lives in `$HOME/.cdx/apps/katastr/.env` as a single line
//! `API_KEY=<value>\n`, mode 0600. Shared with the legacy `kn` CLI tool -
//! either runtime can read what the other wrote.

use std::time::Duration;

use cdx_common::env_file;

use crate::error::{KatastrError, Result};
use crate::paths;

pub const CUZK_BASE: &str = "https://api-kn.cuzk.gov.cz";
const HEALTH_CHECK_PATH: &str = "/api/v1/AplikacniSluzby/StavUctu";

/// Read the stored API key. Returns the empty string when the file is
/// missing or the `API_KEY=` line can't be found - same permissive contract
/// as the Python original, so the CGI treats both as "not configured"
/// rather than erroring out.
pub fn read_api_key() -> String {
    env_file::read_env_var(&paths::env_file(), "API_KEY").unwrap_or_default()
}

/// Write the API key to disk atomically with mode 0600.
/// No validation here - use [`set_api_key`] for the validated path.
pub fn write_api_key(key: &str) -> Result<()> {
    Ok(env_file::write_env_file(&paths::env_file(), &[("API_KEY", key)])?)
}

/// Mask the key for display: `ABCD****WXYZ`. Anything shorter than 12
/// characters collapses to just `****` so we never leak half of a short
/// token.
pub fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() < 12 {
        return "****".to_string();
    }
    let head: String = chars[..4].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}****{tail}")
}

/// True when an API key is stored (file exists and contains an `API_KEY=`
/// line with a non-empty value).
pub fn is_configured() -> bool {
    !read_api_key().is_empty()
}

/// Read the stored key once and return its display status. Mirrors Python's
/// `settings.key_status` so callers needing both flags avoid re-reading `.env`.
pub struct KeyStatus {
    pub configured: bool,
    pub masked: String,
}

pub fn key_status() -> KeyStatus {
    let key = read_api_key();
    if key.is_empty() {
        KeyStatus {
            configured: false,
            masked: String::new(),
        }
    } else {
        KeyStatus {
            configured: true,
            masked: mask_key(&key),
        }
    }
}

/// Validate `key` against ČÚZK's account status endpoint.
///
/// Classifies errors into the same three buckets as the Python port so the
/// CGI can map them to distinct HTTP statuses:
/// - 401/403 → [`KatastrError::ApiKeyInvalid`]
/// - other HTTP → [`KatastrError::ApiHttp`]
/// - connection failure → [`KatastrError::ApiNetwork`]
pub fn validate_key(key: &str) -> Result<()> {
    let url = format!("{CUZK_BASE}{HEALTH_CHECK_PATH}");
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(10))
        .build();

    match agent
        .get(&url)
        .set("ApiKey", key)
        .set("Accept", "application/json")
        .call()
    {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(code, _)) if code == 401 || code == 403 => {
            Err(KatastrError::ApiKeyInvalid(format!(
                "API klíč je neplatný (HTTP {code})."
            )))
        }
        Err(ureq::Error::Status(code, _)) => Err(KatastrError::ApiHttp {
            status: code,
            message: format!("ČÚZK API vrátilo HTTP {code}"),
        }),
        Err(ureq::Error::Transport(t)) => Err(KatastrError::ApiNetwork(format!(
            "Nepodařilo se kontaktovat ČÚZK API: {t}"
        ))),
    }
}

/// Validate + store the API key. Trims whitespace, rejects empty input,
/// calls ČÚZK to confirm the key works, then writes it atomically.
pub fn set_api_key(key: &str) -> Result<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(KatastrError::ApiKeyInvalid("API klíč je prázdný.".into()));
    }
    validate_key(trimmed)?;
    write_api_key(trimmed)
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_short_key_hides_everything() {
        assert_eq!(mask_key(""), "****");
        assert_eq!(mask_key("abc"), "****");
        assert_eq!(mask_key("12345678901"), "****"); // 11 chars < 12
    }

    #[test]
    fn mask_long_key_shows_boundary_slice() {
        assert_eq!(mask_key("ABCDEFGHIJKL"), "ABCD****IJKL");
        assert_eq!(mask_key("ABCDEFGHIJKLMNOP"), "ABCD****MNOP");
    }

    #[test]
    fn read_returns_empty_when_file_missing() {
        // Point CODEXIS_PUBLIC_USER_HOME at a fresh temp dir so the real user
        // env stays untouched while exercising the path logic.
        let tmp = tempfile::tempdir().unwrap();
        let _g = cdx_common::test_support::home_guard(tmp.path());
        assert_eq!(read_api_key(), "");
        assert!(!is_configured());
    }

    #[test]
    fn write_then_read_roundtrips() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = cdx_common::test_support::home_guard(tmp.path());
        write_api_key("SECRET123").unwrap();
        assert_eq!(read_api_key(), "SECRET123");
        assert!(is_configured());
    }

    #[test]
    fn write_sets_mode_0600() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let _g = cdx_common::test_support::home_guard(tmp.path());
        write_api_key("SECRET").unwrap();
        let perms = std::fs::metadata(paths::env_file()).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }
}

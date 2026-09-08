//! ČÚZK API key management. Port of Python's `katastr_core.settings`.
//!
//! The key lives in `$HOME/.cdx/apps/katastr/.env` as a single line
//! `API_KEY=<value>\n`, mode 0600. Shared with the legacy `kn` CLI tool -
//! either runtime can read what the other wrote. The file is read-only inside
//! the plugin's isolation, so it is written by `cdxctl appenv` on the
//! plugin's behalf; katastr only reads it.

use cdx_common::env_file;

use crate::error::{KatastrError, Result};
use crate::paths;

const CUZK_BASE_DEFAULT: &str = "https://api-kn.cuzk.gov.cz";
const RUIAN_BASE_DEFAULT: &str = "https://ags.cuzk.gov.cz/arcgis/rest/services/RUIAN/MapServer";

/// Base URL of the cadastre REST API. Overridable by an env var so the host
/// can change without rebuilding the binary.
pub fn cuzk_base() -> String {
    cdx_common::http::base_url_from_env("CODEXIS_PLUGIN_KATASTR_API_URL", CUZK_BASE_DEFAULT)
}

/// Base URL of the RÚIAN MapServer on ags. This is where `search` goes.
pub fn ruian_base() -> String {
    cdx_common::http::base_url_from_env("CODEXIS_PLUGIN_KATASTR_RUIAN_URL", RUIAN_BASE_DEFAULT)
}

/// Read the stored API key. Returns the empty string when the file is
/// missing or the `API_KEY=` line can't be found - same permissive contract
/// as the Python original, so the CGI treats both as "not configured"
/// rather than erroring out.
pub fn read_api_key() -> String {
    env_file::read_env_var(&paths::env_file(), "API_KEY").unwrap_or_default()
}

/// Store the API key through `cdxctl appenv`; the file is read-only inside the
/// isolation, so the daemon writes it. No validation here - use [`set_api_key`].
pub fn store_api_key(key: &str) -> Result<()> {
    cdx_common::app_env::set("katastr", "API_KEY", key).map_err(app_env_error)
}

/// Remove the stored key through `cdxctl appenv`.
pub fn delete_api_key() -> Result<()> {
    cdx_common::app_env::delete("katastr", "API_KEY").map_err(app_env_error)
}

fn app_env_error(e: cdx_common::app_env::AppEnvError) -> KatastrError {
    KatastrError::AppEnv(format!("Uložení API klíče selhalo: {e}"))
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
///
/// Goes through `call_with_retry` like everything else: `settings set` and
/// `settings test` used to fail on the first network blip, being the only
/// calls that bypassed retrying.
pub fn validate_key(key: &str) -> Result<()> {
    use cdx_common::http_client::{call_with_retry, Outcome};

    use crate::api_client::agent;

    let base = cuzk_base();
    let url = cdx_common::http::resolve_url(&base, crate::lookup::STAV_UCTU);

    match call_with_retry(|| {
        agent()
            .get(&url)
            .header("ApiKey", key)
            .header("Accept", "application/json")
            .call()
    }) {
        // Read the body so the connection returns to the pool - `settings
        // test` makes further calls right after this one.
        Outcome::Ok(mut resp) => {
            let _ = resp.body_mut().read_to_string();
            Ok(())
        }
        Outcome::Status(code, _) if code == 401 || code == 403 => Err(KatastrError::ApiKeyInvalid(
            format!("API klíč je neplatný (HTTP {code})."),
        )),
        Outcome::Status(code, _) => Err(KatastrError::ApiHttp {
            status: code,
            message: format!("ČÚZK API vrátilo HTTP {code}"),
        }),
        Outcome::Transport(t) => Err(KatastrError::ApiNetwork(format!(
            "Nepodařilo se kontaktovat ČÚZK API: {t}"
        ))),
    }
}

/// Validate + store the API key. Trims whitespace, rejects empty input,
/// calls ČÚZK to confirm the key works, then stores it through `cdxctl appenv`.
pub fn set_api_key(key: &str) -> Result<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(KatastrError::ApiKeyInvalid("API klíč je prázdný.".into()));
    }
    validate_key(trimmed)?;
    store_api_key(trimmed)
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
    fn store_api_key_delegates_to_cdxctl_appenv() {
        let tmp = tempfile::tempdir().unwrap();
        let home = cdx_common::test_support::home_guard(tmp.path());
        let fake = cdx_common::test_support::fake_cdxctl(&home, tmp.path());

        store_api_key("SECRET123").unwrap();

        assert_eq!(fake.calls(), vec!["appenv katastr set API_KEY SECRET123"]);
        assert!(
            !paths::env_file().exists(),
            "katastr must not write .env itself"
        );
    }

    #[test]
    fn delete_api_key_delegates_to_cdxctl_appenv() {
        let tmp = tempfile::tempdir().unwrap();
        let home = cdx_common::test_support::home_guard(tmp.path());
        let fake = cdx_common::test_support::fake_cdxctl(&home, tmp.path());

        delete_api_key().unwrap();

        assert_eq!(fake.calls(), vec!["appenv katastr delete API_KEY"]);
    }

    #[test]
    fn cdxctl_failure_surfaces_as_app_env_error() {
        let tmp = tempfile::tempdir().unwrap();
        let home = cdx_common::test_support::home_guard(tmp.path());
        let fake = cdx_common::test_support::fake_cdxctl(&home, tmp.path());
        fake.set_exit(7);

        let error = store_api_key("SECRET123").unwrap_err();

        assert!(matches!(error, KatastrError::AppEnv(_)));
        assert!(error.to_string().contains("Uložení API klíče selhalo"));
    }

    #[test]
    fn read_still_uses_the_env_file() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = cdx_common::test_support::home_guard(tmp.path());
        std::fs::create_dir_all(paths::app_dir()).unwrap();
        std::fs::write(paths::env_file(), "API_KEY=FROMFILE\n").unwrap();

        assert_eq!(read_api_key(), "FROMFILE");
        assert!(is_configured());
    }

    #[test]
    fn cuzk_base_defaults_to_official_host() {
        std::env::remove_var("CODEXIS_PLUGIN_KATASTR_API_URL");
        assert_eq!(cuzk_base(), "https://api-kn.cuzk.gov.cz");
    }

    #[test]
    fn ruian_base_defaults_to_ags_mapserver() {
        std::env::remove_var("CODEXIS_PLUGIN_KATASTR_RUIAN_URL");
        assert_eq!(
            ruian_base(),
            "https://ags.cuzk.gov.cz/arcgis/rest/services/RUIAN/MapServer"
        );
    }
}

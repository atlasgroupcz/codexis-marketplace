//! Where the Datify API token and base URL come from.
//!
//! The token is user-supplied, so it is stored in the per-app credential file
//! (`~/.cdx/apps/datify/.env`, mode 0600) that the settings component writes.
//! Lookup order lets an automation override stored state without touching it:
//! host-injected `CODEXIS_USER_*`, then a plain env var, then the file.
//!
//! The file mechanics live in `cdx_common::env_file`; only what the keys mean
//! and how they are validated is here.

use std::path::PathBuf;

use cdx_common::config::read_env;
use cdx_common::env_file::{read_env_var, write_env_file};
use cdx_common::paths::app_dir;

use crate::error::{DatifyError, Result};

pub const APP_ID: &str = "datify";

const KEY_VAR: &str = "DATIFY_API_KEY";
/// Host-injected per-user secret; outranks everything else.
const USER_KEY_VAR: &str = "CODEXIS_USER_DATIFY_API_KEY";
const URL_VAR: &str = "DATIFY_API_URL";
const KEY_URL_VAR: &str = "DATIFY_API_KEY_URL";

pub const DEFAULT_API_URL: &str = "https://api.datify.cz";
/// Where the settings page sends the user to generate a token, taken from the
/// `externalDocs` link in Datify's own OpenAPI spec.
///
/// That link pointed at an on-prem dev host until 2026-07-28, and a token
/// generated there is rejected by the production API this plugin calls - so if
/// a user reports HTTP 401 on a token they just made, check this value first.
/// Overridable via `CODEXIS_PLUGIN_DATIFY_API_KEY_URL`.
pub const DEFAULT_API_KEY_URL: &str = "https://app.datify.cz/tokens";

/// The credential file backing this plugin's settings page.
pub fn env_path() -> PathBuf {
    app_dir(APP_ID).join(".env")
}

/// The env vars that outrank the credential file, in the order they are tried.
///
/// `read_api_key_source` reports which layer won, so it has to walk the same
/// list - hence one function, not a copy in each.
fn env_layers<'a>(host_var: &'a str, var: &'a str) -> [&'a str; 2] {
    [host_var, var]
}

/// Host-injected var, then a plain one, then the credential file, then a default.
fn layered(host_var: &str, var: &str, fallback: &str) -> String {
    env_layers(host_var, var)
        .into_iter()
        .find_map(read_env)
        .or_else(|| read_env_var(&env_path(), var))
        .unwrap_or_else(|| fallback.to_string())
        .trim()
        .to_string()
}

/// Reject anything that is not an absolute HTTP(S) URL, so a typo surfaces here
/// rather than as a confusing transport error later.
pub fn normalize_api_url(url: &str) -> Result<String> {
    let cleaned = url.trim().trim_end_matches('/');
    if cleaned.is_empty() {
        return Err(DatifyError::Config("Datify API URL je prázdná.".into()));
    }
    let rest = cleaned
        .strip_prefix("https://")
        .or_else(|| cleaned.strip_prefix("http://"))
        .ok_or_else(|| {
            DatifyError::Config("Datify API URL musí být absolutní HTTP(S) URL.".into())
        })?;
    if rest.is_empty() || rest.starts_with('/') {
        return Err(DatifyError::Config(
            "Datify API URL musí obsahovat hostitele.".into(),
        ));
    }
    Ok(cleaned.to_string())
}

pub fn read_api_url() -> Result<String> {
    normalize_api_url(&layered(
        "CODEXIS_PLUGIN_DATIFY_API_URL",
        URL_VAR,
        DEFAULT_API_URL,
    ))
}

pub fn read_api_key_url() -> Result<String> {
    normalize_api_url(&layered(
        "CODEXIS_PLUGIN_DATIFY_API_KEY_URL",
        KEY_URL_VAR,
        DEFAULT_API_KEY_URL,
    ))
}

pub fn read_api_key() -> String {
    layered(USER_KEY_VAR, KEY_VAR, "")
}

/// Which layer supplied the effective token, so the settings page can say
/// whether it is showing stored state or a host/automation override.
pub fn read_api_key_source() -> String {
    for var in env_layers(USER_KEY_VAR, KEY_VAR) {
        if read_env(var).is_some() {
            return var.to_string();
        }
    }
    if read_env_var(&env_path(), KEY_VAR).is_some() {
        return env_path().display().to_string();
    }
    String::new()
}

/// Persist the token, or clear it when `key` is empty.
pub fn write_api_key(key: &str) -> Result<()> {
    let path = env_path();
    let key = key.trim();
    let mut pairs: Vec<(String, String)> = cdx_common::env_file::read_env_file(&path)
        .into_iter()
        .filter(|(k, _)| k != KEY_VAR)
        .collect();
    if !key.is_empty() {
        pairs.push((KEY_VAR.to_string(), key.to_string()));
    }
    let borrowed: Vec<(&str, &str)> = pairs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    write_env_file(&path, &borrowed)?;
    Ok(())
}

/// `abcd****wxyz`, or `****` when the token is too short to reveal any of it.
pub fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() < 12 {
        return "****".to_string();
    }
    let head: String = chars[..4].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}****{tail}")
}

pub fn is_configured() -> bool {
    !read_api_key().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdx_common::test_support::home_guard;

    #[test]
    fn normalize_rejects_relative_and_hostless_urls() {
        assert!(normalize_api_url("api.datify.cz").is_err());
        assert!(normalize_api_url("").is_err());
        assert!(normalize_api_url("https://").is_err());
        assert!(normalize_api_url("https:///v0").is_err());
    }

    #[test]
    fn normalize_strips_a_trailing_slash() {
        assert_eq!(
            normalize_api_url("https://api.datify.cz/").unwrap(),
            "https://api.datify.cz"
        );
    }

    #[test]
    fn mask_reveals_the_ends_only_when_long_enough() {
        assert_eq!(mask_key("abcdefghijkl"), "abcd****ijkl");
        assert_eq!(mask_key("short"), "****");
    }

    #[test]
    fn key_roundtrips_through_the_credential_file() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        assert_eq!(read_api_key(), "");
        assert!(!is_configured());

        write_api_key("  token-value-1234  ").unwrap();

        assert_eq!(read_api_key(), "token-value-1234");
        assert!(is_configured());
        assert_eq!(read_api_key_source(), env_path().display().to_string());
    }

    #[test]
    fn writing_an_empty_key_clears_it() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        write_api_key("token-value-1234").unwrap();
        write_api_key("").unwrap();
        assert_eq!(read_api_key(), "");
        assert_eq!(read_api_key_source(), "");
    }

    #[test]
    fn writing_the_key_preserves_other_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        write_env_file(&env_path(), &[("DATIFY_API_URL", "https://other.example")]).unwrap();

        write_api_key("token-value-1234").unwrap();

        assert_eq!(
            read_env_var(&env_path(), "DATIFY_API_URL").as_deref(),
            Some("https://other.example")
        );
        assert_eq!(read_api_key(), "token-value-1234");
    }

    #[test]
    fn defaults_apply_when_nothing_is_stored() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        assert_eq!(read_api_url().unwrap(), DEFAULT_API_URL);
        assert_eq!(read_api_key_url().unwrap(), DEFAULT_API_KEY_URL);
    }
}

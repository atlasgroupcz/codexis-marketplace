use std::path::PathBuf;

use cdx_common::app_env::AppEnvError;
use cdx_common::config::read_env;
use cdx_common::env_file::read_env_var;
use cdx_common::paths::app_dir;

pub const APP_ID: &str = "turboleal";
pub const API_KEY_VAR: &str = "TURBOLEAL_API_KEY";
pub const API_URL_VAR: &str = "CODEXIS_PLUGIN_TURBOLEAL_API_URL";
pub const DEFAULT_API_URL: &str = "https://turboleal-profidata-main.onprem.agrp.dev";
pub const API_KEY_PREFIX: &str = "api-key-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    Flag,
    Env,
    File(PathBuf),
}

impl KeySource {
    pub fn describe(&self) -> String {
        match self {
            Self::Flag => "--api-key".to_owned(),
            Self::Env => API_KEY_VAR.to_owned(),
            Self::File(path) => path.display().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiKey {
    pub value: String,
    pub source: KeySource,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KeyFormatError {
    #[error("the API key is empty")]
    Empty,
    #[error("the API key must not contain whitespace")]
    Whitespace,
    #[error(
        "a Turboleal API key starts with `{API_KEY_PREFIX}`; create one in Turboleal under API klíče"
    )]
    Prefix,
}

pub fn env_path() -> PathBuf {
    app_dir(APP_ID).join(".env")
}

pub fn api_url() -> String {
    read_env(API_URL_VAR).unwrap_or_else(|| DEFAULT_API_URL.to_owned())
}

pub fn api_keys_url(api_url: &str) -> String {
    format!("{}/api-keys", api_url.trim().trim_end_matches('/'))
}

pub fn resolve_api_key(flag: Option<&str>) -> Option<ApiKey> {
    let flag = flag.map(str::trim).filter(|value| !value.is_empty());
    if let Some(value) = flag {
        return Some(ApiKey {
            value: value.to_owned(),
            source: KeySource::Flag,
        });
    }
    if let Some(value) = read_env(API_KEY_VAR) {
        return Some(ApiKey {
            value,
            source: KeySource::Env,
        });
    }
    let path = env_path();
    read_env_var(&path, API_KEY_VAR).map(|value| ApiKey {
        value,
        source: KeySource::File(path),
    })
}

pub fn check_format(key: &str) -> Result<&str, KeyFormatError> {
    let key = key.trim();
    if key.is_empty() {
        return Err(KeyFormatError::Empty);
    }
    if key.chars().any(char::is_whitespace) {
        return Err(KeyFormatError::Whitespace);
    }
    match key.strip_prefix(API_KEY_PREFIX) {
        Some(secret) if !secret.is_empty() => Ok(key),
        _ => Err(KeyFormatError::Prefix),
    }
}

pub fn mask(key: &str) -> String {
    let key = key.trim();
    let shown: String = match key.strip_prefix(API_KEY_PREFIX) {
        Some(secret) if secret.chars().count() >= 12 => {
            format!(
                "{API_KEY_PREFIX}{}",
                secret.chars().take(4).collect::<String>()
            )
        }
        Some(_) => API_KEY_PREFIX.to_owned(),
        None => String::new(),
    };
    format!("{shown}…")
}

pub fn store_api_key(key: &str) -> Result<(), AppEnvError> {
    cdx_common::app_env::set(APP_ID, API_KEY_VAR, key.trim())
}

pub fn clear_api_key() -> Result<(), AppEnvError> {
    cdx_common::app_env::delete(APP_ID, API_KEY_VAR)
}

#[cfg(test)]
mod tests {
    use cdx_common::env_file::write_env_file;
    use cdx_common::test_support::{fake_cdxctl, home_guard};

    use super::*;

    #[test]
    fn mask_keeps_only_the_prefix_and_four_characters() {
        assert_eq!(mask("api-key-AbCdEfGhIjKlMnOp"), "api-key-AbCd…");
        assert_eq!(mask("api-key-short"), "api-key-…");
        assert_eq!(mask("eyJhbGciOiJSUzI1NiJ9"), "…");
    }

    #[test]
    fn check_format_accepts_only_turboleal_keys() {
        assert_eq!(check_format("  api-key-abc123 \n"), Ok("api-key-abc123"));
        assert_eq!(check_format(" "), Err(KeyFormatError::Empty));
        assert_eq!(check_format("api-key-a b"), Err(KeyFormatError::Whitespace));
        assert_eq!(
            check_format("eyJhbGciOiJSUzI1NiJ9"),
            Err(KeyFormatError::Prefix)
        );
        assert_eq!(check_format("api-key-"), Err(KeyFormatError::Prefix));
    }

    #[test]
    fn api_keys_url_points_at_the_api_keys_page() {
        assert_eq!(
            api_keys_url("https://turboleal.example/"),
            "https://turboleal.example/api-keys"
        );
    }

    #[test]
    fn the_flag_wins_over_the_stored_key() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        write_env_file(&env_path(), &[(API_KEY_VAR, "api-key-stored")]).unwrap();

        let key = resolve_api_key(Some("api-key-flag")).unwrap();

        assert_eq!(key.value, "api-key-flag");
        assert_eq!(key.source, KeySource::Flag);
    }

    #[test]
    fn the_stored_key_is_read_from_the_app_env_file() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        assert!(resolve_api_key(None).is_none());

        write_env_file(&env_path(), &[(API_KEY_VAR, "api-key-stored")]).unwrap();

        let key = resolve_api_key(Some(" ")).unwrap();
        assert_eq!(key.value, "api-key-stored");
        assert_eq!(key.source, KeySource::File(env_path()));
    }

    #[test]
    fn storing_and_clearing_go_through_cdxctl_appenv() {
        let tmp = tempfile::tempdir().unwrap();
        let home = home_guard(tmp.path());
        let fake = fake_cdxctl(&home, tmp.path());

        store_api_key(" api-key-abc123 ").unwrap();
        clear_api_key().unwrap();

        assert_eq!(
            fake.calls(),
            vec![
                "appenv turboleal set TURBOLEAL_API_KEY api-key-abc123",
                "appenv turboleal delete TURBOLEAL_API_KEY",
            ]
        );
        assert!(!env_path().exists());
    }
}

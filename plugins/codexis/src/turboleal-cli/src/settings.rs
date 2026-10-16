use reqwest::StatusCode;
use serde_json::{Value, json};

use crate::api::{ApiClient, ApiError};
use crate::config::{self, KeyFormatError};
use crate::model::Me;

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("{0}")]
    Format(#[from] KeyFormatError),
    #[error("no Turboleal API key is set / API klíč Turbolealu není nastaven")]
    NotConfigured,
    #[error("Turboleal rejected the API key (HTTP {0}) / Turboleal API klíč odmítl")]
    Rejected(u16),
    #[error("{0}")]
    Api(#[from] ApiError),
    #[error("saving the API key failed: {0}")]
    Store(#[from] cdx_common::app_env::AppEnvError),
}

impl SettingsError {
    pub fn http_status(&self) -> u16 {
        match self {
            Self::Format(_) => 400,
            Self::NotConfigured => 412,
            Self::Rejected(_) => 422,
            Self::Api(_) => 502,
            Self::Store(_) => 500,
        }
    }
}

pub type SettingsResult<T> = Result<T, SettingsError>;

pub fn payload(api_url: &str, flag: Option<&str>) -> Value {
    let key = config::resolve_api_key(flag);
    json!({
        "configured": key.is_some(),
        "maskedKey": key.as_ref().map(|key| config::mask(&key.value)).unwrap_or_default(),
        "keySource": key.as_ref().map(|key| key.source.describe()).unwrap_or_default(),
        "settingsFile": config::env_path().display().to_string(),
        "apiUrl": api_url,
        "generateUrl": config::api_keys_url(api_url),
    })
}

pub fn login(api_url: &str, key: &str) -> SettingsResult<Value> {
    let key = config::check_format(key)?;
    let me = verify(api_url, key)?;
    config::store_api_key(key)?;
    Ok(json!({ "ok": true, "maskedKey": config::mask(key), "user": me }))
}

pub fn logout() -> SettingsResult<Value> {
    config::clear_api_key()?;
    Ok(json!({ "ok": true }))
}

pub fn status(api_url: &str, flag: Option<&str>) -> SettingsResult<Value> {
    let key = config::resolve_api_key(flag).ok_or(SettingsError::NotConfigured)?;
    let me = verify(api_url, &key.value)?;
    let mut payload = payload(api_url, flag);
    payload["ok"] = json!(true);
    payload["user"] = json!(me);
    Ok(payload)
}

fn verify(api_url: &str, key: &str) -> SettingsResult<Me> {
    ApiClient::new(api_url, key)?
        .me()
        .map_err(|error| match error {
            ApiError::Problem(problem)
                if problem.status == StatusCode::UNAUTHORIZED.as_u16()
                    || problem.status == StatusCode::FORBIDDEN.as_u16() =>
            {
                SettingsError::Rejected(problem.status)
            }
            other => SettingsError::Api(other),
        })
}

#[cfg(test)]
mod tests {
    use cdx_common::env_file::write_env_file;
    use cdx_common::test_support::home_guard;

    use super::*;

    #[test]
    fn the_payload_never_contains_the_key_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        write_env_file(
            &config::env_path(),
            &[(config::API_KEY_VAR, "api-key-AbCdEfGhIjKlMnOp")],
        )
        .unwrap();

        let payload = payload("https://turboleal.example", None);

        assert_eq!(payload["configured"], true);
        assert_eq!(payload["maskedKey"], "api-key-AbCd…");
        assert_eq!(payload["generateUrl"], "https://turboleal.example/api-keys");
        assert!(!payload.to_string().contains("EfGhIjKlMnOp"), "{payload}");
    }

    #[test]
    fn a_missing_key_reports_as_not_configured() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());

        let payload = payload("https://turboleal.example", None);

        assert_eq!(payload["configured"], false);
        assert_eq!(payload["maskedKey"], "");
        assert_eq!(payload["keySource"], "");
        assert!(matches!(
            status("https://turboleal.example", None),
            Err(SettingsError::NotConfigured)
        ));
    }

    #[test]
    fn login_rejects_a_key_of_the_wrong_shape_before_calling_turboleal() {
        let error = login("http://127.0.0.1:9", "eyJhbGciOiJSUzI1NiJ9").unwrap_err();

        assert!(matches!(
            error,
            SettingsError::Format(KeyFormatError::Prefix)
        ));
        assert_eq!(error.http_status(), 400);
    }
}

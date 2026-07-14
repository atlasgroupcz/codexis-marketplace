//! Config for the codexis (cdx-cli) plugin. `DaemonConfig` and the env/auth
//! helpers live in the `cdx-common` crate; the search `Config` stays here
//! because it differs from the other plugins: auth is required, and every
//! missing env var is reported at once.

use cdx_common::config::{read_env, to_authorization_header};

use crate::core::error::CliError;

pub(crate) use cdx_common::config::DaemonConfig;

pub(crate) const CODEXIS_PLUGIN_API_URL_ENV: &str = "CODEXIS_PLUGIN_API_URL";
pub(crate) const CODEXIS_USER_API_TOKEN_ENV: &str = "CODEXIS_USER_API_TOKEN";

pub(crate) struct Config {
    pub(crate) base_url: String,
    pub(crate) auth_header: String,
}

impl Config {
    pub(crate) fn load() -> Result<Self, CliError> {
        let base_url = read_env(CODEXIS_PLUGIN_API_URL_ENV);
        let jwt_auth = read_env(CODEXIS_USER_API_TOKEN_ENV);

        let mut missing = Vec::new();
        if base_url.is_none() {
            missing.push(CODEXIS_PLUGIN_API_URL_ENV);
        }
        if jwt_auth.is_none() {
            missing.push(CODEXIS_USER_API_TOKEN_ENV);
        }
        if !missing.is_empty() {
            return Err(CliError::MissingConfigVars(missing));
        }

        Ok(Self {
            base_url: base_url.unwrap().trim_end_matches('/').to_string(),
            auth_header: to_authorization_header(jwt_auth.as_deref().unwrap()),
        })
    }
}

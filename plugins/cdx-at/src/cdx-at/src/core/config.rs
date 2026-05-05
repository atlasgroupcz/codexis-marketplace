use std::env;

use crate::core::error::CliError;

pub(crate) const CODEXIS_PLUGIN_AT_API_URL_ENV: &str = "CODEXIS_PLUGIN_AT_API_URL";
pub(crate) const CODEXIS_USER_API_TOKEN_ENV: &str = "CODEXIS_USER_API_TOKEN";

pub(crate) struct Config {
    pub(crate) base_url: String,
    pub(crate) auth_header: Option<String>,
}

impl Config {
    pub(crate) fn load() -> Result<Self, CliError> {
        let base_url = read_env(CODEXIS_PLUGIN_AT_API_URL_ENV);
        let jwt_auth = read_env(CODEXIS_USER_API_TOKEN_ENV);

        match base_url {
            Some(url) => Ok(Self {
                base_url: url.trim_end_matches('/').to_string(),
                auth_header: jwt_auth.map(|v| to_authorization_header(&v)),
            }),
            None => Err(CliError::MissingConfig(CODEXIS_PLUGIN_AT_API_URL_ENV)),
        }
    }
}

fn read_env(name: &str) -> Option<String> {
    env::var(name).ok().and_then(normalize_config_value)
}

fn normalize_config_value(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn to_authorization_header(jwt_auth: &str) -> String {
    if starts_with_ignore_ascii_case(jwt_auth, "authorization:") {
        return jwt_auth.trim().to_string();
    }
    if starts_with_ignore_ascii_case(jwt_auth, "bearer ") {
        return format!("Authorization: {}", jwt_auth.trim());
    }
    if looks_like_jwt(jwt_auth) {
        return format!("Authorization: Bearer {}", jwt_auth.trim());
    }
    format!("Authorization: {}", jwt_auth.trim())
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value.len() >= prefix.len() && value[..prefix.len()].eq_ignore_ascii_case(prefix)
}

fn looks_like_jwt(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.contains(' ') {
        return false;
    }
    let mut parts = value.split('.');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(a), Some(b), Some(c), None) => !a.is_empty() && !b.is_empty() && !c.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_header_passes_through_full_header() {
        assert_eq!(
            to_authorization_header("Authorization: Bearer abc"),
            "Authorization: Bearer abc"
        );
    }

    #[test]
    fn authorization_header_wraps_bearer_value() {
        assert_eq!(
            to_authorization_header("Bearer abc"),
            "Authorization: Bearer abc"
        );
    }

    #[test]
    fn authorization_header_wraps_raw_jwt() {
        assert_eq!(
            to_authorization_header("a.b.c"),
            "Authorization: Bearer a.b.c"
        );
    }

    #[test]
    fn authorization_header_wraps_other_scheme() {
        assert_eq!(
            to_authorization_header("Basic xyz"),
            "Authorization: Basic xyz"
        );
    }

    #[test]
    fn normalize_rejects_blank_and_trims_value() {
        assert_eq!(normalize_config_value("  ".to_string()), None);
        assert_eq!(
            normalize_config_value("  abc  ".to_string()),
            Some("abc".to_string())
        );
    }
}

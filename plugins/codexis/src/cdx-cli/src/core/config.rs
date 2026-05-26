use std::env;

use crate::core::error::CliError;

pub(crate) const CODEXIS_PLUGIN_API_URL_ENV: &str = "CODEXIS_PLUGIN_API_URL";
pub(crate) const CODEXIS_USER_API_TOKEN_ENV: &str = "CODEXIS_USER_API_TOKEN";
pub(crate) const CODEXIS_PUBLIC_DAEMON_URL_ENV: &str = "CODEXIS_PUBLIC_DAEMON_URL";
pub(crate) const CODEXIS_PUBLIC_USER_HOME_ENV: &str = "CODEXIS_PUBLIC_USER_HOME";
pub(crate) const CODEXIS_PUBLIC_SESSION_ID_ENV: &str = "CODEXIS_PUBLIC_SESSION_ID";
pub(crate) const CODEXIS_PUBLIC_TOOL_CALL_ID_ENV: &str = "CODEXIS_PUBLIC_TOOL_CALL_ID";

const DAEMON_SECRET_RELATIVE_PATH: &str = ".cdx/env/secret";

pub(crate) struct Config {
    pub(crate) base_url: String,
    pub(crate) auth_header: String,
}

pub(crate) struct DaemonConfig {
    pub(crate) auth_header: String,
    pub(crate) chat_id: String,
    pub(crate) tool_call_id: String,
    daemon_url: String,
}

impl DaemonConfig {
    /// Loads daemon config from the gateway-injected env vars and `~/.cdx/env/secret`.
    /// Returns `None` when any piece is missing — outside a chat VM context the sources
    /// publish path is silently disabled.
    pub(crate) fn load() -> Option<Self> {
        let daemon_url = read_env(CODEXIS_PUBLIC_DAEMON_URL_ENV)?;
        let token = read_daemon_secret()?;
        let chat_id = read_env(CODEXIS_PUBLIC_SESSION_ID_ENV)?;
        let tool_call_id = read_env(CODEXIS_PUBLIC_TOOL_CALL_ID_ENV)?;
        Some(Self {
            daemon_url,
            auth_header: to_authorization_header(&token),
            chat_id,
            tool_call_id,
        })
    }

    #[cfg(test)]
    pub(crate) fn test_only_new(
        daemon_url: String,
        auth_header: String,
        chat_id: String,
        tool_call_id: String,
    ) -> Self {
        Self {
            daemon_url,
            auth_header,
            chat_id,
            tool_call_id,
        }
    }

    /// `CODEXIS_PUBLIC_DAEMON_URL` points at the daemon's GraphQL endpoint; strip the
    /// trailing `/graphql` to reach the daemon REST base. The gateway proxies `/rest/*`
    /// straight through to the daemon backend.
    pub(crate) fn sources_url(&self) -> String {
        let base = self
            .daemon_url
            .trim_end_matches('/')
            .trim_end_matches("/graphql")
            .trim_end_matches('/');
        format!(
            "{base}/rest/v1/plugin/chats/{}/tool-calls/{}/sources",
            self.chat_id, self.tool_call_id
        )
    }
}

fn read_daemon_secret() -> Option<String> {
    let home = read_env(CODEXIS_PUBLIC_USER_HOME_ENV).or_else(|| read_env("HOME"))?;
    let path = format!("{}/{}", home.trim_end_matches('/'), DAEMON_SECRET_RELATIVE_PATH);
    std::fs::read_to_string(path).ok().and_then(normalize_config_value)
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
            return Err(CliError::MissingConfig(missing));
        }

        Ok(Self {
            base_url: base_url.unwrap().trim_end_matches('/').to_string(),
            auth_header: to_authorization_header(jwt_auth.as_deref().unwrap()),
        })
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
        let header = to_authorization_header("Authorization: Bearer abc");
        assert_eq!(header, "Authorization: Bearer abc");
    }

    #[test]
    fn authorization_header_wraps_bearer_value() {
        let header = to_authorization_header("Bearer abc");
        assert_eq!(header, "Authorization: Bearer abc");
    }

    #[test]
    fn authorization_header_wraps_raw_jwt() {
        let header = to_authorization_header("a.b.c");
        assert_eq!(header, "Authorization: Bearer a.b.c");
    }

    #[test]
    fn authorization_header_wraps_other_scheme() {
        let header = to_authorization_header("Basic xyz");
        assert_eq!(header, "Authorization: Basic xyz");
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

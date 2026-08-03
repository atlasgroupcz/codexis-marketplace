use std::env;

use crate::error::CliError;

/// Per-user API token injected by the host. Same name for every plugin.
pub const CODEXIS_USER_API_TOKEN_ENV: &str = "CODEXIS_USER_API_TOKEN";

/// Host-injected runtime context. Same names for every plugin.
pub const CODEXIS_PUBLIC_DAEMON_URL_ENV: &str = "CODEXIS_PUBLIC_DAEMON_URL";
pub const CODEXIS_PUBLIC_USER_HOME_ENV: &str = "CODEXIS_PUBLIC_USER_HOME";
pub const CODEXIS_PUBLIC_SESSION_ID_ENV: &str = "CODEXIS_PUBLIC_SESSION_ID";
pub const CODEXIS_PUBLIC_TOOL_CALL_ID_ENV: &str = "CODEXIS_PUBLIC_TOOL_CALL_ID";
pub const CODEXIS_PUBLIC_SUBAGENT_RUN_ID_ENV: &str = "CODEXIS_PUBLIC_SUBAGENT_RUN_ID";

const DAEMON_SECRET_RELATIVE_PATH: &str = ".cdx/env/secret";

/// Configuration for talking to a CODEXIS search API.
///
/// `base_url` comes from a plugin-specific env var (passed to [`SearchConfig::load`]);
/// the auth token env var is the same for every plugin.
pub struct SearchConfig {
    pub base_url: String,
    pub auth_header: Option<String>,
}

impl SearchConfig {
    /// Loads the search config, reading `api_url_env` for the base URL and the
    /// shared `CODEXIS_USER_API_TOKEN` for auth. Errors if the base URL is unset.
    pub fn load(api_url_env: &'static str) -> Result<Self, CliError> {
        let base_url = read_env(api_url_env);
        let jwt_auth = read_env(CODEXIS_USER_API_TOKEN_ENV);

        match base_url {
            Some(url) => Ok(Self {
                base_url: url.trim_end_matches('/').to_string(),
                auth_header: jwt_auth.map(|v| to_authorization_header(&v)),
            }),
            None => Err(CliError::MissingConfig(api_url_env)),
        }
    }

    /// Like [`SearchConfig::load`] but falls back to `default_base_url` instead
    /// of erroring when `api_url_env` is unset. Used by plugins that ship a
    /// public default endpoint.
    pub fn load_or_default(api_url_env: &str, default_base_url: &str) -> Self {
        let base_url = read_env(api_url_env).unwrap_or_else(|| default_base_url.to_string());
        let jwt_auth = read_env(CODEXIS_USER_API_TOKEN_ENV);
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header: jwt_auth.map(|v| to_authorization_header(&v)),
        }
    }
}

/// Configuration for publishing sources back to the CDX daemon.
///
/// Byte-identical across every plugin that publishes sources. Reads
/// gateway-injected env vars plus the on-disk daemon secret.
pub struct DaemonConfig {
    auth_header: String,
    chat_id: String,
    tool_call_id: String,
    subagent_run_id: Option<String>,
    daemon_url: String,
}

impl DaemonConfig {
    /// Loads daemon config from gateway-injected env + `~/.cdx/env/secret`.
    /// Returns `None` outside a chat-VM context (sources publish silently disabled).
    pub fn load() -> Option<Self> {
        let daemon_url = read_env(CODEXIS_PUBLIC_DAEMON_URL_ENV)?;
        let token = read_daemon_secret()?;
        let chat_id = read_env(CODEXIS_PUBLIC_SESSION_ID_ENV)?;
        let tool_call_id = read_env(CODEXIS_PUBLIC_TOOL_CALL_ID_ENV)?;
        let subagent_run_id = read_env(CODEXIS_PUBLIC_SUBAGENT_RUN_ID_ENV);
        Some(Self {
            daemon_url,
            auth_header: to_authorization_header(&token),
            chat_id,
            tool_call_id,
            subagent_run_id,
        })
    }

    /// Constructs a `DaemonConfig` from explicit parts. Intended for tests in
    /// consumer crates; the normal path is [`DaemonConfig::load`].
    #[doc(hidden)]
    pub fn from_parts(
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
            subagent_run_id: None,
        }
    }

    pub fn auth_header(&self) -> &str {
        &self.auth_header
    }

    /// Daemon REST sources endpoint for the active chat + tool-call.
    pub fn sources_url(&self) -> String {
        let base = self
            .daemon_url
            .trim_end_matches('/')
            .trim_end_matches("/graphql")
            .trim_end_matches('/');
        match self.subagent_run_id.as_deref() {
            Some(run) => format!(
                "{base}/rest/v1/plugin/chats/{}/subagents/{}/tool-calls/{}/sources",
                self.chat_id, run, self.tool_call_id
            ),
            None => format!(
                "{base}/rest/v1/plugin/chats/{}/tool-calls/{}/sources",
                self.chat_id, self.tool_call_id
            ),
        }
    }
}

/// Reads the VM daemon token from `$HOME/.cdx/env/secret`. The host writes the
/// secret to the real VM home - the same file `cdxctl` and the Python cores
/// read. The daemon-injected `CODEXIS_PUBLIC_USER_HOME` is deliberately NOT
/// consulted: it points at per-user app state, not at the VM home. Single
/// source of truth for the secret lookup - [`DaemonConfig::load`] and
/// `daemon_rest::daemon_bearer` both use it.
pub fn read_daemon_secret() -> Option<String> {
    let home = read_env("HOME")?;
    let path = format!("{}/{}", home.trim_end_matches('/'), DAEMON_SECRET_RELATIVE_PATH);
    std::fs::read_to_string(path).ok().and_then(normalize_config_value)
}

/// Reads an env var, returning `None` for unset or blank values.
pub fn read_env(name: &str) -> Option<String> {
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

/// Normalizes a raw token/header value into a full `Authorization:` header line.
pub fn to_authorization_header(jwt_auth: &str) -> String {
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

    #[test]
    fn sources_url_strips_graphql_suffix() {
        let config = DaemonConfig::from_parts(
            "https://daemon.example/graphql".to_string(),
            "Authorization: Bearer a.b.c".to_string(),
            "chat-1".to_string(),
            "tc-1".to_string(),
        );
        assert_eq!(
            config.sources_url(),
            "https://daemon.example/rest/v1/plugin/chats/chat-1/tool-calls/tc-1/sources"
        );
    }
}

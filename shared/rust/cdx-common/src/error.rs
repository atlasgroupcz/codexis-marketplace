use std::fmt;
use std::io;

/// Error type shared by the CODEXIS search CLIs.
///
/// This is the superset of the per-plugin `CliError` enums. Individual plugins
/// only construct the variants they need; unused variants are harmless.
#[derive(Debug)]
pub enum CliError {
    /// A single required env var is missing.
    MissingConfig(&'static str),
    /// Several required env vars are missing at once (collected before reporting).
    MissingConfigVars(Vec<&'static str>),
    Io { context: String, source: io::Error },
    InvalidJson(String),
    SerializePayload(String),
    SearchPayloadMustBeObject,
    InvalidSearchArgument(String),
    InvalidCdxUrl(String),
    InvalidStoredSchema(String),
    CurlSpawn(io::Error),
    RequestFailed { code: i32 },
    CommandTerminated { command: &'static str },
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::RequestFailed { code } => *code,
            Self::MissingConfig(_)
            | Self::MissingConfigVars(_)
            | Self::Io { .. }
            | Self::InvalidJson(_)
            | Self::SerializePayload(_)
            | Self::SearchPayloadMustBeObject
            | Self::InvalidSearchArgument(_)
            | Self::InvalidCdxUrl(_)
            | Self::InvalidStoredSchema(_) => 2,
            Self::CurlSpawn(_) | Self::CommandTerminated { .. } => 1,
        }
    }

    pub fn should_print(&self) -> bool {
        !matches!(self, Self::RequestFailed { .. })
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::CurlSpawn(source) => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConfig(name) => write!(
                f,
                "missing required configuration: {name}. Set it in the environment."
            ),
            Self::MissingConfigVars(names) => write!(
                f,
                "missing required configuration: {}. Set them in the environment.",
                names.join(", ")
            ),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::InvalidJson(message) => write!(f, "invalid JSON payload: {message}"),
            Self::SerializePayload(message) => write!(f, "failed to serialize payload: {message}"),
            Self::SearchPayloadMustBeObject => {
                write!(f, "search payload must be a JSON object")
            }
            Self::InvalidSearchArgument(message) => write!(f, "{message}"),
            Self::InvalidCdxUrl(message) => write!(f, "{message}"),
            Self::InvalidStoredSchema(message) => write!(f, "invalid stored schema: {message}"),
            Self::CurlSpawn(source) => write!(f, "failed to run curl: {source}"),
            Self::RequestFailed { code } => write!(f, "request failed with exit code {code}"),
            Self::CommandTerminated { command } => {
                write!(f, "{command} terminated without an exit status")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn cli_error_is_std_error_with_io_source() {
        let e = CliError::Io {
            context: "ctx".to_string(),
            source: io::Error::other("boom"),
        };
        let dyn_err: &dyn Error = &e;
        assert!(dyn_err.source().is_some(), "Io carries a source");
        assert!(CliError::MissingConfig("X").source().is_none());
        assert!(CliError::CurlSpawn(io::Error::other("x")).source().is_some());
    }
}

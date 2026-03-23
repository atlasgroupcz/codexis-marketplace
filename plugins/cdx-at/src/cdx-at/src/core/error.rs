use std::fmt;
use std::io;

use crate::core::config::CDX_ENV_FILE_DISPLAY_PATH;

#[derive(Debug)]
pub(crate) enum CliError {
    MissingConfig(&'static str),
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
    pub(crate) fn exit_code(&self) -> i32 {
        match self {
            Self::RequestFailed { code } => *code,
            Self::MissingConfig(_)
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

    pub(crate) fn should_print(&self) -> bool {
        !matches!(self, Self::RequestFailed { .. })
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConfig(name) => write!(
                f,
                "missing required configuration: {name}. Set it in the environment or {CDX_ENV_FILE_DISPLAY_PATH}."
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

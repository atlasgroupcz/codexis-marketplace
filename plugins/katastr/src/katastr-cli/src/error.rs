//! Error types. Mirrors the Python `katastr_core.exceptions` hierarchy -
//! each variant maps to a specific HTTP status on the CGI side (see
//! `error_status` in the handler) and to an exit code on the CLI side.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, KatastrError>;

#[derive(Debug, Error)]
pub enum KatastrError {
    /// API key not configured yet.
    #[error("{0}")]
    ApiKeyMissing(String),

    /// ČÚZK rejected the key with 401/403.
    #[error("{0}")]
    ApiKeyInvalid(String),

    /// Network error talking to ČÚZK.
    #[error("{0}")]
    ApiNetwork(String),

    /// ČÚZK returned an unexpected HTTP status.
    #[error("{message}")]
    ApiHttp { status: u16, message: String },

    /// Proceeding number doesn't match expected format (TYP-ČÍSLO/ROK-KÓD).
    #[error("{0}")]
    InvalidProceedingNumber(String),

    /// Proceeding doesn't exist in ČÚZK.
    #[error("{0}")]
    ProceedingNotFound(String),

    /// Proceeding isn't in the local tracking state.
    #[error("{0}")]
    ProceedingNotTracked(String),

    /// Attempted to add a proceeding that is already tracked.
    #[error("{0}")]
    ProceedingAlreadyTracked(String),

    /// A recipient e-mail address failed validation (maps Python's `ValueError`
    /// from `notify_settings.clean_recipients`). Surfaces as HTTP 400.
    #[error("{0}")]
    InvalidRecipient(String),

    /// The daemon notify transport (e-mail send / in-app notification) failed.
    /// Notifications are best-effort, so this is usually swallowed - but the CLI
    /// `check` path reports it as an informational line like Python does.
    #[error("{0}")]
    Notify(String),

    /// Filesystem / IO failure (reading state, writing atomically).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON (de)serialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<cdx_notify::NotifyError> for KatastrError {
    fn from(e: cdx_notify::NotifyError) -> Self {
        use cdx_notify::NotifyError as N;
        match e {
            N::InvalidRecipient(s) => KatastrError::InvalidRecipient(s),
            N::Notify(s) => KatastrError::Notify(s),
            N::Io(e) => KatastrError::Io(e),
            N::Json(e) => KatastrError::Json(e),
        }
    }
}

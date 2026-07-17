//! Error type for the notify capability. Each plugin's own error enum converts
//! from this via a `From` impl, so a plugin's CGI status mapping keeps working.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, NotifyError>;

#[derive(Debug, Error)]
pub enum NotifyError {
    /// A recipient e-mail address failed validation. Surfaces as HTTP 400.
    #[error("{0}")]
    InvalidRecipient(String),

    /// The daemon notify transport (e-mail send / in-app notification) failed.
    #[error("{0}")]
    Notify(String),

    /// Filesystem / IO failure (e.g. from `cdx_common::fs`).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON (de)serialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl NotifyError {
    /// The HTTP status a CGI handler should return for this error: an invalid
    /// recipient is a client error (400), everything else a server error (500).
    /// Lets every component plugin map the shared notify variants uniformly
    /// instead of re-implementing the same match.
    pub fn http_status(&self) -> u16 {
        match self {
            NotifyError::InvalidRecipient(_) => 400,
            _ => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_variant_displays_message_verbatim() {
        assert_eq!(NotifyError::Notify("boom".into()).to_string(), "boom");
    }

    #[test]
    fn http_status_maps_invalid_recipient_to_400_else_500() {
        assert_eq!(NotifyError::InvalidRecipient("x".into()).http_status(), 400);
        assert_eq!(NotifyError::Notify("x".into()).http_status(), 500);
        let io = NotifyError::Io(std::io::Error::other("x"));
        assert_eq!(io.http_status(), 500);
    }
}

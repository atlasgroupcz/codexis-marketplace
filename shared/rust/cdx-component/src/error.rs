//! Error type shared by the hosted-component building blocks. Each plugin's own
//! error enum converts from this via a `From` impl, so a plugin's CGI status
//! mapping keeps working unchanged.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ComponentError>;

#[derive(Debug, Error)]
pub enum ComponentError {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notify_variant_displays_message_verbatim() {
        assert_eq!(ComponentError::Notify("boom".into()).to_string(), "boom");
    }
}

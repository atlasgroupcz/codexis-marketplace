//! Errors for the watcher domain. Neutral messages - each plugin converts these
//! into its own (localized) error type via a `From` impl.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WatchError {
    /// The requested item is not in the store.
    #[error("not tracked: {0}")]
    NotTracked(String),

    /// A confirm index was outside the change list.
    #[error("change index {index} out of range (0..{len})")]
    IndexOutOfRange { index: usize, len: usize },
}

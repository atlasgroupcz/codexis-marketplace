//! Shared logic for the Datify plugin: credential storage and the REST client.
//!
//! Both binaries build on this - `datify-cli` (installed system-wide) and
//! `datify-cgi` (the settings component, run in place by the daemon) - so the
//! token is read and written the same way from either.

pub mod client;
pub mod config;
pub mod error;
pub mod settings;

/// A stub Datify for this crate's own tests. Behind a feature so the
/// integration tests, which link the lib as a dependency, can reach it too.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub use client::DatifyClient;
pub use error::{DatifyError, Result};

//! The notify capability shared by the CODEXIS watcher/component plugins:
//! daemon e-mail + in-app bell transport, the layered per-app notify-settings
//! store, and the branded HTML e-mail shell. Builds on `cdx-common` for the
//! daemon token, config and atomic fs.
//!
//! This is a *capability* crate, not a "component framework": it is independent
//! of the CGI runtime (`cdx-cgi`) and of the watcher/tracking domain
//! (`cdx-watch`). A plugin composes the capabilities it actually uses.

pub mod email;
pub mod error;
pub mod notify;
pub mod notify_settings;

pub use error::{NotifyError, Result};

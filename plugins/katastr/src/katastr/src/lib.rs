//! Shared core library for the katastr plugin.
//!
//! Port of the Python `katastr_core` package. Used by both the `katastr-cli`
//! binary (user-facing command line) and the `katastr-cgi` handler (HTTP
//! backend serving the plugin UI). All heavy logic - ČÚZK API calls, state
//! management, change detection, notifications - lives here so neither caller
//! has to duplicate it.

pub mod api_client;
pub mod cuzk_error;
pub mod error;
pub mod geom;
pub mod lookup;
pub mod notify;
pub mod notify_settings;
pub mod paths;
pub mod pravo_stavby;
pub mod ruian;
pub mod settings;
pub mod state;
pub mod tracking;

pub use cdx_common::fs;

pub use error::{KatastrError, Result};
pub use notify_settings::NotifyConfig;
pub use state::{Change, Operace, ProceedingState};
pub use tracking::{CheckResult, OverviewEntry, ProceedingNumber};

//! Shared core library for the katastr plugin.
//!
//! Port of the Python `katastr_core` package. Used by both the `katastr-cli`
//! binary (user-facing command line) and the `katastr-cgi` handler (HTTP
//! backend serving the plugin UI). All heavy logic - ČÚZK API calls, state
//! management, change detection, notifications - lives here so neither caller
//! has to duplicate it.

pub mod api_client;
pub mod error;
pub mod notify;
pub mod notify_settings;
pub mod paths;
pub mod settings;
pub mod state;
pub mod tracking;

pub use cdx_common::fs;

pub use error::{KatastrError, Result};
pub use notify_settings::NotifyConfig;
pub use state::{Change, Operace, ProceedingState};
pub use tracking::{CheckResult, OverviewEntry, ProceedingNumber};

/// Test-only helpers shared across module test suites. Kept in the library (not
/// a `tests/` integration crate) so unit tests for private items can use them.
#[cfg(test)]
pub(crate) mod test_support {
    use std::path::Path;
    use std::sync::{Mutex, MutexGuard};

    /// `std::env::set_var` is process-global and not thread-safe; serialize the
    /// tests that repoint the user home directory.
    static LOCK: Mutex<()> = Mutex::new(());

    pub struct HomeGuard {
        _guard: MutexGuard<'static, ()>,
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            std::env::remove_var("CODEXIS_PUBLIC_USER_HOME");
            std::env::remove_var("CDX_USER_HOME");
            std::env::remove_var("HOME");
        }
    }

    /// Point every user-home env var at `home` for the duration of the guard, so
    /// state, settings and defaults all resolve inside the scratch dir.
    pub fn home_guard(home: &Path) -> HomeGuard {
        let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("CODEXIS_PUBLIC_USER_HOME", home);
        std::env::set_var("CDX_USER_HOME", home);
        std::env::set_var("HOME", home);
        HomeGuard { _guard: guard }
    }
}

//! Shared building blocks for the codexis-marketplace Rust CLI plugins.
//!
//! These plugins are separate Cargo crates (no workspace) that each ship a
//! single binary. Historically they copy-pasted the same daemon-publishing
//! plumbing, config loading and error type. This crate holds the parts that
//! were byte-identical (or differed only by a plugin-specific env var) so the
//! plugins can depend on it by relative path instead of duplicating code.
//!
//! Nothing here is plugin-specific. Anything that genuinely differs per plugin
//! (search schemas, CLI commands, the curl search engine) stays in the plugin.

pub mod config;
pub mod daemon_rest;
pub mod error;
pub mod paths;

#[cfg(feature = "curl")]
pub mod curl;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "http-client")]
pub mod http_client;

#[cfg(feature = "reqwest-client")]
pub mod reqwest_client;

#[cfg(feature = "url")]
pub mod url;

#[cfg(feature = "schema")]
pub mod schema;

#[cfg(feature = "search")]
pub mod search;

#[cfg(feature = "search-engine")]
pub mod search_engine;

#[cfg(feature = "sources")]
pub mod sources;

#[cfg(feature = "ico")]
pub mod ico;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "env-file")]
pub mod env_file;

#[cfg(feature = "settings")]
pub mod settings;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "store")]
pub mod store;

/// Test-only helpers, shared with sibling crates via the `test-support` feature.
/// Gated so nothing ships in normal release builds.
#[cfg(any(test, feature = "test-support"))]
pub mod test_support {
    use std::path::Path;
    use std::sync::{Mutex, MutexGuard};

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

    /// Point every user-home env var at `home` for the guard's lifetime.
    /// `std::env::set_var` is process-global; the lock serialises these tests.
    pub fn home_guard(home: &Path) -> HomeGuard {
        let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("CODEXIS_PUBLIC_USER_HOME", home);
        std::env::set_var("CDX_USER_HOME", home);
        std::env::set_var("HOME", home);
        HomeGuard { _guard: guard }
    }
}

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

#[cfg(feature = "app-env")]
pub mod app_env;

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

    /// A fake `cdxctl` on `PATH` that records its argv into `<dir>/cdxctl-calls.log`
    /// and exits with the code in `<dir>/cdxctl-exit` (default 0). Takes the
    /// `HomeGuard` so PATH is only changed under the same lock.
    pub struct FakeCdxctl {
        dir: std::path::PathBuf,
        previous_path: Option<std::ffi::OsString>,
    }

    pub fn fake_cdxctl(_home: &HomeGuard, dir: &Path) -> FakeCdxctl {
        use std::os::unix::fs::PermissionsExt;
        let script = dir.join("cdxctl");
        std::fs::write(
            &script,
            "#!/bin/sh\nhere=\"$(dirname \"$0\")\"\nprintf '%s\\n' \"$*\" >> \"$here/cdxctl-calls.log\"\ncode=$(cat \"$here/cdxctl-exit\" 2>/dev/null || echo 0)\n[ \"$code\" -eq 0 ] || echo \"fake failure\" >&2\nexit \"$code\"\n",
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let previous_path = std::env::var_os("PATH");
        let mut paths = vec![dir.to_path_buf()];
        if let Some(ref previous) = previous_path {
            paths.extend(std::env::split_paths(previous));
        }
        std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
        FakeCdxctl {
            dir: dir.to_path_buf(),
            previous_path,
        }
    }

    impl FakeCdxctl {
        /// One line per invocation: the argv joined by spaces.
        pub fn calls(&self) -> Vec<String> {
            std::fs::read_to_string(self.dir.join("cdxctl-calls.log"))
                .map(|s| s.lines().map(str::to_string).collect())
                .unwrap_or_default()
        }

        pub fn set_exit(&self, code: i32) {
            std::fs::write(self.dir.join("cdxctl-exit"), code.to_string()).unwrap();
        }
    }

    impl Drop for FakeCdxctl {
        fn drop(&mut self) {
            match &self.previous_path {
                Some(path) => std::env::set_var("PATH", path),
                None => std::env::remove_var("PATH"),
            }
        }
    }
}

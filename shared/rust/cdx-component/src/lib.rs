//! Shared building blocks for the codexis-marketplace hosted-component plugins
//! (katastr, insolvence, znamky, codexis): daemon notify transport, layered
//! settings, and the CGI runtime their handlers share. Builds on `cdx-common`
//! for config/daemon-token/fs/paths. Plugin-specific logic (change detection,
//! API clients, the tracking lifecycle) stays in each plugin's own `*-core`.

pub mod error;
pub mod notify;
pub mod notify_settings;
pub mod cgi;

pub use error::{ComponentError, Result};

#[cfg(test)]
pub(crate) mod test_support {
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

    pub fn home_guard(home: &Path) -> HomeGuard {
        let guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("CODEXIS_PUBLIC_USER_HOME", home);
        std::env::set_var("CDX_USER_HOME", home);
        std::env::set_var("HOME", home);
        HomeGuard { _guard: guard }
    }
}

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

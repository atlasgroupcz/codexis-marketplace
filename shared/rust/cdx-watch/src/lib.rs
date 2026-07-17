//! The watcher domain shared by the CODEXIS "sledování" plugins (katastr,
//! insolvence, znamky, …): the change-log confirm mechanics and the
//! poll → persist → collect-result check orchestration over a keyed item store.
//!
//! A *capability* crate: it owns the parts of a watcher that are identical
//! across apps (the change-log confirm/count and the check loop), and leaves the
//! app-specific parts — fetching, diffing, the snapshot and change shapes — to
//! the plugin via the [`Watcher`](track::Watcher) trait. Independent of the
//! notify (`cdx-notify`) and CGI (`cdx-cgi`) capabilities; a plugin wires them
//! together (e.g. "check → if change → notify").

pub mod change;
pub mod error;
pub mod track;

pub use error::WatchError;

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

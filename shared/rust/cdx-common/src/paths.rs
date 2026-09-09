//! Resolve on-disk paths from the user home. Usable by any plugin (a CLI that
//! persists local state, or a hosted component). Everything derives from
//! `CODEXIS_PUBLIC_USER_HOME`, then the legacy `CDX_USER_HOME`, then `$HOME`
//! (for bare `cargo test`).

use std::env;
use std::path::{Path, PathBuf};

/// Root of the user's home on the sandboxed VM.
pub fn user_home() -> PathBuf {
    for var in ["CODEXIS_PUBLIC_USER_HOME", "CDX_USER_HOME"] {
        if let Ok(dir) = env::var(var) {
            if !dir.trim().is_empty() {
                return PathBuf::from(dir);
            }
        }
    }
    env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

/// `$HOME/.cdx/apps/<app_id>` - the app's secret dir, holding only `.env`.
/// The chat isolation mounts `~/.cdx/apps` read-only: plugins read the key from
/// here, and only the daemon writes it (through `cdxctl appenv`). Anything a
/// plugin writes itself belongs under [`app_state_dir`].
pub fn app_dir(app_id: &str) -> PathBuf {
    user_home().join(".cdx/apps").join(app_id)
}

/// `$HOME/.cdx/app-state/<app_id>` - parent dir for a plugin's per-user state
/// (settings, tracked items, caches). The chat isolation mounts
/// `~/.cdx/app-state` read-write.
pub fn app_state_dir(app_id: &str) -> PathBuf {
    user_home().join(".cdx/app-state").join(app_id)
}

/// True when `path` exists and is a regular file (not a dir / dangling symlink).
pub fn is_regular_file(path: &Path) -> bool {
    path.metadata().map(|m| m.is_file()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_dir_is_under_user_home_cdx_apps() {
        let _g = crate::test_support::home_guard(Path::new("/tmp/x-home"));
        assert_eq!(
            app_dir("katastr"),
            PathBuf::from("/tmp/x-home/.cdx/apps/katastr")
        );
    }

    #[test]
    fn app_state_dir_is_under_user_home_cdx_app_state() {
        let _g = crate::test_support::home_guard(Path::new("/tmp/x-home"));
        assert_eq!(
            app_state_dir("katastr"),
            PathBuf::from("/tmp/x-home/.cdx/app-state/katastr")
        );
    }

    #[test]
    fn user_home_prefers_codexis_public() {
        let _g = crate::test_support::home_guard(Path::new("/tmp/y-home"));
        assert_eq!(user_home(), PathBuf::from("/tmp/y-home"));
    }
}

//! Resolve on-disk paths for the katastr plugin. Home + app-dir + regular-file
//! checks come from `cdx_common::paths`; the proceeding-specific layout stays here.

use std::path::PathBuf;

pub use cdx_common::paths::{is_regular_file, user_home};

/// `$HOME/.cdx/apps/katastr`.
pub fn app_dir() -> PathBuf {
    cdx_common::paths::app_dir("katastr")
}

pub fn rizeni_dir() -> PathBuf {
    app_dir().join("rizeni")
}

pub fn state_path(cislo_rizeni: &str) -> PathBuf {
    rizeni_dir().join(dir_name(cislo_rizeni)).join("state.json")
}

pub fn dir_name(cislo_rizeni: &str) -> String {
    cislo_rizeni.replace('/', "_")
}

pub fn env_file() -> PathBuf {
    app_dir().join(".env")
}

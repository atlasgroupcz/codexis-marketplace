//! Resolve on-disk paths for the katastr plugin. The app-dir root comes from
//! `cdx_common::paths`; the proceeding slug rule and the `.env` path stay here.
//! The tracked-proceeding state files themselves live under `cdx-common`'s
//! `ItemStore` (see `state::store`), which puts them in
//! `~/.cdx/app-state/katastr/rizeni/`. Only the API key stays in
//! `~/.cdx/apps/katastr/.env`, which the chat isolation mounts read-only.

use std::path::PathBuf;

/// `$HOME/.cdx/apps/katastr` - the secret dir, holding only `.env`.
pub fn app_dir() -> PathBuf {
    cdx_common::paths::app_dir("katastr")
}

/// Filesystem-safe storage key for a proceeding: the canonical number with `/`
/// replaced by `_` (the `ItemStore` slug must be a single path component).
pub fn dir_name(cislo_rizeni: &str) -> String {
    cislo_rizeni.replace('/', "_")
}

pub fn env_file() -> PathBuf {
    app_dir().join(".env")
}

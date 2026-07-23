//! On-disk state representation for a tracked proceeding.
//!
//! The JSON schema here matches what the Python version writes - so an
//! existing user's `~/.cdx/apps/katastr/rizeni/<num>/state.json` loads
//! fine into Rust and vice versa. Keys are snake_case (Python style) for
//! our own fields; ČÚZK API shape leaks into `Operace` (mixed camelCase).

use serde::{Deserialize, Serialize};

use cdx_common::store::ItemStore;
use cdx_watch::change::ChangeRecord;

use crate::paths;

/// A single operation performed on the proceeding (e.g. "zapsáno", "plomba
/// nasazena"). Field names match the raw ČÚZK API response - we store them
/// verbatim instead of renaming to keep change detection byte-identical
/// with the Python implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operace {
    #[serde(default)]
    pub nazev: String,

    #[serde(rename = "datumProvedeni", default)]
    pub datum_provedeni: String,
}

/// A detected change between consecutive polls of ČÚZK.
///
/// `confirmed_on` is null until the user acknowledges the change in the UI -
/// that flips it to the timestamp of the confirmation so the badge disappears.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub detected_on: String,
    pub old_stav: String,
    pub new_stav: String,

    #[serde(default)]
    pub new_operations: Vec<Operace>,

    pub stav_uhrady_changed: bool,

    #[serde(default)]
    pub old_stav_uhrady: Option<String>,

    #[serde(default)]
    pub new_stav_uhrady: Option<String>,

    #[serde(default)]
    pub confirmed_on: Option<String>,
}

/// A katastr change is unconfirmed (unread) until `confirmed_on` is stamped.
/// Implementing `ChangeRecord` lets `cdx-watch` own the confirm/count logic
/// shared with the other watcher plugins.
impl ChangeRecord for Change {
    fn is_confirmed(&self) -> bool {
        self.confirmed_on.is_some()
    }
    fn set_confirmed(&mut self, timestamp: String) {
        self.confirmed_on = Some(timestamp);
    }
}

/// The full on-disk state for one tracked proceeding. Mirrors the Python
/// `state` dict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceedingState {
    pub cislo_rizeni: String,
    pub typ_rizeni: String,
    pub poradove_cislo: u32,
    pub rok: u32,
    pub kod_pracoviste: u32,

    #[serde(default)]
    pub api_id: Option<serde_json::Value>,

    #[serde(default)]
    pub label: String,

    pub added_on: String,

    #[serde(default)]
    pub last_check_at: String,

    #[serde(default)]
    pub stav: String,

    #[serde(default)]
    pub stav_uhrady: Option<String>,

    #[serde(default)]
    pub datum_prijeti: String,

    #[serde(default)]
    pub provedene_operace: Vec<Operace>,

    #[serde(default)]
    pub changes: Vec<Change>,
}

/// The per-item state store: `$HOME/.cdx/apps/katastr/rizeni/<slug>/state.json`,
/// one directory per tracked proceeding. Same layout the Python version used, so
/// existing state files load unchanged. `cdx-watch` drives its load/save/remove.
pub const APP_ID: &str = "katastr";
pub const COLLECTION: &str = "rizeni";

pub fn store() -> ItemStore {
    ItemStore::new(APP_ID, COLLECTION)
}

impl ProceedingState {
    /// Read state.json for a canonical proceeding number, or `None` when not
    /// tracked. Missing or corrupt files return `None` - matches Python's
    /// forgiving semantics where a half-written state file after a crash just
    /// drops out of `list_all` rather than failing the whole CGI.
    pub fn load(cislo_rizeni: &str) -> Option<Self> {
        store().load(&paths::dir_name(cislo_rizeni))
    }
}

/// Every readable tracked proceeding, in filesystem order. Corrupt state files
/// and trashed (`.deleted-*`) directories are skipped by the store - same
/// tolerance as the Python version.
pub fn list_all_raw() -> Vec<ProceedingState> {
    store().load_all()
}

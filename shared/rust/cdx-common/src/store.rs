//! Keyed, atomic, corruption-tolerant JSON item store.
//!
//! Every watcher plugin persists its tracked items the same way: one directory
//! per item under `$HOME/.cdx/apps/<app>/<collection>/<slug>/`, holding a single
//! pretty-printed JSON file (`state.json` by default). This is the port of the
//! Python watchers' `state.py`/`fs.py` layer - the mechanism is identical across
//! apps; only the stored value type differs, so it is generic over `T`.
//!
//! `slug` must be a single path component (the caller slugifies its id, e.g.
//! katastr replaces `/` with `_`; uuid-keyed apps pass the uuid verbatim).

use std::fs;
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

/// Substring marking a directory as trash left by an interrupted [`ItemStore::remove`].
const TRASH_MARKER: &str = ".deleted-";
const DEFAULT_FILE: &str = "state.json";

/// A per-app, per-collection store of JSON items keyed by slug.
pub struct ItemStore {
    app_id: String,
    collection: String,
    file_name: String,
}

impl ItemStore {
    /// Store items under `$HOME/.cdx/apps/<app_id>/<collection>/`, one
    /// `state.json` per item.
    pub fn new(app_id: impl Into<String>, collection: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            collection: collection.into(),
            file_name: DEFAULT_FILE.to_string(),
        }
    }

    /// Override the per-item file name (e.g. `"topic.json"`).
    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = file_name.into();
        self
    }

    /// `$HOME/.cdx/apps/<app>/<collection>`.
    pub fn dir(&self) -> PathBuf {
        crate::paths::app_dir(&self.app_id).join(&self.collection)
    }

    /// The directory holding one item.
    pub fn item_dir(&self, slug: &str) -> PathBuf {
        self.dir().join(slug)
    }

    /// The item's JSON file path.
    pub fn path(&self, slug: &str) -> PathBuf {
        self.item_dir(slug).join(&self.file_name)
    }

    /// True when the item's JSON file exists.
    pub fn exists(&self, slug: &str) -> bool {
        crate::paths::is_regular_file(&self.path(slug))
    }

    /// Load one item. `None` when missing or corrupt (a half-written file after
    /// a crash simply drops out instead of failing the whole run). `fs::read`
    /// already yields `Err` (→ `None`) for a missing path or a directory, so no
    /// separate existence check is needed.
    pub fn load<T: DeserializeOwned>(&self, slug: &str) -> Option<T> {
        let bytes = fs::read(self.path(slug)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Load every readable item in the collection, skipping corrupt files and
    /// `.deleted-*` trash left by an interrupted [`ItemStore::remove`]. Order is
    /// unspecified - callers sort.
    pub fn load_all<T: DeserializeOwned>(&self) -> Vec<T> {
        let Ok(entries) = fs::read_dir(self.dir()) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if name.contains(TRASH_MARKER) {
                continue;
            }
            // Reuse the DirEntry's path instead of re-deriving app_dir/collection
            // per item; read the file directly (missing/dir → skip).
            let path = entry.path().join(&self.file_name);
            if let Some(item) = fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<T>(&bytes).ok())
            {
                out.push(item);
            }
        }
        out
    }

    /// Persist one item atomically (tmp file + rename), pretty-printed.
    pub fn save<T: Serialize>(&self, slug: &str, value: &T) -> std::io::Result<()> {
        crate::fs::atomic_write_json(&self.path(slug), value)
    }

    /// Stop tracking an item. Rename-then-rmtree keeps a crash recoverable: if
    /// we crash between the rename and the rmtree, [`load_all`](Self::load_all)
    /// ignores the `.deleted-*` directory.
    pub fn remove(&self, slug: &str) -> std::io::Result<()> {
        let target = self.item_dir(slug);
        // Unique sibling name via the shared fs token (pid + counter), probing
        // for a free name - same scheme atomic_replace uses, no rand dependency.
        let trash = loop {
            let candidate =
                target.with_file_name(format!("{slug}{TRASH_MARKER}{}", crate::fs::unique_token()));
            if !candidate.exists() {
                break candidate;
            }
        };
        fs::rename(&target, &trash)?;
        let _ = fs::remove_dir_all(&trash); // best-effort: orphan cleanup can run later
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::home_guard;
    use serde::Deserialize;
    use tempfile::tempdir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Item {
        id: String,
        n: u32,
    }

    fn item(id: &str, n: u32) -> Item {
        Item {
            id: id.to_string(),
            n,
        }
    }

    #[test]
    fn save_then_load_roundtrips() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("testapp", "items");
        store.save("a", &item("a", 7)).unwrap();
        assert_eq!(store.load::<Item>("a"), Some(item("a", 7)));
    }

    #[test]
    fn load_missing_is_none() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("testapp", "items");
        assert_eq!(store.load::<Item>("nope"), None);
    }

    #[test]
    fn load_corrupt_is_none() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("testapp", "items");
        let path = store.path("bad");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{ not json").unwrap();
        assert_eq!(store.load::<Item>("bad"), None);
    }

    #[test]
    fn load_all_collects_and_skips_trash() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("testapp", "items");
        store.save("a", &item("a", 1)).unwrap();
        store.save("b", &item("b", 2)).unwrap();
        // Simulate an interrupted remove: a .deleted-* dir still holding a file.
        let trash = store.dir().join("a.deleted-deadbeef");
        fs::create_dir_all(&trash).unwrap();
        fs::write(trash.join("state.json"), br#"{"id":"a","n":1}"#).unwrap();
        let mut all = store.load_all::<Item>();
        all.sort_by(|x, y| x.id.cmp(&y.id));
        assert_eq!(all, vec![item("a", 1), item("b", 2)]);
    }

    #[test]
    fn load_all_empty_when_no_dir() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("testapp", "items");
        assert!(store.load_all::<Item>().is_empty());
    }

    #[test]
    fn remove_deletes_item() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("testapp", "items");
        store.save("a", &item("a", 1)).unwrap();
        assert!(store.exists("a"));
        store.remove("a").unwrap();
        assert!(!store.exists("a"));
        assert!(store.load_all::<Item>().is_empty());
    }

    #[test]
    fn custom_file_name_roundtrips() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("testapp", "topics").with_file_name("topic.json");
        store.save("t1", &item("t1", 9)).unwrap();
        assert!(store.path("t1").ends_with("topic.json"));
        assert_eq!(store.load::<Item>("t1"), Some(item("t1", 9)));
    }
}

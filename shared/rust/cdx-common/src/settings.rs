//! Layered per-app settings store: shipped defaults merged with a sparse user
//! override. The generic mechanism behind `cdx-notify`'s `NotifySettings` -
//! usable for any per-app settings block, not just `notify`.
//!
//! - defaults → `$HOME/.local/share/<app_id>/settings/defaults.json` (delivered
//!   by the install hook under the real `$HOME`, NOT the daemon-injected user
//!   home - the hook runs outside the chat VM context)
//! - override → `<user_home>/.cdx/apps/<app_id>/settings.json` (only changed keys)
//!
//! The effective config is computed at read time; nothing is merged back into
//! the user file.

use std::env;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

/// Recursively merge `override_` onto `base` (object values merge; everything
/// else replaces). Port of the Python cores' `deep_merge`.
pub fn deep_merge(base: Value, override_: &Value) -> Value {
    match (base, override_) {
        (Value::Object(mut base_map), Value::Object(over_map)) => {
            for (k, v) in over_map {
                let merged = match base_map.remove(k) {
                    Some(existing) if existing.is_object() && v.is_object() => {
                        deep_merge(existing, v)
                    }
                    _ => v.clone(),
                };
                base_map.insert(k.clone(), merged);
            }
            Value::Object(base_map)
        }
        (_, over) => over.clone(),
    }
}

/// Layered settings store for one app.
pub struct SettingsStore {
    app_id: String,
    schema_version: u64,
}

impl SettingsStore {
    pub fn new(app_id: impl Into<String>, schema_version: u64) -> Self {
        Self {
            app_id: app_id.into(),
            schema_version,
        }
    }

    /// Installed defaults, under the real `$HOME` (see module docs).
    pub fn defaults_path(&self) -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| "/".to_string());
        PathBuf::from(home)
            .join(".local/share")
            .join(&self.app_id)
            .join("settings/defaults.json")
    }

    /// The sparse user override file.
    pub fn user_path(&self) -> PathBuf {
        crate::paths::app_dir(&self.app_id).join("settings.json")
    }

    /// Effective settings: installed defaults (or `fallback` when the defaults
    /// file is missing/corrupt) with the user override deep-merged on top.
    pub fn merged_or(&self, fallback: Value) -> Value {
        let defaults = read_json_object(&self.defaults_path()).unwrap_or(fallback);
        let override_ =
            read_json_object(&self.user_path()).unwrap_or_else(|| Value::Object(Map::new()));
        deep_merge(defaults, &override_)
    }

    /// The raw sparse override map (empty when the file is missing/corrupt).
    pub fn read_override(&self) -> Map<String, Value> {
        match read_json_object(&self.user_path()) {
            Some(Value::Object(map)) => map,
            _ => Map::new(),
        }
    }

    /// Atomically persist the sparse override.
    pub fn write_override(&self, override_: &Map<String, Value>) -> io::Result<()> {
        crate::fs::atomic_write_json(&self.user_path(), &Value::Object(override_.clone()))
    }

    /// Create the user file carrying the schema version, or bump `version` when
    /// an existing file is below this store's version.
    pub fn ensure(&self) -> io::Result<()> {
        match read_json_object(&self.user_path()) {
            None => self.write_override(
                json!({ "version": self.schema_version })
                    .as_object()
                    .expect("literal object"),
            ),
            Some(Value::Object(mut existing)) => {
                let version = existing.get("version").and_then(Value::as_u64).unwrap_or(0);
                if version < self.schema_version {
                    existing.insert("version".to_string(), Value::from(self.schema_version));
                    self.write_override(&existing)
                } else {
                    Ok(())
                }
            }
            // read_json_object only yields objects; unreachable in practice.
            Some(_) => Ok(()),
        }
    }
}

fn read_json_object(path: &Path) -> Option<Value> {
    let bytes = std::fs::read(path).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    if value.is_object() {
        Some(value)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::home_guard;
    use tempfile::tempdir;

    #[test]
    fn deep_merge_overrides_leaf_and_keeps_siblings() {
        let base = json!({"notify": {"email": false, "inApp": true}, "version": 1});
        let over = json!({"notify": {"email": true}});
        let merged = deep_merge(base, &over);
        assert_eq!(merged["notify"]["email"], json!(true));
        assert_eq!(merged["notify"]["inApp"], json!(true));
        assert_eq!(merged["version"], json!(1));
    }

    #[test]
    fn merged_uses_fallback_and_applies_override() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = SettingsStore::new("testapp", 1);

        let mut override_ = Map::new();
        override_.insert("flag".to_string(), json!(true));
        store.write_override(&override_).unwrap();

        let merged = store.merged_or(json!({"flag": false, "kept": "x"}));
        assert_eq!(merged["flag"], json!(true));
        assert_eq!(merged["kept"], json!("x"));
        assert_eq!(store.read_override().get("flag"), Some(&json!(true)));
    }

    #[test]
    fn merged_prefers_installed_defaults_file_over_fallback() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = SettingsStore::new("testapp", 1);

        let defaults_path = store.defaults_path();
        std::fs::create_dir_all(defaults_path.parent().unwrap()).unwrap();
        std::fs::write(&defaults_path, br#"{"flag": "from-defaults"}"#).unwrap();

        let merged = store.merged_or(json!({"flag": "from-fallback"}));
        assert_eq!(merged["flag"], json!("from-defaults"));
    }

    #[test]
    fn ensure_creates_versioned_file_and_migrates_old_versions() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = SettingsStore::new("testapp", 3);

        store.ensure().unwrap();
        let raw: Value =
            serde_json::from_str(&std::fs::read_to_string(store.user_path()).unwrap()).unwrap();
        assert_eq!(raw["version"], json!(3));

        // An existing older file gets its version bumped, other keys kept.
        std::fs::write(store.user_path(), br#"{"version": 1, "keep": "me"}"#).unwrap();
        store.ensure().unwrap();
        let raw: Value =
            serde_json::from_str(&std::fs::read_to_string(store.user_path()).unwrap()).unwrap();
        assert_eq!(raw["version"], json!(3));
        assert_eq!(raw["keep"], json!("me"));
    }
}

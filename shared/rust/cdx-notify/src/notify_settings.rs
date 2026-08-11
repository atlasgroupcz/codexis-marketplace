//! Per-app notify settings: shipped defaults merged with a sparse user
//! override. Port of Python's `<app>_core.notify_settings`.
//!
//! - default → `~/.local/share/<app_id>/settings/defaults.json` (delivered by the
//!   install hook, sibling of the plugin's binaries)
//! - user    → `${CODEXIS_PUBLIC_USER_HOME|~}/.cdx/apps/<app_id>/settings.json`
//!   (only changed keys)
//!
//! The effective config is computed at read time; nothing is merged back into
//! the user file.

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::error::{NotifyError, Result};
use cdx_common::settings::SettingsStore;

pub const SCHEMA_VERSION: u64 = 1;

/// Effective notify block. Field names/casing match the JSON the FE reads.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NotifyConfig {
    pub email: bool,
    #[serde(rename = "inApp")]
    pub in_app: bool,
    pub recipients: Vec<String>,
}

fn fallback_defaults() -> Value {
    json!({
        "version": SCHEMA_VERSION,
        "notify": { "email": false, "inApp": true, "recipients": [] }
    })
}

// ── recipient hygiene ─────────────────────────────────────────────────────────

/// Trim, validate format, dedupe (case-insensitive, order-preserving). Errors
/// on an invalid address; empty entries are dropped. Port of Python's
/// `clean_recipients` (the same regex, hand-rolled to avoid a regex dep).
pub fn clean_recipients<I, S>(recipients: I) -> Result<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for r in recipients {
        let addr = r.as_ref().trim();
        if addr.is_empty() {
            continue;
        }
        if !is_valid_email(addr) {
            return Err(NotifyError::InvalidRecipient(format!(
                "Neplatná e-mailová adresa: {addr}"
            )));
        }
        let key = addr.to_lowercase();
        if seen.insert(key) {
            out.push(addr.to_string());
        }
    }
    Ok(out)
}

/// Matches Python's `^[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}$`.
fn is_valid_email(addr: &str) -> bool {
    let Some((local, domain)) = addr.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    let local_ok = local
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-'));
    if !local_ok {
        return false;
    }
    // Domain: one or more `[A-Za-z0-9.\-]`, then a `.` and a 2+ alpha TLD.
    let Some((host, tld)) = domain.rsplit_once('.') else {
        return false;
    };
    if host.is_empty() {
        return false;
    }
    let host_ok = host
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'));
    let tld_ok = tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic());
    host_ok && tld_ok
}

// ── store ────────────────────────────────────────────────────────────────────

/// Layered notify-settings store for one app: the notify-typed view over the
/// generic [`cdx_common::settings::SettingsStore`]. Defaults come from the
/// installed `~/.local/share/<app_id>/settings/defaults.json` (written by the
/// install hook, under the real `$HOME`); the sparse user override lives in
/// `<user_home>/.cdx/apps/<app_id>/settings.json`.
pub struct NotifySettings {
    store: SettingsStore,
}

impl NotifySettings {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            store: SettingsStore::new(app_id, SCHEMA_VERSION),
        }
    }

    #[cfg(test)]
    fn user_path(&self) -> std::path::PathBuf {
        self.store.user_path()
    }

    pub fn get_notify(&self) -> NotifyConfig {
        let merged = self.store.merged_or(fallback_defaults());
        let notify = merged.get("notify").cloned().unwrap_or(Value::Null);
        NotifyConfig {
            email: notify
                .get("email")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            in_app: notify.get("inApp").and_then(Value::as_bool).unwrap_or(true),
            recipients: notify
                .get("recipients")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    pub fn set_notify(
        &self,
        email: Option<bool>,
        in_app: Option<bool>,
        recipients: Option<Vec<String>>,
    ) -> Result<NotifyConfig> {
        let mut override_ = self.store.read_override();
        let mut notify = match override_.get("notify") {
            Some(Value::Object(m)) => m.clone(),
            _ => Map::new(),
        };
        if let Some(e) = email {
            notify.insert("email".into(), Value::Bool(e));
        }
        if let Some(a) = in_app {
            notify.insert("inApp".into(), Value::Bool(a));
        }
        if let Some(r) = recipients {
            let cleaned = clean_recipients(r)?;
            notify.insert(
                "recipients".into(),
                Value::Array(cleaned.into_iter().map(Value::String).collect()),
            );
        }
        override_.insert("notify".into(), Value::Object(notify));
        self.store.write_override(&override_)?;
        Ok(self.get_notify())
    }

    pub fn ensure_settings(&self) -> Result<()> {
        self.store.ensure().map_err(NotifyError::from)
    }
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_email_format() {
        assert!(is_valid_email("a.b_c%d+e@example.co.uk"));
        assert!(is_valid_email("x@y.io"));
        assert!(!is_valid_email("no-at-sign"));
        assert!(!is_valid_email("bad@domain"));
        assert!(!is_valid_email("bad@dom.a")); // TLD too short
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("sp ace@example.com"));
    }

    #[test]
    fn clean_recipients_dedupes_case_insensitive_preserving_order() {
        let out = clean_recipients(["A@x.com", " b@x.com ", "a@X.com", ""]).unwrap();
        assert_eq!(out, vec!["A@x.com".to_string(), "b@x.com".to_string()]);
    }

    #[test]
    fn clean_recipients_rejects_invalid() {
        assert!(clean_recipients(["not-an-email"]).is_err());
    }

    #[test]
    fn set_and_get_roundtrip_against_temp_home() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = cdx_common::test_support::home_guard(tmp.path());
        let store = NotifySettings::new("katastr");
        let cfg = store.get_notify();
        assert!(!cfg.email);
        assert!(cfg.in_app);

        let updated = store
            .set_notify(Some(true), None, Some(vec!["me@example.com".into()]))
            .unwrap();
        assert!(updated.email);
        assert!(updated.in_app);
        assert_eq!(updated.recipients, vec!["me@example.com".to_string()]);

        assert_eq!(store.get_notify(), updated);
    }

    #[test]
    fn ensure_settings_creates_versioned_file() {
        let tmp = tempfile::tempdir().unwrap();
        let _g = cdx_common::test_support::home_guard(tmp.path());
        let store = NotifySettings::new("katastr");
        store.ensure_settings().unwrap();
        let raw = std::fs::read_to_string(store.user_path()).unwrap();
        assert!(raw.contains("\"version\""));
    }
}

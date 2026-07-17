//! The check orchestration shared by the watcher plugins. Each check run is the
//! same shape everywhere: load the tracked item, poll the external source, fold
//! the result into the item, persist it, and record a per-item result — with one
//! item's failure isolated from the rest. Only the polling/diffing and the
//! snapshot/change shapes differ per app, expressed through [`Watcher`].

use std::fmt;

use cdx_common::store::ItemStore;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::WatchError;

/// A plugin's change-detection strategy. The crate owns the load/persist/loop;
/// the plugin owns fetching, diffing, and its snapshot/change shapes.
pub trait Watcher {
    /// The persisted per-item state.
    type Snapshot: Serialize + DeserializeOwned;
    /// One detected change.
    type Change;
    /// The plugin's fetch/diff error.
    type Error: fmt::Display;

    /// Poll the external source and fold the result into `prev`. Returns the
    /// snapshot to persist plus any newly detected change (`None` = a quiet run;
    /// the snapshot is still saved so bookkeeping like last-checked persists).
    ///
    /// The impl owns fetching, diffing, updating snapshot fields, appending to
    /// its own change log, and stamping timestamps.
    fn poll(
        &self,
        prev: Self::Snapshot,
    ) -> Result<(Self::Snapshot, Option<Self::Change>), Self::Error>;

    /// The item's storage slug (its key in the [`ItemStore`]).
    fn slug(&self, snapshot: &Self::Snapshot) -> String;

    /// A human-readable label for the item, for the result row.
    fn display_name(&self, snapshot: &Self::Snapshot) -> String;
}

/// Per-item outcome of a check run. Serializes for the CGI's JSON response.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult<C> {
    pub slug: String,
    pub display_name: String,
    pub ok: bool,
    pub change: Option<C>,
    pub error: Option<String>,
}

impl<C> CheckResult<C> {
    /// True when the check completed and carries a newly detected change.
    pub fn has_change(&self) -> bool {
        self.ok && self.change.is_some()
    }
}

/// Check every tracked item, capturing per-item errors so one failure never
/// aborts the run.
pub fn check_all<W: Watcher>(store: &ItemStore, watcher: &W) -> Vec<CheckResult<W::Change>> {
    store
        .load_all::<W::Snapshot>()
        .into_iter()
        .map(|snapshot| check_snapshot(store, watcher, snapshot))
        .collect()
}

/// Check one tracked item by slug. `Err(NotTracked)` when the slug isn't stored.
pub fn check_one<W: Watcher>(
    store: &ItemStore,
    watcher: &W,
    slug: &str,
) -> Result<CheckResult<W::Change>, WatchError> {
    let snapshot = store
        .load::<W::Snapshot>(slug)
        .ok_or_else(|| WatchError::NotTracked(slug.to_string()))?;
    Ok(check_snapshot(store, watcher, snapshot))
}

/// Run the watcher over one already-loaded snapshot and persist the result.
fn check_snapshot<W: Watcher>(
    store: &ItemStore,
    watcher: &W,
    snapshot: W::Snapshot,
) -> CheckResult<W::Change> {
    let slug = watcher.slug(&snapshot);
    let display_name = watcher.display_name(&snapshot);
    match watcher.poll(snapshot) {
        Ok((updated, change)) => match store.save(&slug, &updated) {
            Ok(()) => CheckResult {
                slug,
                display_name,
                ok: true,
                change,
                error: None,
            },
            Err(e) => CheckResult {
                slug,
                display_name,
                ok: false,
                change: None,
                error: Some(e.to_string()),
            },
        },
        Err(e) => CheckResult {
            slug,
            display_name,
            ok: false,
            change: None,
            error: Some(e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cdx_common::test_support::home_guard;
    use serde::Deserialize;
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Clone)]
    struct Snap {
        id: String,
        value: u32,
        checks: u32,
    }

    // Bumps `checks` each poll; reports a change the first time value is 0.
    // Errors for the poisoned id "boom".
    struct FakeWatcher;
    impl Watcher for FakeWatcher {
        type Snapshot = Snap;
        type Change = String;
        type Error = String;

        fn poll(&self, mut prev: Snap) -> Result<(Snap, Option<String>), String> {
            if prev.id == "boom" {
                return Err("kaboom".into());
            }
            prev.checks += 1;
            let change = if prev.value == 0 {
                prev.value = 9;
                Some("went to 9".to_string())
            } else {
                None
            };
            Ok((prev, change))
        }
        fn slug(&self, s: &Snap) -> String {
            s.id.clone()
        }
        fn display_name(&self, s: &Snap) -> String {
            format!("item {}", s.id)
        }
    }

    fn seed(store: &ItemStore, id: &str, value: u32) {
        store
            .save(id, &Snap { id: id.into(), value, checks: 0 })
            .unwrap();
    }

    #[test]
    fn check_one_detects_change_and_persists() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("t", "items");
        seed(&store, "a", 0);

        let r = check_one(&store, &FakeWatcher, "a").unwrap();
        assert!(r.ok);
        assert!(r.has_change());
        assert_eq!(r.change.as_deref(), Some("went to 9"));
        assert_eq!(r.display_name, "item a");

        let saved: Snap = store.load("a").unwrap();
        assert_eq!(saved.value, 9);
        assert_eq!(saved.checks, 1);
    }

    #[test]
    fn check_one_quiet_run_still_persists_bookkeeping() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("t", "items");
        seed(&store, "a", 5); // value != 0 -> no change

        let r = check_one(&store, &FakeWatcher, "a").unwrap();
        assert!(r.ok);
        assert!(r.change.is_none());
        let saved: Snap = store.load("a").unwrap();
        assert_eq!(saved.checks, 1);
    }

    #[test]
    fn check_one_not_tracked_errors() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("t", "items");
        assert!(matches!(
            check_one(&store, &FakeWatcher, "nope"),
            Err(WatchError::NotTracked(_))
        ));
    }

    #[test]
    fn check_one_poll_error_is_captured_and_not_saved() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("t", "items");
        seed(&store, "boom", 0);

        let r = check_one(&store, &FakeWatcher, "boom").unwrap();
        assert!(!r.ok);
        assert_eq!(r.error.as_deref(), Some("kaboom"));
        let saved: Snap = store.load("boom").unwrap();
        assert_eq!(saved.checks, 0); // not saved on error
    }

    #[test]
    fn check_all_runs_every_item_and_isolates_failures() {
        let home = tempdir().unwrap();
        let _g = home_guard(home.path());
        let store = ItemStore::new("t", "items");
        seed(&store, "a", 0);
        seed(&store, "boom", 0);

        let mut results = check_all(&store, &FakeWatcher);
        results.sort_by(|x, y| x.slug.cmp(&y.slug));
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].slug, "a");
        assert!(results[0].ok);
        assert_eq!(results[1].slug, "boom");
        assert!(!results[1].ok);
    }
}

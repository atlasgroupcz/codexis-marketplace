//! Watching a subject's Sbírka listin for newly filed documents.
//!
//! The diff is a set difference on the full číslo listiny (`B 8573/SL95/MSPH`).
//! That key is safe to diff on: it is unique per subject and rises monotonically
//! with the filing date, and a re-filed document takes a new number rather than
//! overwriting the old one - ČEZ's 2023 účetní závěrka sits at both SL288 and
//! SL289, same file, two rows. The collection is append-only in practice, so
//! `Change` only reports additions; a listina that disappears is logged as an
//! anomaly rather than silently absorbed, because we have never observed one.

use std::collections::BTreeMap;

use cdx_common::store::ItemStore;
use cdx_common::time::now_utc;
use cdx_watch::change::{unconfirmed_count, ChangeRecord};
use cdx_watch::track::Watcher;
use serde::{Deserialize, Serialize};

use crate::api;
use crate::http::PortalClient;

pub const APP_ID: &str = "sbirka-listin";
pub const COLLECTION: &str = "subjekty";

pub fn store() -> ItemStore {
    ItemStore::new(APP_ID, COLLECTION)
}

/// One row of the tracked-subjects overview - the shape both `sledovani list`
/// (CLI) and the dashboard `overview` (CGI) return, so the two can never drift.
#[derive(Debug, Clone, Serialize)]
pub struct OverviewRow {
    pub ico: String,
    pub nazev: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spisovaznacka: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidlo: Option<String>,
    pub listin: usize,
    pub nepotvrzenych_zmen: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_check_at: Option<String>,
}

impl OverviewRow {
    /// Project one snapshot into an overview row, given its already-counted
    /// unconfirmed changes.
    pub fn from_snapshot(s: Snapshot, nepotvrzenych_zmen: usize) -> Self {
        Self {
            ico: s.ico,
            nazev: s.nazev,
            spisovaznacka: s.spisovaznacka,
            sidlo: s.sidlo,
            listin: s.listiny.len(),
            nepotvrzenych_zmen,
            last_check_at: s.last_check_at,
        }
    }
}

/// Every tracked subject as an overview row, subjects with unread changes first,
/// then most recently changed.
pub fn overview_rows(store: &ItemStore) -> Vec<OverviewRow> {
    let mut ranked: Vec<(usize, String, Snapshot)> = store
        .load_all::<Snapshot>()
        .into_iter()
        .map(|s| {
            let unconfirmed = unconfirmed_count(&s.changes);
            let latest = s
                .changes
                .iter()
                .map(|c| c.detected_on.as_str())
                .max()
                .unwrap_or("")
                .to_string();
            (unconfirmed, latest, s)
        })
        .collect();
    ranked.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    ranked
        .into_iter()
        .map(|(unconfirmed, _, s)| OverviewRow::from_snapshot(s, unconfirmed))
        .collect()
}

/// One row of the persisted document list - the diff baseline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ListinaSnapshot {
    pub typ: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zalozeno: Option<String>,
}

/// The persisted state of one watched subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub ico: String,
    pub nazev: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spisovaznacka: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidlo: Option<String>,
    /// Keyed by the full číslo listiny. BTreeMap so the persisted file is stable.
    #[serde(default)]
    pub listiny: BTreeMap<String, ListinaSnapshot>,
    #[serde(default)]
    pub changes: Vec<Change>,
    pub added_on: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_check_at: Option<String>,
}

/// One check run that found something. Mirrors the Python watchers: a single
/// record per run, carrying every item the run discovered, unread until the user
/// acknowledges it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub detected_on: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nove_listiny: Vec<NovaListina>,
    /// Never observed in the wild; recorded rather than dropped so we find out
    /// if the collection turns out not to be append-only after all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zmizele_listiny: Vec<String>,
    pub confirmed_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovaListina {
    pub cislo: String,
    pub typ: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zalozeno: Option<String>,
}

impl ChangeRecord for Change {
    fn is_confirmed(&self) -> bool {
        self.confirmed_on.is_some()
    }

    fn set_confirmed(&mut self, timestamp: String) {
        self.confirmed_on = Some(timestamp);
    }
}

/// Build the baseline map from a fetched detail.
pub fn listiny_map(detail: &api::Detail) -> BTreeMap<String, ListinaSnapshot> {
    detail
        .listiny
        .iter()
        .map(|l| {
            (
                l.cislo.clone(),
                ListinaSnapshot {
                    typ: l.typ.clone(),
                    zalozeno: l.zalozeno.clone(),
                },
            )
        })
        .collect()
}

/// Seed a snapshot at add-time: the current list becomes the baseline, so the
/// first check reports only what arrives afterwards.
pub fn seed(detail: &api::Detail) -> Snapshot {
    Snapshot {
        ico: detail.ico.clone(),
        nazev: detail.nazev.clone(),
        spisovaznacka: detail.spisovaznacka.clone(),
        sidlo: detail.sidlo.clone(),
        listiny: listiny_map(detail),
        changes: Vec::new(),
        added_on: now_utc(),
        last_check_at: Some(now_utc()),
    }
}

pub struct SbirkaListinWatcher<'a> {
    pub client: &'a PortalClient,
}

impl Watcher for SbirkaListinWatcher<'_> {
    type Snapshot = Snapshot;
    type Change = Change;
    type Error = anyhow::Error;

    fn poll(&self, mut prev: Snapshot) -> anyhow::Result<(Snapshot, Option<Change>)> {
        let detail = api::fetch_detail(self.client, &prev.ico)?;
        let fresh = listiny_map(&detail);

        let nove_listiny: Vec<NovaListina> = detail
            .listiny
            .iter()
            .filter(|l| !prev.listiny.contains_key(&l.cislo))
            .map(|l| NovaListina {
                cislo: l.cislo.clone(),
                typ: l.typ.clone(),
                zalozeno: l.zalozeno.clone(),
            })
            .collect();
        let zmizele_listiny: Vec<String> = prev
            .listiny
            .keys()
            .filter(|key| !fresh.contains_key(*key))
            .cloned()
            .collect();

        // Refresh identity but never wipe it: a sparse response must not blank
        // out a name we already have.
        if !detail.nazev.is_empty() {
            prev.nazev = detail.nazev.clone();
        }
        if detail.spisovaznacka.is_some() {
            prev.spisovaznacka = detail.spisovaznacka.clone();
        }
        if detail.sidlo.is_some() {
            prev.sidlo = detail.sidlo.clone();
        }
        prev.listiny = fresh;
        prev.last_check_at = Some(now_utc());

        if nove_listiny.is_empty() && zmizele_listiny.is_empty() {
            return Ok((prev, None));
        }
        let change = Change {
            detected_on: now_utc(),
            nove_listiny,
            zmizele_listiny,
            confirmed_on: None,
        };
        prev.changes.push(change.clone());
        Ok((prev, Some(change)))
    }

    fn slug(&self, snapshot: &Snapshot) -> String {
        // IČO is already a single path component - eight digits, nothing to escape.
        snapshot.ico.clone()
    }

    fn display_name(&self, snapshot: &Snapshot) -> String {
        if snapshot.nazev.is_empty() {
            snapshot.ico.clone()
        } else {
            snapshot.nazev.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detail_with(rows: &[(&str, &str)]) -> api::Detail {
        let listiny: Vec<api::Listina> = rows
            .iter()
            .map(|(cislo, typ)| api::Listina {
                cislo: cislo.to_string(),
                typ: typ.to_string(),
                vznik: None,
                doslo: None,
                zalozeno: Some("1.1.2026".into()),
                stran: None,
                dokument: "1".into(),
                spis: "1".into(),
            })
            .collect();
        api::Detail {
            ico: "27082440".into(),
            nazev: "Alza.cz a.s.".into(),
            spisovaznacka: Some("B 8573/MSPH".into()),
            sidlo: None,
            subjekt_id: "701502".into(),
            pocet_listin: listiny.len(),
            listiny,
        }
    }

    #[test]
    fn seed_takes_the_current_list_as_baseline() {
        let snapshot = seed(&detail_with(&[("B 8573/SL1/MSPH", "stanovy")]));
        assert_eq!(snapshot.ico, "27082440");
        assert_eq!(snapshot.nazev, "Alza.cz a.s.");
        assert_eq!(snapshot.listiny.len(), 1);
        assert!(snapshot.changes.is_empty());
    }

    #[test]
    fn diff_reports_only_keys_absent_from_the_baseline() {
        let before = listiny_map(&detail_with(&[("B 8573/SL1/MSPH", "stanovy")]));
        let after = listiny_map(&detail_with(&[
            ("B 8573/SL1/MSPH", "stanovy"),
            ("B 8573/SL2/MSPH", "účetní závěrka [2025]"),
        ]));
        let nove: Vec<&String> = after.keys().filter(|k| !before.contains_key(*k)).collect();
        assert_eq!(nove, vec!["B 8573/SL2/MSPH"]);
    }

    #[test]
    fn slug_is_the_ico_verbatim() {
        let watcher = SbirkaListinWatcher {
            client: &PortalClient::new().unwrap(),
        };
        assert_eq!(watcher.slug(&seed(&detail_with(&[]))), "27082440");
    }

    #[test]
    fn display_name_falls_back_to_ico_when_the_name_is_unknown() {
        let watcher = SbirkaListinWatcher {
            client: &PortalClient::new().unwrap(),
        };
        let mut snapshot = seed(&detail_with(&[]));
        snapshot.nazev = String::new();
        assert_eq!(watcher.display_name(&snapshot), "27082440");
    }

    #[test]
    fn change_confirm_flips_is_confirmed() {
        let mut change = Change {
            detected_on: "2026-07-17T07:00:00Z".into(),
            nove_listiny: vec![],
            zmizele_listiny: vec![],
            confirmed_on: None,
        };
        assert!(!change.is_confirmed());
        change.set_confirmed("2026-07-17T08:00:00Z".into());
        assert!(change.is_confirmed());
    }
}

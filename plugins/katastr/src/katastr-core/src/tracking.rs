//! Tracking logic - parse proceeding numbers, fetch from ČÚZK, detect
//! changes, and manage on-disk state. Port of Python's `katastr_core.tracking`.
//!
//! All business logic the CLI and CGI share lives here: add / remove / check /
//! confirm / set-label plus the presentation projection (`OverviewEntry`).

use std::collections::HashSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cdx_watch::change::{confirm_changes, unconfirmed_count};
use cdx_watch::track::{self, Watcher};

use crate::error::{KatastrError, Result};
use crate::state::{self, Change, Operace, ProceedingState};

/// Canonicalized form of a proceeding number, e.g. `V-123/2026-701`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceedingNumber {
    pub typ_rizeni: String,
    pub poradove_cislo: u32,
    pub rok: u32,
    pub kod_pracoviste: u32,
}

impl ProceedingNumber {
    /// Canonical `TYP-CISLO/ROK-PRACOVISTE` rendering. Used as both the
    /// display form and the filesystem-safe key (slash gets replaced with
    /// underscore at path construction).
    pub fn canonical(&self) -> String {
        format!(
            "{}-{}/{}-{}",
            self.typ_rizeni, self.poradove_cislo, self.rok, self.kod_pracoviste
        )
    }
}

/// Parse strings like `V-123/2026-701` or `v-123/2026-701` (case-insensitive,
/// surrounding whitespace tolerated). Errors point the user at the expected
/// format so the UI error message is helpful on its own.
///
/// The grammar is hand-rolled instead of a regex crate dependency - 80% less
/// binary bloat, and the pattern is fixed forever by the ČÚZK schema.
pub fn parse_proceeding_number(input: &str) -> Result<ProceedingNumber> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(KatastrError::InvalidProceedingNumber(
            "Prázdné číslo řízení.".into(),
        ));
    }

    let invalid = || {
        KatastrError::InvalidProceedingNumber(format!(
            "Neplatný formát čísla řízení: {input}. \
             Očekávaný formát: TYP-ČÍSLO/ROK-KÓD (např. V-123/2026-701)."
        ))
    };

    // Split on the required structural separators: `-`, `/`, `-`.
    let (typ, rest) = trimmed.split_once('-').ok_or_else(invalid)?;
    let (cislo, rest) = rest.split_once('/').ok_or_else(invalid)?;
    let (rok, pracoviste) = rest.split_once('-').ok_or_else(invalid)?;

    if typ.is_empty() || !typ.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(invalid());
    }
    let poradove_cislo: u32 = cislo.parse().map_err(|_| invalid())?;
    if rok.len() != 4 {
        return Err(invalid());
    }
    let rok: u32 = rok.parse().map_err(|_| invalid())?;
    let kod_pracoviste: u32 = pracoviste.parse().map_err(|_| invalid())?;

    Ok(ProceedingNumber {
        typ_rizeni: typ.to_ascii_uppercase(),
        poradove_cislo,
        rok,
        kod_pracoviste,
    })
}

/// Parse + canonicalize in one step. Useful everywhere the CLI/CGI accepts
/// user input and needs the normalized form for filesystem lookups.
pub fn normalize_number(input: &str) -> Result<String> {
    Ok(parse_proceeding_number(input)?.canonical())
}

/// ISO-8601 UTC timestamp with second precision and `Z` suffix.
/// Matches the format `datetime.now(UTC).isoformat(timespec="seconds").replace("+00:00", "Z")`
/// produces in Python, so mixed Python/Rust state files remain comparable.
pub fn now_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ── presentation projection ───────────────────────────────────────────────────

/// UI-facing summary of a tracked proceeding. Keep field names matching
/// `state_to_overview_entry` in Python so the TS schemas don't need touching.
#[derive(Debug, Clone, Serialize)]
pub struct OverviewEntry {
    pub uuid: String,
    pub cislo_rizeni: String,
    pub typ_rizeni: String,
    pub label: String,
    pub stav: String,
    pub stav_uhrady: Option<String>,
    pub stav_uhrady_label: Option<String>,
    pub datum_prijeti: String,
    pub added_on: String,
    pub last_check_at: String,
    pub last_op_date: Option<String>,
    pub operace_count: usize,
    pub changes_count: usize,
    pub unconfirmed_count: usize,
    pub provedene_operace: Vec<OverviewOperace>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OverviewOperace {
    pub nazev: String,
    #[serde(rename = "datumProvedeni")]
    pub datum_provedeni: String,
}

impl OverviewEntry {
    pub fn from_state(state: &ProceedingState) -> Self {
        // ČÚZK returns `provedeneOperace` already oldest→newest (verified live
        // 2026-07). We pass it through unchanged, so the newest operation is the
        // last element - matching Python's `_ordered_operace` + `ops[-1]`.
        let last_op_date = state
            .provedene_operace
            .last()
            .map(|op| op.datum_provedeni.clone())
            .filter(|d| !d.is_empty());

        Self {
            uuid: state.uuid.clone(),
            cislo_rizeni: state.cislo_rizeni.clone(),
            typ_rizeni: state.typ_rizeni.clone(),
            label: state.label.clone(),
            stav: state.stav.clone(),
            stav_uhrady: state.stav_uhrady.clone(),
            stav_uhrady_label: state
                .stav_uhrady
                .as_deref()
                .and_then(uhrada_label)
                .map(String::from),
            datum_prijeti: state.datum_prijeti.clone(),
            added_on: state.added_on.clone(),
            last_check_at: state.last_check_at.clone(),
            last_op_date,
            operace_count: state.provedene_operace.len(),
            changes_count: state.changes.len(),
            unconfirmed_count: unconfirmed_count(&state.changes),
            provedene_operace: state
                .provedene_operace
                .iter()
                .map(|op| OverviewOperace {
                    nazev: op.nazev.clone(),
                    datum_provedeni: op.datum_provedeni.clone(),
                })
                .collect(),
        }
    }
}

/// Map the single-letter ČÚZK payment code (`U`/`N`/`O`) to its display label.
/// Returns `None` for unknown codes - UI falls back to showing the raw code.
pub fn uhrada_label(code: &str) -> Option<&'static str> {
    match code {
        "U" => Some("Uhrazeno"),
        "N" => Some("Neuhrazeno"),
        "O" => Some("Osvobozeno"),
        _ => None,
    }
}

fn uhrada_display(code: Option<&str>) -> String {
    code.and_then(uhrada_label)
        .map(String::from)
        .or_else(|| code.map(String::from))
        .unwrap_or_else(|| "-".to_string())
}

/// Human-readable, unindented lines describing one detected change.
///
/// Single source shared by the CLI check output and the e-mail summary, so a
/// change (stav / new operations / úhrada) renders the same in both channels.
/// Each caller adds its own prefix/indentation. Port of Python's
/// `tracking.describe_change`.
pub fn describe_change(change: &Change) -> Vec<String> {
    let mut lines = Vec::new();
    if change.old_stav != change.new_stav {
        lines.push(format!("Stav: {} → {}", change.old_stav, change.new_stav));
    }
    for op in &change.new_operations {
        lines.push(format!(
            "Nová operace: {} ({})",
            if op.nazev.is_empty() { "?" } else { &op.nazev },
            op.datum_provedeni
        ));
    }
    if change.stav_uhrady_changed {
        lines.push(format!(
            "Úhrada: {} → {}",
            uhrada_display(change.old_stav_uhrady.as_deref()),
            uhrada_display(change.new_stav_uhrady.as_deref())
        ));
    }
    lines
}

// ── listing ───────────────────────────────────────────────────────────────────

/// Return all tracked proceedings, newest added first - purely by date added.
/// Confirming a change (or a newly detected one) never reorders; order is
/// stable once a proceeding is added. `uuid` is a deterministic tiebreaker when
/// two share an `added_on` second. Matches Python's `list_all`.
pub fn list_all() -> Vec<ProceedingState> {
    let mut all = state::list_all_raw();
    all.sort_by(|a, b| {
        (b.added_on.as_str(), b.uuid.as_str()).cmp(&(a.added_on.as_str(), a.uuid.as_str()))
    });
    all
}

/// Same as `list_all`, but returning the overview projection used by the
/// CGI's JSON response for the overview endpoint.
pub fn list_overview() -> Vec<OverviewEntry> {
    list_all().iter().map(OverviewEntry::from_state).collect()
}

// ── ČÚZK fetch ───────────────────────────────────────────────────────────────

/// Fetch a proceeding via the search endpoint. Returns `Ok(None)` when the API
/// replied successfully but with an empty `data` array - treat as "not found".
pub fn fetch_from_cuzk(parsed: &ProceedingNumber) -> Result<Option<serde_json::Value>> {
    let path = format!(
        "/api/v1/Rizeni/Vyhledani?TypRizeni={}&Cislo={}&Rok={}&KodPracoviste={}",
        parsed.typ_rizeni, parsed.poradove_cislo, parsed.rok, parsed.kod_pracoviste
    );
    let result = crate::api_client::get(&path)?;
    let data = result.get("data").and_then(|d| d.as_array());
    Ok(data.and_then(|arr| arr.first().cloned()))
}

/// Fetch a proceeding directly by its ČÚZK id. Unlike `Rizeni/Vyhledani` (whose
/// `data` is a list), `Rizeni/{id}` wraps a single object under `data` - same
/// envelope, no `[0]` unwrap. This is the fast path used by `check` once an
/// `api_id` has been learned. Port of Python's `fetch_by_id`.
pub fn fetch_by_id(api_id: &serde_json::Value) -> Result<Option<serde_json::Value>> {
    let segment = api_id_segment(api_id);
    let result = crate::api_client::get(&format!("/api/v1/Rizeni/{segment}"))?;
    match result.get("data") {
        Some(v) if !v.is_null() => Ok(Some(v.clone())),
        _ => Ok(None),
    }
}

/// Render an `api_id` scalar as a URL path segment without JSON quoting
/// (Python interpolates the raw int/str). Objects/arrays never occur here.
fn api_id_segment(api_id: &serde_json::Value) -> String {
    match api_id {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn operace_from_data(data: &serde_json::Value) -> Vec<Operace> {
    data.get("provedeneOperace")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|op| Operace {
                    nazev: op
                        .get("nazev")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    datum_provedeni: op
                        .get("datumProvedeni")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn str_field(data: &serde_json::Value, key: &str) -> String {
    data.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

// ── change detection ────────────────────────────────────────────────────────

/// Compare previous state against a fresh API payload and return a
/// [`Change`] record if anything interesting differs. "Interesting" = stav
/// string changed, a new operation appeared, or the payment status flipped.
/// Identical to Python's `_detect_changes`.
fn detect_changes(old_state: &ProceedingState, new_data: &serde_json::Value) -> Option<Change> {
    let old_stav = old_state.stav.clone();
    let new_stav = str_field(new_data, "stav");

    let old_uhrada = old_state.stav_uhrady.clone();
    let new_uhrada = new_data
        .get("stavUhrady")
        .and_then(|v| v.as_str())
        .map(String::from);

    let old_ops: HashSet<(String, String)> = old_state
        .provedene_operace
        .iter()
        .map(|op| (op.nazev.clone(), op.datum_provedeni.clone()))
        .collect();

    let new_operations: Vec<Operace> = operace_from_data(new_data)
        .into_iter()
        .filter(|op| !old_ops.contains(&(op.nazev.clone(), op.datum_provedeni.clone())))
        .collect();

    let stav_changed = new_stav != old_stav;
    let uhrada_changed = new_uhrada != old_uhrada;

    if !stav_changed && new_operations.is_empty() && !uhrada_changed {
        return None;
    }

    Some(Change {
        detected_on: now_utc(),
        old_stav,
        new_stav,
        new_operations,
        stav_uhrady_changed: uhrada_changed,
        old_stav_uhrady: old_uhrada,
        new_stav_uhrady: new_uhrada,
        confirmed_on: None,
    })
}

// ── high-level operations ────────────────────────────────────────────────────

/// Start tracking a proceeding. Fetches current state from ČÚZK as the
/// baseline and writes `state.json` atomically.
pub fn add(cislo_rizeni: &str, label: &str) -> Result<ProceedingState> {
    let parsed = parse_proceeding_number(cislo_rizeni)?;
    let canonical = parsed.canonical();

    if ProceedingState::load(&canonical).is_some() {
        return Err(KatastrError::ProceedingAlreadyTracked(format!(
            "Řízení {canonical} je již sledováno."
        )));
    }

    let data = fetch_from_cuzk(&parsed)?.ok_or_else(|| {
        KatastrError::ProceedingNotFound(format!("Řízení {canonical} nebylo nalezeno v ČÚZK."))
    })?;

    let timestamp = now_utc();
    let state = ProceedingState {
        uuid: Uuid::new_v4().to_string(),
        cislo_rizeni: canonical.clone(),
        typ_rizeni: parsed.typ_rizeni.clone(),
        poradove_cislo: parsed.poradove_cislo,
        rok: parsed.rok,
        kod_pracoviste: parsed.kod_pracoviste,
        api_id: data.get("id").cloned(),
        label: label.to_string(),
        added_on: timestamp.clone(),
        last_check_at: timestamp,
        stav: str_field(&data, "stav"),
        stav_uhrady: data.get("stavUhrady").and_then(|v| v.as_str()).map(String::from),
        datum_prijeti: str_field(&data, "datumPrijeti"),
        provedene_operace: operace_from_data(&data),
        changes: Vec::new(),
    };
    save_state(&canonical, &state)?;
    Ok(state)
}

/// Update the user-defined label. Returns the updated state. Errors when the
/// proceeding isn't tracked.
pub fn set_label(cislo_rizeni: &str, label: &str) -> Result<ProceedingState> {
    let canonical = normalize_number(cislo_rizeni)?;
    let mut state = ProceedingState::load(&canonical).ok_or_else(|| {
        KatastrError::ProceedingNotTracked(format!("Řízení {canonical} není sledováno."))
    })?;
    state.label = label.to_string();
    save_state(&canonical, &state)?;
    Ok(state)
}

/// Return full state of a tracked proceeding. Errors when not tracked.
pub fn show(cislo_rizeni: &str) -> Result<ProceedingState> {
    let canonical = normalize_number(cislo_rizeni)?;
    ProceedingState::load(&canonical).ok_or_else(|| {
        KatastrError::ProceedingNotTracked(format!(
            "Řízení {canonical} není sledováno. \
             Použij `katastr-cli tracking add` pro přidání nebo \
             `katastr-cli api get` pro one-shot lookup."
        ))
    })
}

/// Stop tracking a proceeding. Rename-then-rmtree keeps a crash recoverable:
/// if we crash between the rename and the rmtree, the next `list_all()` simply
/// ignores the now-renamed directory.
pub fn remove(cislo_rizeni: &str) -> Result<()> {
    let canonical = normalize_number(cislo_rizeni)?;
    let slug = crate::paths::dir_name(&canonical);
    if !state::store().exists(&slug) {
        return Err(KatastrError::ProceedingNotTracked(format!(
            "Řízení {canonical} není sledováno."
        )));
    }
    // The store's remove is the same crash-safe rename-then-rmtree: a crash
    // between the rename and the rmtree just leaves a `.deleted-*` dir that
    // `load_all` ignores.
    state::store().remove(&slug)?;
    Ok(())
}

/// The katastr change-detection strategy plugged into `cdx-watch`. The crate
/// owns the load → poll → persist → collect loop over the [`ItemStore`]; this
/// impl owns the ČÚZK fetch, the diff, and the snapshot update.
pub struct KatastrWatcher;

impl Watcher for KatastrWatcher {
    type Snapshot = ProceedingState;
    type Change = Change;
    type Error = KatastrError;

    /// Re-fetch from ČÚZK (the `api_id` fast path when available, else the search
    /// endpoint) and fold the fresh data into the snapshot. Mirrors Python's
    /// `_check_state`, including the backfill-but-never-wipe rules for `api_id`
    /// and `datum_prijeti`. `cdx-watch` persists the returned snapshot.
    fn poll(&self, mut state: ProceedingState) -> Result<(ProceedingState, Option<Change>)> {
        let canonical = state.cislo_rizeni.clone();

        let fetched = match &state.api_id {
            Some(id) if !id.is_null() => fetch_by_id(id)?,
            _ => fetch_from_cuzk(&parse_proceeding_number(&canonical)?)?,
        };
        let data = fetched.ok_or_else(|| {
            KatastrError::ProceedingNotFound("Řízení nebylo nalezeno v ČÚZK API.".into())
        })?;

        let change = detect_changes(&state, &data);

        state.last_check_at = now_utc();
        // Backfill but never wipe: keep the stored id / datum_prijeti when ČÚZK
        // omits it (datum_prijeti is immutable; a missing id would else strand us
        // on the slower Vyhledani path forever).
        if let Some(id) = data.get("id") {
            if !id.is_null() {
                state.api_id = Some(id.clone());
            }
        }
        state.stav = str_field(&data, "stav");
        state.stav_uhrady = data.get("stavUhrady").and_then(|v| v.as_str()).map(String::from);
        let new_prijeti = str_field(&data, "datumPrijeti");
        if !new_prijeti.is_empty() {
            state.datum_prijeti = new_prijeti;
        }
        state.provedene_operace = operace_from_data(&data);
        if let Some(c) = change.clone() {
            state.changes.push(c);
        }

        Ok((state, change))
    }

    /// Filesystem-safe storage key: the canonical number with `/` → `_`.
    fn slug(&self, state: &ProceedingState) -> String {
        crate::paths::dir_name(&state.cislo_rizeni)
    }

    /// katastr's result row is keyed by the canonical proceeding number; the CLI
    /// and CGI carry it through unchanged as `cislo_rizeni`.
    fn display_name(&self, state: &ProceedingState) -> String {
        state.cislo_rizeni.clone()
    }
}

/// Re-fetch a single tracked proceeding and apply any detected changes.
pub fn check_one(cislo_rizeni: &str) -> Result<CheckResult> {
    let canonical = normalize_number(cislo_rizeni)?;
    let slug = crate::paths::dir_name(&canonical);
    match track::check_one(&state::store(), &KatastrWatcher, &slug) {
        Ok(result) => Ok(CheckResult::from_watch(result)),
        // The only error `check_one` returns here is `NotTracked`.
        Err(_) => Err(KatastrError::ProceedingNotTracked(format!(
            "Řízení {canonical} není sledováno."
        ))),
    }
}

/// Re-fetch every tracked proceeding. A failure on any single one becomes an
/// error entry - the whole run still completes so the UI can surface per-row
/// status.
pub fn check_all() -> Vec<CheckResult> {
    track::check_all(&state::store(), &KatastrWatcher)
        .into_iter()
        .map(CheckResult::from_watch)
        .collect()
}

/// Mark changes as confirmed. When `change_index` is `None`, every unconfirmed
/// change is marked; when `Some(n)`, only that index flips. Returns the number
/// of entries actually marked (0 when already confirmed).
pub fn confirm(cislo_rizeni: &str, change_index: Option<usize>) -> Result<usize> {
    let canonical = normalize_number(cislo_rizeni)?;
    let mut state = ProceedingState::load(&canonical).ok_or_else(|| {
        KatastrError::ProceedingNotTracked(format!("Řízení {canonical} není sledováno."))
    })?;

    let marked = confirm_changes(&mut state.changes, change_index, &now_utc()).map_err(|e| match e {
        cdx_watch::WatchError::IndexOutOfRange { index, len } => {
            KatastrError::InvalidProceedingNumber(format!(
                "Index {index} je mimo rozsah (0..{}).",
                len.saturating_sub(1)
            ))
        }
        other => KatastrError::InvalidProceedingNumber(other.to_string()),
    })?;

    if marked > 0 {
        save_state(&canonical, &state)?;
    }
    Ok(marked)
}

// ── check result ────────────────────────────────────────────────────────────

/// Per-proceeding result returned by [`check_one`] / [`check_all`].
/// Matches the Python dict shape so the CGI can pass it straight through.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub cislo_rizeni: String,
    pub ok: bool,
    pub change: Option<Change>,
    pub error: Option<String>,
}

impl CheckResult {
    /// Adapt a `cdx-watch` result into katastr's shape. The watcher carries the
    /// canonical proceeding number in `display_name` (see [`KatastrWatcher`]),
    /// which is exactly the `cislo_rizeni` the CLI/CGI expose.
    fn from_watch(result: track::CheckResult<Change>) -> Self {
        Self {
            cislo_rizeni: result.display_name,
            ok: result.ok,
            change: result.change,
            error: result.error,
        }
    }

    /// True when the check completed and carries a newly detected change.
    pub fn has_change(&self) -> bool {
        self.ok && self.change.is_some()
    }
}

// ── state save helper ───────────────────────────────────────────────────────

fn save_state(cislo_rizeni: &str, state: &ProceedingState) -> Result<()> {
    crate::state::store()
        .save(&crate::paths::dir_name(cislo_rizeni), state)
        .map_err(KatastrError::Io)
}

// ── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_form() {
        let p = parse_proceeding_number("V-123/2026-701").unwrap();
        assert_eq!(p.typ_rizeni, "V");
        assert_eq!(p.poradove_cislo, 123);
        assert_eq!(p.rok, 2026);
        assert_eq!(p.kod_pracoviste, 701);
    }

    #[test]
    fn uppercases_type() {
        let p = parse_proceeding_number("v-123/2026-701").unwrap();
        assert_eq!(p.typ_rizeni, "V");
    }

    #[test]
    fn trims_whitespace() {
        let p = parse_proceeding_number("  V-123/2026-701  ").unwrap();
        assert_eq!(p.canonical(), "V-123/2026-701");
    }

    #[test]
    fn multi_letter_type_ok() {
        let p = parse_proceeding_number("OZ-1/2026-701").unwrap();
        assert_eq!(p.typ_rizeni, "OZ");
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_proceeding_number("").is_err());
        assert!(parse_proceeding_number("   ").is_err());
    }

    #[test]
    fn rejects_wrong_separators() {
        assert!(parse_proceeding_number("V123/2026-701").is_err());
        assert!(parse_proceeding_number("V-123-2026-701").is_err());
        assert!(parse_proceeding_number("V-123/2026/701").is_err());
    }

    #[test]
    fn rejects_nonnumeric_parts() {
        assert!(parse_proceeding_number("V-abc/2026-701").is_err());
        assert!(parse_proceeding_number("1-123/2026-701").is_err());
        assert!(parse_proceeding_number("V-123/26-701").is_err()); // 2-digit year
    }

    #[test]
    fn normalize_matches_canonical() {
        assert_eq!(normalize_number("v-123/2026-701").unwrap(), "V-123/2026-701");
    }

    #[test]
    fn uhrada_label_maps_known_codes() {
        assert_eq!(uhrada_label("U"), Some("Uhrazeno"));
        assert_eq!(uhrada_label("N"), Some("Neuhrazeno"));
        assert_eq!(uhrada_label("O"), Some("Osvobozeno"));
        assert_eq!(uhrada_label("X"), None);
    }

    #[test]
    fn now_utc_is_iso8601_with_z_suffix() {
        let ts = now_utc();
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20); // YYYY-MM-DDTHH:MM:SSZ
        assert!(ts.contains('T'));
    }

    #[test]
    fn api_id_segment_renders_scalar_without_quotes() {
        assert_eq!(api_id_segment(&serde_json::json!(12345)), "12345");
        assert_eq!(api_id_segment(&serde_json::json!("ab-9")), "ab-9");
    }

    #[test]
    fn detect_changes_flags_new_operation() {
        let state = ProceedingState {
            uuid: "u".into(),
            cislo_rizeni: "V-1/2026-701".into(),
            typ_rizeni: "V".into(),
            poradove_cislo: 1,
            rok: 2026,
            kod_pracoviste: 701,
            api_id: None,
            label: String::new(),
            added_on: "2026-07-01T00:00:00Z".into(),
            last_check_at: "2026-07-01T00:00:00Z".into(),
            stav: "Probíhá".into(),
            stav_uhrady: Some("N".into()),
            datum_prijeti: "2026-06-01".into(),
            provedene_operace: vec![Operace {
                nazev: "Založení".into(),
                datum_provedeni: "2026-06-01".into(),
            }],
            changes: vec![],
        };
        let fresh = serde_json::json!({
            "stav": "Probíhá",
            "stavUhrady": "N",
            "provedeneOperace": [
                {"nazev": "Založení", "datumProvedeni": "2026-06-01"},
                {"nazev": "Plomba", "datumProvedeni": "2026-06-10"}
            ]
        });
        let change = detect_changes(&state, &fresh).expect("a new operation should be a change");
        assert_eq!(change.new_operations.len(), 1);
        assert_eq!(change.new_operations[0].nazev, "Plomba");
        assert!(!change.stav_uhrady_changed);
    }

    #[test]
    fn detect_changes_returns_none_when_identical() {
        let state = ProceedingState {
            uuid: "u".into(),
            cislo_rizeni: "V-1/2026-701".into(),
            typ_rizeni: "V".into(),
            poradove_cislo: 1,
            rok: 2026,
            kod_pracoviste: 701,
            api_id: None,
            label: String::new(),
            added_on: "2026-07-01T00:00:00Z".into(),
            last_check_at: "2026-07-01T00:00:00Z".into(),
            stav: "Probíhá".into(),
            stav_uhrady: Some("N".into()),
            datum_prijeti: "2026-06-01".into(),
            provedene_operace: vec![Operace {
                nazev: "Založení".into(),
                datum_provedeni: "2026-06-01".into(),
            }],
            changes: vec![],
        };
        let fresh = serde_json::json!({
            "stav": "Probíhá",
            "stavUhrady": "N",
            "provedeneOperace": [{"nazev": "Založení", "datumProvedeni": "2026-06-01"}]
        });
        assert!(detect_changes(&state, &fresh).is_none());
    }

    #[test]
    fn describe_change_renders_all_facets() {
        let change = Change {
            detected_on: now_utc(),
            old_stav: "Probíhá".into(),
            new_stav: "Povoleno".into(),
            new_operations: vec![Operace {
                nazev: "Vklad proveden".into(),
                datum_provedeni: "2026-07-10".into(),
            }],
            stav_uhrady_changed: true,
            old_stav_uhrady: Some("N".into()),
            new_stav_uhrady: Some("U".into()),
            confirmed_on: None,
        };
        let lines = describe_change(&change);
        assert_eq!(lines[0], "Stav: Probíhá → Povoleno");
        assert_eq!(lines[1], "Nová operace: Vklad proveden (2026-07-10)");
        assert_eq!(lines[2], "Úhrada: Neuhrazeno → Uhrazeno");
    }
}

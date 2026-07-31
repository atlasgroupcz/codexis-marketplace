//! RÚIAN through ArcGIS on ags.cuzk.gov.cz.
//!
//! Two doors into the same service, and each is good at something the other
//! cannot do:
//!
//! - a **layer** is one table (obce, katastrální území, ulice, ...). It filters
//!   server-side, has no result cap, and returns codes. It needs the exact
//!   stored spelling, and you have to know which table to open.
//! - the **suggester** searches across all tables at once and tolerates missing
//!   diacritics. It caps at 15 results, cannot filter by type, and returns no
//!   codes.
//!
//! So: ask the layer whenever possible, the suggester only when the layer
//! cannot be used - when the caller has not said what kind of thing this is, or
//! when the text is not what is stored (an address is filed under its street,
//! so "Hladké Životice 66" lives as "Malá Strana 66, 74247 Hladké Životice").
//!
//! Asking the suggester and filtering its answer here is what the previous
//! version did, and it produced wrong answers: the cap applies before our
//! filter, so a type that did not make the top 15 was reported as not existing
//! at all. Verified: 15 suggestions for "Nová Ves" are all Obec, while RÚIAN
//! also holds a KatastralniUzemi and 62 CastObce of that name.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::api_client::{agent, call_with_retry, Outcome};
use crate::error::{KatastrError, Result};
use crate::settings::ruian_base;

/// The MapServer layer holding a given type. `None` means we do not resolve a
/// code for it - `ParcelaDefinicniBod` is the case that matters, where the
/// cadastre API is the right route, not RÚIAN.
pub fn layer_for_type(typ: &str) -> Option<u8> {
    match typ {
        "AdresniMisto" => Some(1),
        "Ulice" => Some(4),
        "ZakladniSidelniJednotka" => Some(6),
        "KatastralniUzemi" => Some(7),
        "MestskyObvodMestskaCastVeStatutarnimMesteAHlavnimMestePraze" => Some(8),
        "SpravniObvodVHlavnimMestePraze" => Some(10),
        "CastObce" => Some(11),
        "Obec" => Some(12),
        "ObecSPoverenymObecnimUradem" => Some(13),
        "ObecSRozsirenouPusobnosti" => Some(14),
        "Okres" => Some(15),
        "VyssiUzemneSamospravnyCelek" => Some(17),
        _ => None,
    }
}

/// Reverse of [`layer_for_type`], for naming what a `magicKey` pointed at.
pub fn type_for_layer(layer: u8) -> Option<&'static str> {
    match layer {
        1 => Some("AdresniMisto"),
        4 => Some("Ulice"),
        6 => Some("ZakladniSidelniJednotka"),
        7 => Some("KatastralniUzemi"),
        8 => Some("MestskyObvodMestskaCastVeStatutarnimMesteAHlavnimMestePraze"),
        10 => Some("SpravniObvodVHlavnimMestePraze"),
        11 => Some("CastObce"),
        12 => Some("Obec"),
        13 => Some("ObecSPoverenymObecnimUradem"),
        14 => Some("ObecSRozsirenouPusobnosti"),
        15 => Some("Okres"),
        17 => Some("VyssiUzemneSamospravnyCelek"),
        _ => None,
    }
}

/// The field a layer keeps its name in. Addresses are the exception: their
/// whole "Street 66, 12345 Town" line lives in `adresa`.
pub fn name_field(layer: u8) -> &'static str {
    if layer == 1 {
        "adresa"
    } else {
        "nazev"
    }
}

/// The parent that makes rows of a layer tell each other apart, and the layer
/// holding that parent's name. Fourteen municipalities are called Nová Ves;
/// their districts are what distinguishes them.
pub fn context_for_layer(layer: u8) -> Option<(&'static str, u8)> {
    match layer {
        4 | 7 | 8 | 10 | 11 => Some(("obec", 12)),
        12 | 14 => Some(("okres", 15)),
        15 => Some(("vusc", 17)),
        6 => Some(("katastralniuzemi", 7)),
        _ => None,
    }
}

/// ArcGIS `where` is SQL, so an apostrophe in a name has to be doubled.
///
/// This is not a general SQL escaper and must not be reused elsewhere without
/// review. It suffices here because what gets escaped is either the caller's
/// name or the suggester's own canonical text, into a template whose field name
/// is hardcoded.
pub fn escape_sql(value: &str) -> String {
    value.replace('\'', "''")
}

/// Does a suggestion pass the type filter? With no type given, every one does.
///
/// Its own function on purpose: were the condition ever inverted, a search
/// without a type would return empty for every query - a silent fault only a
/// live run would surface.
pub fn matches_type(candidate_typ: &str, want: Option<&str>) -> bool {
    want.is_none_or(|w| candidate_typ == w)
}

/// Splits a suggester `magicKey` into the layer and the record it points at.
/// The suggester renders them as `<layer>_<objectid>` - verified: `12_10108`
/// is layer 12, objectid 10108, which is Nová Ves in the Žďár nad Sázavou
/// district, code 596213.
pub fn split_magic_key(key: &str) -> Option<(u8, i64)> {
    let (layer, objectid) = key.split_once('_')?;
    Some((layer.parse().ok()?, objectid.parse().ok()?))
}

// ── transport ────────────────────────────────────────────────────────────────
// Not unit-tested, same as `http.rs` in `ares` - it needs a live call to
// ags.cuzk.gov.cz.

#[derive(Deserialize)]
struct SuggestResponse {
    #[serde(default)]
    suggestions: Vec<RawSuggestion>,
}

#[derive(Deserialize)]
struct RawSuggestion {
    #[serde(default)]
    text: String,
    #[serde(rename = "magicKey", default)]
    magic_key: String,
    #[serde(rename = "type", default)]
    typ: String,
}

/// Retrying is handled by `api_client::call_with_retry` - the same rule as for
/// ČÚZK. This used to be called exactly once, and it is precisely the call
/// whose transport reset (the connection died mid TLS handshake) started this
/// branch.
///
/// The error message separates transport from HTTP status: both used to land
/// under "could not reach RÚIAN", so even a successfully contacted service
/// answering HTTP 500 looked like a network outage.
fn get_json(url: &str) -> Result<serde_json::Value> {
    match call_with_retry(|| agent().get(url).set("Accept", "application/json")) {
        Outcome::Ok(resp) => resp
            .into_json::<serde_json::Value>()
            .map_err(|e| KatastrError::ApiNetwork(format!("RÚIAN vrátil neplatný JSON: {e}"))),
        Outcome::Status(code, body) => Err(KatastrError::ApiHttp {
            status: code,
            message: match crate::cuzk_error::truncate(&body) {
                Some(snippet) => format!("RÚIAN vrátilo HTTP {code}: {snippet}"),
                None => format!("RÚIAN vrátilo HTTP {code}"),
            },
        }),
        Outcome::Transport(t) => Err(KatastrError::ApiNetwork(format!(
            "Nepodařilo se kontaktovat RÚIAN: {t}"
        ))),
    }
}

/// One row of a layer, reduced to what a caller can act on.
struct Row {
    kod: i64,
    nazev: String,
    kontext: Option<i64>,
}

/// Queries one layer. `where_clause` is passed through as given; every caller
/// builds it from [`escape_sql`].
fn query_layer(layer: u8, where_clause: &str) -> Result<Vec<Row>> {
    let base = ruian_base();
    let field = name_field(layer);
    let ctx = context_for_layer(layer).map(|(f, _)| f);
    let out = match ctx {
        Some(c) => format!("kod,{field},{c}"),
        None => format!("kod,{field}"),
    };
    let url = format!(
        "{base}/{layer}/query?where={}&outFields={out}&returnGeometry=false&f=json",
        crate::lookup::percent_encode(where_clause)
    );
    let body = get_json(&url)?;
    let features = body
        .get("features")
        .and_then(|f| f.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(features
        .iter()
        .filter_map(|f| {
            let a = f.get("attributes")?;
            Some(Row {
                kod: a.get("kod")?.as_i64()?,
                nazev: a.get(field)?.as_str().unwrap_or_default().to_string(),
                kontext: ctx.and_then(|c| a.get(c)).and_then(|v| v.as_i64()),
            })
        })
        .collect())
}

/// Resolves the parent codes of a result set to names, in one query. Free -
/// the ags service does not count against the ČÚZK call quota.
fn resolve_context(layer: u8, rows: &[Row]) -> BTreeMap<i64, String> {
    let Some((_, parent_layer)) = context_for_layer(layer) else {
        return BTreeMap::new();
    };
    let mut codes: Vec<String> = rows
        .iter()
        .filter_map(|r| r.kontext)
        .map(|c| c.to_string())
        .collect();
    codes.sort();
    codes.dedup();
    if codes.is_empty() {
        return BTreeMap::new();
    }
    let where_clause = format!("kod IN ({})", codes.join(","));
    // Best-effort: a missing parent name costs a label, not the answer.
    query_layer(parent_layer, &where_clause)
        .map(|parents| parents.into_iter().map(|p| (p.kod, p.nazev)).collect())
        .unwrap_or_default()
}

fn rows_to_json(layer: u8, rows: Vec<Row>) -> serde_json::Value {
    let names = resolve_context(layer, &rows);
    serde_json::Value::Array(
        rows.into_iter()
            .map(|r| {
                let mut o = serde_json::Map::new();
                o.insert("kod".into(), r.kod.into());
                o.insert("nazev".into(), r.nazev.into());
                if let Some(c) = r.kontext {
                    let label = names.get(&c).cloned();
                    o.insert(
                        "kontext".into(),
                        label.map_or_else(|| c.into(), serde_json::Value::String),
                    );
                }
                serde_json::Value::Object(o)
            })
            .collect(),
    )
}

/// Asks the suggester what a piece of text might be. Used only where a layer
/// cannot answer: to learn the type, or to turn free text into the exact
/// spelling a layer needs.
fn suggest(text: &str) -> Result<Vec<RawSuggestion>> {
    let base = ruian_base();
    let url = format!(
        "{base}/exts/GeocodeSOE/suggest?text={}&f=json",
        crate::lookup::percent_encode(text)
    );
    let raw = get_json(&url)?;
    let parsed: SuggestResponse = serde_json::from_value(raw)?;
    Ok(parsed.suggestions)
}

/// Resolves a `magicKey` from a suggestion straight to its record. Exact by
/// construction - the key names the row, so nothing has to be matched by name.
pub fn by_key(key: &str) -> Result<serde_json::Value> {
    let Some((layer, objectid)) = split_magic_key(key) else {
        return Err(KatastrError::InvalidProceedingNumber(format!(
            "Klíč {key} nemá tvar <vrstva>_<id>."
        )));
    };
    let rows = query_layer(layer, &format!("objectid={objectid}"))?;
    Ok(serde_json::json!({
        "typ": type_for_layer(layer),
        "vrstva": layer,
        "nalezy": rows_to_json(layer, rows),
    }))
}

/// Translates human text into RÚIAN codes.
///
/// With `typ`, the layer is known, so it is asked directly: no cap, filtering
/// on the server, every match returned with its code. Without it, the type is
/// what has to be found first, and only the suggester can say.
pub fn search(text: &str, typ: Option<&str>) -> Result<serde_json::Value> {
    let Some(want) = typ else {
        return Ok(serde_json::json!({ "navrhy": suggestions_json(text, None)? }));
    };
    let Some(layer) = layer_for_type(want) else {
        return Ok(serde_json::json!({
            "typ": want,
            "poznamka": "pro tento typ RÚIAN kód nedohledáváme",
        }));
    };

    // An address is filed under its street, so the text a caller types is never
    // what the layer stores. Go through the suggester first.
    if layer != 1 {
        let rows = query_layer(layer, &exact_where(layer, text))?;
        if !rows.is_empty() {
            return Ok(found(want, layer, text, rows));
        }
    }

    // Nothing matched exactly - the spelling is off, or it is an address. Let
    // the suggester canonicalise, then ask the layer again.
    let ids: Vec<String> = suggest(text)?
        .into_iter()
        .filter(|s| matches_type(&s.typ, Some(want)))
        .filter_map(|s| split_magic_key(&s.magic_key).map(|(_, id)| id.to_string()))
        .collect();
    if ids.is_empty() {
        return Ok(found(want, layer, text, Vec::new()));
    }
    let rows = query_layer(layer, &format!("objectid IN ({})", ids.join(",")))?;
    Ok(found(want, layer, text, rows))
}

fn exact_where(layer: u8, text: &str) -> String {
    format!("{}='{}'", name_field(layer), escape_sql(text))
}

fn found(typ: &str, layer: u8, dotaz: &str, rows: Vec<Row>) -> serde_json::Value {
    serde_json::json!({
        "typ": typ,
        "vrstva": layer,
        "dotaz": dotaz,
        "pocet": rows.len(),
        "nalezy": rows_to_json(layer, rows),
    })
}

/// The suggester's own list, passed on as it comes. Its `text` already carries
/// the district ("Nová Ves (Louny)"), so entries tell each other apart without
/// any further lookup, and `klic` points at the exact record.
fn suggestions_json(text: &str, typ: Option<&str>) -> Result<serde_json::Value> {
    Ok(serde_json::Value::Array(
        suggest(text)?
            .into_iter()
            .filter(|s| matches_type(&s.typ, typ))
            .map(|s| {
                serde_json::json!({
                    "text": s.text,
                    "typ": s.typ,
                    "klic": s.magic_key,
                })
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_type_the_suggester_returns_to_its_layer() {
        // Measured against the service's own layer list; the five that used to
        // be missing are the reason a Prague city district had no code.
        for (typ, layer) in [
            ("AdresniMisto", 1),
            ("Ulice", 4),
            ("ZakladniSidelniJednotka", 6),
            ("KatastralniUzemi", 7),
            ("MestskyObvodMestskaCastVeStatutarnimMesteAHlavnimMestePraze", 8),
            ("SpravniObvodVHlavnimMestePraze", 10),
            ("CastObce", 11),
            ("Obec", 12),
            ("ObecSPoverenymObecnimUradem", 13),
            ("ObecSRozsirenouPusobnosti", 14),
            ("Okres", 15),
            ("VyssiUzemneSamospravnyCelek", 17),
        ] {
            assert_eq!(layer_for_type(typ), Some(layer), "type {typ}");
            assert_eq!(type_for_layer(layer), Some(typ), "layer {layer}");
        }
    }

    #[test]
    fn parcel_points_stay_unmapped_because_the_cadastre_api_owns_them() {
        assert_eq!(layer_for_type("ParcelaDefinicniBod"), None);
        assert_eq!(type_for_layer(0), None);
    }

    #[test]
    fn addresses_match_on_their_full_line_everything_else_on_a_name() {
        assert_eq!(name_field(1), "adresa");
        assert_eq!(name_field(12), "nazev");
    }

    #[test]
    fn layers_whose_rows_repeat_a_name_carry_a_parent_to_tell_them_apart() {
        assert_eq!(context_for_layer(12), Some(("okres", 15)));
        assert_eq!(context_for_layer(7), Some(("obec", 12)));
        assert_eq!(context_for_layer(15), Some(("vusc", 17)));
        // An address line is already unique; a region has nothing above it.
        assert_eq!(context_for_layer(1), None);
        assert_eq!(context_for_layer(17), None);
    }

    #[test]
    fn magic_key_splits_into_layer_and_record() {
        assert_eq!(split_magic_key("12_10108"), Some((12, 10108)));
        assert_eq!(split_magic_key("0_13174496"), Some((0, 13174496)));
    }

    #[test]
    fn a_malformed_key_is_refused_rather_than_guessed() {
        assert_eq!(split_magic_key("12"), None);
        assert_eq!(split_magic_key("obec_10108"), None);
        assert_eq!(split_magic_key("12_x"), None);
        assert_eq!(split_magic_key(""), None);
    }

    #[test]
    fn where_clause_escapes_single_quotes() {
        assert_eq!(escape_sql("O'Brien"), "O''Brien");
        assert_eq!(
            exact_where(12, "Ves O'Brien"),
            "nazev='Ves O''Brien'"
        );
        assert_eq!(
            exact_where(1, "Malá Strana 66, 74247 Hladké Životice"),
            "adresa='Malá Strana 66, 74247 Hladké Životice'"
        );
    }

    #[test]
    fn without_a_requested_type_every_suggestion_passes() {
        assert!(matches_type("AdresniMisto", None));
        assert!(matches_type("ParcelaDefinicniBod", None));
    }

    #[test]
    fn a_requested_type_admits_only_that_type() {
        assert!(matches_type("AdresniMisto", Some("AdresniMisto")));
        assert!(!matches_type("ParcelaDefinicniBod", Some("AdresniMisto")));
    }
}

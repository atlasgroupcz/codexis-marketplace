use std::collections::HashSet;

use serde_json::{json, Value};


const ARES_WEB_BASE_URL: &str = "https://ares.gov.cz/ekonomicke-subjekty";
const SNIPPET_MAX_CHARS: usize = 300;
pub(crate) const SEARCH_SOURCES_CAP: usize = 10;

/// Best-effort publish of chat sources; delegates to the shared publisher.
pub(crate) fn try_publish(entries: &[Value]) {
    cdx_common::sources::try_publish("ares-cli", entries);
}

/// One source for `detail <ICO>` - the official ARES web page of the subject.
pub(crate) fn detail_source(ico8: &str, body: &str) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    vec![subject_entry(
        ico8,
        parsed.get("obchodniJmeno"),
        parsed.pointer("/sidlo/textovaAdresa"),
    )]
}

/// One source per found subject in a `search` response, capped and deduped.
pub(crate) fn search_sources(body: &str, cap: usize) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    subjects_from(&parsed, cap)
}

/// Sources for the raw `api` escape hatch, derived by response shape:
/// a search-style envelope or a single subject with an `ico`.
pub(crate) fn sniff_sources(body: &str) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if parsed.get("ekonomickeSubjekty").is_some() {
        return subjects_from(&parsed, SEARCH_SOURCES_CAP);
    }
    if let Some(ico) = parsed.get("ico").and_then(Value::as_str) {
        return vec![subject_entry(
            ico,
            parsed.get("obchodniJmeno"),
            parsed.pointer("/sidlo/textovaAdresa"),
        )];
    }
    Vec::new()
}

fn subjects_from(parsed: &Value, cap: usize) -> Vec<Value> {
    let Some(items) = parsed.get("ekonomickeSubjekty").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    for item in items {
        let Some(ico) = item.get("ico").and_then(Value::as_str) else {
            continue;
        };
        if !seen.insert(ico.to_string()) {
            continue;
        }
        entries.push(subject_entry(
            ico,
            item.get("obchodniJmeno"),
            item.pointer("/sidlo/textovaAdresa"),
        ));
        if entries.len() == cap {
            break;
        }
    }
    entries
}

fn subject_entry(ico: &str, name: Option<&Value>, address: Option<&Value>) -> Value {
    let title = name
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("IČO {ico} - ARES"));
    let mut entry = serde_json::Map::new();
    entry.insert("__kind".into(), json!("url"));
    entry.insert("url".into(), json!(format!("{ARES_WEB_BASE_URL}?ico={ico}")));
    entry.insert("title".into(), json!(title));
    if let Some(addr) = address.and_then(Value::as_str) {
        entry.insert("snippet".into(), json!(truncate_chars(addr, SNIPPET_MAX_CHARS)));
    }
    Value::Object(entry)
}

fn truncate_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_source_uses_name_and_address() {
        let body = r#"{"ico":"45274649","obchodniJmeno":"ČEZ, a. s.","sidlo":{"textovaAdresa":"Duhová 1444/2, Michle, 14000 Praha 4"}}"#;
        let entries = detail_source("45274649", body);
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0],
            json!({
                "__kind": "url",
                "url": "https://ares.gov.cz/ekonomicke-subjekty?ico=45274649",
                "title": "ČEZ, a. s.",
                "snippet": "Duhová 1444/2, Michle, 14000 Praha 4"
            })
        );
    }

    #[test]
    fn detail_source_falls_back_to_ico_title_for_vr_shape() {
        let body = r#"{"icoId":"45274649","zaznamy":[{"obchodniJmeno":[{"hodnota":"ČEZ, a. s."}]}]}"#;
        let entries = detail_source("45274649", body);
        assert_eq!(entries[0]["title"], json!("IČO 45274649 - ARES"));
        assert!(entries[0].get("snippet").is_none());
    }

    #[test]
    fn search_sources_caps_and_dedupes() {
        let subjects: Vec<String> = (0..12)
            .map(|i| format!(r#"{{"ico":"{:08}","obchodniJmeno":"Firma {i}"}}"#, i % 11))
            .collect();
        let body = format!(r#"{{"ekonomickeSubjekty":[{}]}}"#, subjects.join(","));
        let entries = search_sources(&body, 10);
        assert_eq!(entries.len(), 10);
        let urls: HashSet<&str> = entries.iter().map(|e| e["url"].as_str().unwrap()).collect();
        assert_eq!(urls.len(), 10);
    }

    #[test]
    fn sniff_sources_handles_detail_search_and_other_shapes() {
        assert_eq!(
            sniff_sources(r#"{"ico":"04997476","obchodniJmeno":"X"}"#)[0]["url"],
            json!("https://ares.gov.cz/ekonomicke-subjekty?ico=04997476")
        );
        assert_eq!(
            sniff_sources(r#"{"ekonomickeSubjekty":[{"ico":"04997476"}]}"#).len(),
            1
        );
        assert!(sniff_sources(r#"{"pocetCelkem":0}"#).is_empty());
        assert!(sniff_sources("not json").is_empty());
    }

    #[test]
    fn snippet_truncates_on_char_boundary() {
        let address = "Šťěpánská ".repeat(40);
        let body = format!(
            r#"{{"ico":"00000001","obchodniJmeno":"X","sidlo":{{"textovaAdresa":"{address}"}}}}"#
        );
        let entries = detail_source("00000001", &body);
        let snippet = entries[0]["snippet"].as_str().unwrap();
        assert_eq!(snippet.chars().count(), 300);
    }
}

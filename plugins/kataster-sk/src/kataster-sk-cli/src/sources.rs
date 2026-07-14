use std::collections::HashSet;

use serde_json::{json, Value};


const MAPKA_DETAIL_BASE_URL: &str = "https://zbgis.skgeodesy.sk/mapka/sk/kataster/detail";
const SNIPPET_MAX_CHARS: usize = 300;
pub(crate) const PARCEL_SOURCES_CAP: usize = 10;

/// Best-effort publish of chat sources; delegates to the shared publisher.
pub(crate) fn try_publish(entries: &[Value]) {
    cdx_common::sources::try_publish("kataster-sk-cli", entries);
}

/// One MAPKA deep link per parcel found by `parcela`, capped and deduped.
/// Only `kataster/…` routes are citable MAPKA pages — other categories
/// (municipalities, KU geocoding hits) are skipped.
pub(crate) fn parcela_sources(body: &str, cap: usize) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let Some(items) = parsed.get("items").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    for item in items {
        let Some(data) = item.get("data") else {
            continue;
        };
        let Some(route) = data.get("route").and_then(Value::as_str) else {
            continue;
        };
        if !route.starts_with("kataster/") || !seen.insert(route.to_string()) {
            continue;
        }
        entries.push(parcel_entry(route, data));
        if entries.len() == cap {
            break;
        }
    }
    entries
}

fn parcel_entry(route: &str, data: &Value) -> Value {
    let mut entry = serde_json::Map::new();
    entry.insert("__kind".into(), json!("url"));
    entry.insert("url".into(), json!(format!("{MAPKA_DETAIL_BASE_URL}/{route}")));
    entry.insert("title".into(), json!(parcel_title(route, data)));
    if let Some(description) = data.get("description").and_then(Value::as_str) {
        entry.insert(
            "snippet".into(),
            json!(truncate_chars(description, SNIPPET_MAX_CHARS)),
        );
    }
    Value::Object(entry)
}

fn parcel_title(route: &str, data: &Value) -> String {
    let text = data.get("text").and_then(Value::as_str).unwrap_or(route);
    match data.get("category").and_then(Value::as_str) {
        Some("parcela-c") => format!("Parcela {text} (register C)"),
        Some("parcela-e") => format!("Parcela {text} (register E)"),
        _ => format!("Parcela {text}"),
    }
}

fn truncate_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parcela_sources_builds_mapka_links() {
        let body = r#"{"items": [
            {"data": {"category": "parcela-c", "id": 1070374091, "text": "1234/9",
                      "description": "Ružinov (805556)", "route": "kataster/parcela-c/805556/1234_9"}},
            {"data": {"category": "parcela-e", "id": 99, "text": "1234/9",
                      "route": "kataster/parcela-e/805556/1234_9"}}
        ]}"#;
        let entries = parcela_sources(body, 10);
        assert_eq!(
            entries[0],
            json!({
                "__kind": "url",
                "url": "https://zbgis.skgeodesy.sk/mapka/sk/kataster/detail/kataster/parcela-c/805556/1234_9",
                "title": "Parcela 1234/9 (register C)",
                "snippet": "Ružinov (805556)"
            })
        );
        assert_eq!(entries[1]["title"], json!("Parcela 1234/9 (register E)"));
        assert!(entries[1].get("snippet").is_none());
    }

    #[test]
    fn parcela_sources_skips_non_kataster_routes() {
        let body = r#"{"items": [
            {"data": {"category": "katastrálne územie", "id": 805556, "text": "Ružinov (805556)", "route": "ease/basic/805556"}},
            {"data": {"category": "obec", "text": "Ružinov"}}
        ]}"#;
        assert!(parcela_sources(body, 10).is_empty());
    }

    #[test]
    fn parcela_sources_caps_and_dedupes() {
        let items: Vec<String> = (0..12)
            .map(|i| {
                format!(
                    r#"{{"data": {{"category": "parcela-c", "text": "{i}", "route": "kataster/parcela-c/805556/{}"}}}}"#,
                    i % 11
                )
            })
            .collect();
        let body = format!(r#"{{"items": [{}]}}"#, items.join(","));
        assert_eq!(parcela_sources(&body, 10).len(), 10);
    }

    #[test]
    fn parcela_sources_handles_invalid_json() {
        assert!(parcela_sources("not json", 10).is_empty());
        assert!(parcela_sources(r#"{"items": "nope"}"#, 10).is_empty());
    }
}

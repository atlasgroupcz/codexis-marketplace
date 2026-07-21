use std::collections::HashSet;

use serde_json::{json, Value};


const RPO_ENTITY_BASE_URL: &str = "https://api.statistics.sk/rpo/v1/entity";
const SNIPPET_MAX_CHARS: usize = 300;
pub(crate) const SEARCH_SOURCES_CAP: usize = 10;

/// Best-effort publish of chat sources; delegates to the shared publisher.
pub(crate) fn try_publish(entries: &[Value]) {
    cdx_common::sources::try_publish("zrsr-cli", entries);
}

/// One source for `detail` - the public RPO record of the subject.
pub(crate) fn detail_source(body: &str) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    entity_entry(&parsed).into_iter().collect()
}

/// One source per found subject in a `search` response, capped and deduped.
pub(crate) fn search_sources(body: &str, cap: usize) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    entities_from(&parsed, cap)
}

/// Sources for the raw `api` escape hatch, derived by response shape:
/// a search-style `results` envelope or a single entity with an `id`.
pub(crate) fn sniff_sources(body: &str) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if parsed.get("results").is_some() {
        return entities_from(&parsed, SEARCH_SOURCES_CAP);
    }
    entity_entry(&parsed).into_iter().collect()
}

fn entities_from(parsed: &Value, cap: usize) -> Vec<Value> {
    let Some(items) = parsed.get("results").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    for item in items {
        let Some(id) = entity_id(item) else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        if let Some(entry) = entity_entry(item) {
            entries.push(entry);
        }
        if entries.len() == cap {
            break;
        }
    }
    entries
}

fn entity_entry(entity: &Value) -> Option<Value> {
    let id = entity_id(entity)?;
    let title = current_value(entity.get("fullNames"))
        .and_then(|name| name.get("value"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| fallback_title(entity, id));
    let mut entry = serde_json::Map::new();
    entry.insert("__kind".into(), json!("url"));
    entry.insert("url".into(), json!(format!("{RPO_ENTITY_BASE_URL}/{id}")));
    entry.insert("title".into(), json!(title));
    if let Some(address) = current_value(entity.get("addresses")).and_then(format_address) {
        entry.insert("snippet".into(), json!(truncate_chars(&address, SNIPPET_MAX_CHARS)));
    }
    Some(Value::Object(entry))
}

fn entity_id(entity: &Value) -> Option<u64> {
    entity.get("id").and_then(Value::as_u64)
}

fn fallback_title(entity: &Value, id: u64) -> String {
    match current_value(entity.get("identifiers"))
        .and_then(|identifier| identifier.get("value"))
        .and_then(Value::as_str)
    {
        Some(ico) => format!("IČO {ico} - RPO"),
        None => format!("RPO {id}"),
    }
}

/// Picks the current entry of an RPO validFrom/validTo history array -
/// the one without `validTo`, falling back to the last entry.
fn current_value(history: Option<&Value>) -> Option<&Value> {
    let items = history?.as_array()?;
    items
        .iter()
        .find(|item| item.get("validTo").is_none_or(Value::is_null))
        .or_else(|| items.last())
}

fn format_address(address: &Value) -> Option<String> {
    let street_name = address
        .get("street")
        .and_then(Value::as_str)
        .or_else(|| address.pointer("/municipality/value").and_then(Value::as_str))?;
    let mut formatted = street_name.to_string();
    if let Some(building) = address.get("buildingNumber").and_then(Value::as_str) {
        formatted = format!("{formatted} {building}");
    }
    if let Some(municipality) = address.pointer("/municipality/value").and_then(Value::as_str) {
        if municipality != street_name {
            formatted = format!("{formatted}, {municipality}");
        }
    }
    Some(formatted)
}

fn truncate_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: u64, name: &str) -> String {
        format!(
            r#"{{"id":{id},"identifiers":[{{"value":"00686930"}}],
               "fullNames":[
                 {{"value":"Stará firma","validFrom":"1990-11-01","validTo":"2014-07-31"}},
                 {{"value":"{name}","validFrom":"2014-08-01"}}
               ],
               "addresses":[{{"street":"Hodžovo námestie","buildingNumber":"3","municipality":{{"value":"Bratislava 1"}}}}]}}"#
        )
    }

    #[test]
    fn detail_source_uses_current_name_and_address() {
        let entries = detail_source(&entity(4445617, "Tatra banka, a.s."));
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0],
            serde_json::json!({
                "__kind": "url",
                "url": "https://api.statistics.sk/rpo/v1/entity/4445617",
                "title": "Tatra banka, a.s.",
                "snippet": "Hodžovo námestie 3, Bratislava 1"
            })
        );
    }

    #[test]
    fn detail_source_falls_back_to_ico_title_and_skips_without_id() {
        let entries = detail_source(r#"{"id":7,"identifiers":[{"value":"12345678"}]}"#);
        assert_eq!(entries[0]["title"], serde_json::json!("IČO 12345678 - RPO"));
        assert!(entries[0].get("snippet").is_none());
        assert!(detail_source(r#"{"message":"No record found"}"#).is_empty());
    }

    #[test]
    fn search_sources_caps_and_dedupes() {
        let items: Vec<String> = (0..12).map(|i| entity(i % 11, "Firma")).collect();
        let body = format!(r#"{{"results":[{}]}}"#, items.join(","));
        let entries = search_sources(&body, 10);
        assert_eq!(entries.len(), 10);
        let urls: HashSet<&str> = entries.iter().map(|e| e["url"].as_str().unwrap()).collect();
        assert_eq!(urls.len(), 10);
    }

    #[test]
    fn sniff_sources_handles_detail_search_and_other_shapes() {
        assert_eq!(
            sniff_sources(&entity(42, "X"))[0]["url"],
            serde_json::json!("https://api.statistics.sk/rpo/v1/entity/42")
        );
        assert_eq!(sniff_sources(&format!(r#"{{"results":[{}]}}"#, entity(1, "X"))).len(), 1);
        assert!(sniff_sources(r#"{"results":[]}"#).is_empty());
        assert!(sniff_sources("not json").is_empty());
    }

    #[test]
    fn snippet_truncates_on_char_boundary() {
        let street = "Šťěpánská ".repeat(40);
        let body = format!(
            r#"{{"id":1,"fullNames":[{{"value":"X"}}],"addresses":[{{"street":"{street}"}}]}}"#
        );
        let entries = detail_source(&body);
        let snippet = entries[0]["snippet"].as_str().unwrap();
        assert_eq!(snippet.chars().count(), 300);
    }
}

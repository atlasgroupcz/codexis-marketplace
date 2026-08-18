//! Search-payload building blocks shared by the CODEXIS search plugins.
//!
//! Helpers for assembling a search request body from CLI flags and an optional
//! JSON override: inserting typed values into a payload map, reading the JSON
//! override (from a string or stdin) and range-checking numeric fields. The
//! per-plugin argument structs, the `SearchPayloadArgs` trait and each source's
//! `extend_payload` stay in the plugin.

use std::io::{self, Read};

use serde_json::{Map, Value};

use crate::error::CliError;

/// Inserts a trimmed, non-empty string value under `key`.
pub fn insert_string(payload: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        payload.insert(key.to_string(), Value::String(value.to_string()));
    }
}

/// Inserts a numeric value under `key` when present.
pub fn insert_u64(payload: &mut Map<String, Value>, key: &str, value: Option<u64>) {
    if let Some(value) = value {
        payload.insert(key.to_string(), Value::from(value));
    }
}

/// Inserts `true` under `key` when `value` is set; omits the key otherwise.
pub fn insert_bool(payload: &mut Map<String, Value>, key: &str, value: bool) {
    if value {
        payload.insert(key.to_string(), Value::Bool(true));
    }
}

/// Loads an optional JSON payload override - a literal string, or `-` to read
/// it from stdin. Must be a JSON object.
pub fn load_search_payload(
    raw_payload: Option<&str>,
) -> Result<Option<Map<String, Value>>, CliError> {
    let Some(raw_payload) = raw_payload else {
        return Ok(None);
    };

    let payload = if raw_payload == "-" {
        let mut payload = String::new();
        io::stdin()
            .read_to_string(&mut payload)
            .map_err(|source| CliError::Io {
                context: "failed to read search payload from stdin".to_string(),
                source,
            })?;
        payload
    } else {
        raw_payload.to_string()
    };

    let value: Value =
        serde_json::from_str(&payload).map_err(|error| CliError::InvalidJson(error.to_string()))?;

    match value {
        Value::Object(object) => Ok(Some(object)),
        _ => Err(CliError::SearchPayloadMustBeObject),
    }
}

/// Reads a payload field as a non-negative integer, erroring if it is present
/// but not a `u64`.
pub fn get_non_negative_integer(
    payload: &Map<String, Value>,
    key: &str,
) -> Result<Option<u64>, CliError> {
    match payload.get(key) {
        None => Ok(None),
        Some(Value::Number(number)) => number
            .as_u64()
            .ok_or_else(|| {
                CliError::InvalidSearchArgument(format!("{key} must be a non-negative integer"))
            })
            .map(Some),
        Some(_) => Err(CliError::InvalidSearchArgument(format!(
            "{key} must be a non-negative integer"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn insert_string_trims_and_skips_empty() {
        let mut p = Map::new();
        insert_string(&mut p, "q", &Some("  hi  ".to_string()));
        insert_string(&mut p, "empty", &Some("   ".to_string()));
        assert_eq!(p.get("q"), Some(&json!("hi")));
        assert!(!p.contains_key("empty"));
    }

    #[test]
    fn load_payload_parses_object_and_rejects_non_object() {
        assert!(load_search_payload(None).unwrap().is_none());
        assert!(load_search_payload(Some(r#"{"a":1}"#)).unwrap().is_some());
        assert!(matches!(
            load_search_payload(Some("[1,2]")).unwrap_err(),
            CliError::SearchPayloadMustBeObject
        ));
    }

    #[test]
    fn non_negative_integer_validates() {
        let mut p = Map::new();
        p.insert("offset".to_string(), json!(5));
        p.insert("bad".to_string(), json!(-1));
        assert_eq!(get_non_negative_integer(&p, "offset").unwrap(), Some(5));
        assert_eq!(get_non_negative_integer(&p, "missing").unwrap(), None);
        assert!(get_non_negative_integer(&p, "bad").is_err());
    }
}

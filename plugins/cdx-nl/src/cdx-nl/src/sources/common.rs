use clap::Args;
use serde_json::{Map, Value};
use std::io::{self, Read};

use crate::core::error::CliError;

pub(crate) type JsonMap = Map<String, Value>;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchBaseArgs {
    #[arg(long, short = 'q', help = "Fulltext query")]
    pub(crate) query: Option<String>,

    #[arg(long, help = "Sort mode (e.g. relevance, date)")]
    pub(crate) sort: Option<String>,

    #[arg(long, help = "Sort order (asc or desc)")]
    pub(crate) order: Option<String>,

    #[arg(
        value_name = "JSON_PAYLOAD",
        help = "Optional JSON request body, or '-' to read it from stdin. JSON booleans use true/false"
    )]
    pub(crate) payload: Option<String>,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}

impl SearchBaseArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        // sort/order are @RequestParam on the backend (see
        // WettenOverheidNlBwbController.search and RechtspraakNlUitsprakenController.search) —
        // they are appended to the search URL as query params, NOT into the JSON body.
        // `query` belongs to the body via WettenOverheidNlBwbSearchRequest.query /
        // RechtspraakSearchRequest.query.
        insert_string(payload, "query", &self.query);
    }
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct OffsetLimitArgs {
    #[arg(long, help = "Result offset (backend default: 0)")]
    pub(crate) offset: Option<u64>,

    #[arg(long, help = "Result limit (backend default: 20, max 100)")]
    pub(crate) limit: Option<u64>,
}

impl OffsetLimitArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_u64(payload, "offset", self.offset);
        insert_u64(payload, "limit", self.limit);
    }
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct FromSizeArgs {
    #[arg(long, help = "Result offset (backend default: 0)")]
    pub(crate) from: Option<u64>,

    #[arg(long, help = "Page size (backend default: 20, max 100)")]
    pub(crate) size: Option<u64>,
}

impl FromSizeArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_u64(payload, "from", self.from);
        insert_u64(payload, "size", self.size);
    }
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct DateRangeArgs {
    #[arg(long = "date-from", help = "Date from (YYYY-MM-DD)")]
    pub(crate) date_from: Option<String>,

    #[arg(long = "date-to", help = "Date to (YYYY-MM-DD)")]
    pub(crate) date_to: Option<String>,
}
// no insert_into — sources map to the right field names themselves

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct IssuedDateArgs {
    #[arg(long = "issued-from", help = "Issued from date (YYYY-MM-DD)")]
    pub(crate) issued_from: Option<String>,

    #[arg(long = "issued-to", help = "Issued to date (YYYY-MM-DD)")]
    pub(crate) issued_to: Option<String>,
}
// no insert_into — sources map to the right field names themselves

pub(crate) trait SearchPayloadArgs {
    fn base(&self) -> &SearchBaseArgs;
    fn extend_payload(&self, payload: &mut JsonMap);

    fn build_payload(&self, source_code: &'static str) -> Result<String, CliError> {
        let mut payload = JsonMap::new();
        self.base().insert_into(&mut payload);
        self.extend_payload(&mut payload);
        merge_payload_override(source_code, &mut payload, self.base().payload.as_deref())?;
        validate_search_payload(&payload)?;
        serde_json::to_string(&Value::Object(payload))
            .map_err(|error| CliError::SerializePayload(error.to_string()))
    }

    fn dry_run(&self) -> bool {
        self.base().dry_run
    }

    fn sort(&self) -> Option<&str> {
        self.base().sort.as_deref()
    }

    fn order(&self) -> Option<&str> {
        self.base().order.as_deref()
    }
}

pub(crate) fn insert_string(payload: &mut JsonMap, key: &str, value: &Option<String>) {
    if let Some(value) = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        payload.insert(key.to_string(), Value::String(value.to_string()));
    }
}

pub(crate) fn insert_string_list(payload: &mut JsonMap, key: &str, value: &[String]) {
    if value.is_empty() {
        return;
    }
    let arr: Vec<Value> = value.iter().map(|s| Value::String(s.clone())).collect();
    payload.insert(key.into(), Value::Array(arr));
}

pub(crate) fn insert_u64(payload: &mut JsonMap, key: &str, value: Option<u64>) {
    if let Some(value) = value {
        payload.insert(key.to_string(), Value::from(value));
    }
}

#[allow(dead_code)]
pub(crate) fn insert_bool(payload: &mut JsonMap, key: &str, value: bool) {
    if value {
        payload.insert(key.to_string(), Value::Bool(true));
    }
}

fn merge_payload_override(
    _source_code: &str,
    payload: &mut JsonMap,
    raw_payload: Option<&str>,
) -> Result<(), CliError> {
    if let Some(override_payload) = load_search_payload(raw_payload)? {
        payload.extend(override_payload);
    }

    Ok(())
}

fn load_search_payload(raw_payload: Option<&str>) -> Result<Option<JsonMap>, CliError> {
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

fn validate_search_payload(payload: &JsonMap) -> Result<(), CliError> {
    let _ = get_non_negative_integer(payload, "offset")?;
    Ok(())
}

fn get_non_negative_integer(payload: &JsonMap, key: &str) -> Result<Option<u64>, CliError> {
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
    use serde_json::{Map, Value};

    use super::*;

    // TODO: replace in Task B2/B6 — SK source-specific tests removed.
    // Two tests that don't depend on any source were retained.

    #[test]
    fn search_payload_must_be_object() {
        let error = load_search_payload(Some(r#"["not","an","object"]"#)).unwrap_err();
        assert!(matches!(error, CliError::SearchPayloadMustBeObject));
    }

    #[test]
    fn search_without_query_is_accepted() {
        let payload = Map::from_iter([("limit".to_string(), Value::from(10_u64))]);
        assert!(validate_search_payload(&payload).is_ok());
    }
}

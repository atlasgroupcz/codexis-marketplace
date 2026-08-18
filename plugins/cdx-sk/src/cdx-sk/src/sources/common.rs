use clap::Args;
use serde_json::{Map, Value};

use crate::core::error::CliError;
pub(crate) use cdx_common::search::{
    get_non_negative_integer, insert_string, insert_u64, load_search_payload,
};

pub(crate) type JsonMap = Map<String, Value>;

const DEFAULT_RESULT_LIMIT: u64 = 10;
const DEFAULT_RESULT_OFFSET: u64 = 0;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchBaseArgs {
    #[arg(long, short = 'q', help = "Fulltext query")]
    pub(crate) query: Option<String>,

    #[arg(long, help = "Result limit (default: 10)")]
    pub(crate) limit: Option<u64>,

    #[arg(long, help = "Result offset (default: 0)")]
    pub(crate) offset: Option<u64>,

    #[arg(long, help = "Sort mode (e.g. relevance, title, date)")]
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

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct DateRangeArgs {
    #[arg(long = "date-from", help = "Date from (YYYY-MM-DD)")]
    pub(crate) date_from: Option<String>,

    #[arg(long = "date-to", help = "Date to (YYYY-MM-DD)")]
    pub(crate) date_to: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct IssuedDateArgs {
    #[arg(long = "issued-from", help = "Issued from date (YYYY-MM-DD)")]
    pub(crate) issued_from: Option<String>,

    #[arg(long = "issued-to", help = "Issued to date (YYYY-MM-DD)")]
    pub(crate) issued_to: Option<String>,
}

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

impl SearchBaseArgs {
    fn insert_into(&self, payload: &mut JsonMap) {
        insert_string(payload, "query", &self.query);
        insert_u64(
            payload,
            "limit",
            Some(self.limit.unwrap_or(DEFAULT_RESULT_LIMIT)),
        );
        insert_u64(
            payload,
            "offset",
            Some(self.offset.unwrap_or(DEFAULT_RESULT_OFFSET)),
        );
    }
}

impl DateRangeArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string(payload, "dateFrom", &self.date_from);
        insert_string(payload, "dateTo", &self.date_to);
    }
}

impl IssuedDateArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string(payload, "issuedFrom", &self.issued_from);
        insert_string(payload, "issuedTo", &self.issued_to);
    }
}

/// Insert a JSON array of trimmed, non-empty strings under `key`. No-op when the slice yields
/// nothing, so an unset repeatable filter never appears in the payload.
pub(crate) fn insert_string_vec(payload: &mut JsonMap, key: &str, values: &[String]) {
    let items: Vec<Value> = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| Value::String(value.to_string()))
        .collect();
    if !items.is_empty() {
        payload.insert(key.to_string(), Value::Array(items));
    }
}

/// Court-domain default page size - matches the court controllers' DEFAULT_LIMIT (20).
const DEFAULT_COURT_PAGE_SIZE: u64 = 20;

/// Build a court-domain search payload. The Slovak court `/search` endpoints (nssud_sk,
/// ustavnysud_sk, nsud_sk) page with `from`/`size` (offset semantics) rather than the
/// `offset`/`limit` keys the slov-lex sources use - and there are no `@JsonAlias` fallbacks - so
/// this mirrors the base `query` + paging emission with the correct keys, then applies the domain
/// filters (`extend`) and any JSON payload override.
pub(crate) fn build_court_search_payload(
    base: &SearchBaseArgs,
    extend: impl FnOnce(&mut JsonMap),
) -> Result<String, CliError> {
    let mut payload = JsonMap::new();
    insert_string(&mut payload, "query", &base.query);
    insert_u64(
        &mut payload,
        "size",
        Some(base.limit.unwrap_or(DEFAULT_COURT_PAGE_SIZE)),
    );
    insert_u64(
        &mut payload,
        "from",
        Some(base.offset.unwrap_or(DEFAULT_RESULT_OFFSET)),
    );
    extend(&mut payload);
    merge_payload_override("", &mut payload, base.payload.as_deref())?;
    validate_search_payload(&payload)?;
    serde_json::to_string(&Value::Object(payload))
        .map_err(|error| CliError::SerializePayload(error.to_string()))
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

fn validate_search_payload(payload: &JsonMap) -> Result<(), CliError> {
    let _ = get_non_negative_integer(payload, "offset")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};

    use super::*;
    use crate::sources::skez::SearchSkezArgs;
    use crate::sources::skvs::SearchSkvsArgs;

    #[test]
    fn json_payload_overrides_matching_flag_values() {
        let args = SearchSkvsArgs {
            base: SearchBaseArgs {
                query: Some("from-flags".to_string()),
                limit: Some(5),
                payload: Some(r#"{"query":"from-json","limit":1,"offset":7}"#.to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchSkvsArgs::default()
        };

        let payload = args.build_payload("SKVS").unwrap();
        assert_eq!(payload, r#"{"limit":1,"offset":7,"query":"from-json"}"#);
    }

    #[test]
    fn search_payload_can_be_built_from_flags_only() {
        let args = SearchSkezArgs {
            base: SearchBaseArgs {
                query: Some("občiansky zákonník".to_string()),
                limit: Some(5),
                ..SearchBaseArgs::default()
            },
            doc_number: Some("40/1964 Zb.".to_string()),
            ..SearchSkezArgs::default()
        };

        let payload = args.build_payload("SKEZ").unwrap();
        let parsed: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["query"], "občiansky zákonník");
        assert_eq!(parsed["limit"], 5);
        assert_eq!(parsed["offset"], 0);
        assert_eq!(parsed["docNumber"], "40/1964 Zb.");
    }

    #[test]
    fn search_payload_includes_default_limit_and_offset_when_omitted() {
        let args = SearchSkezArgs {
            base: SearchBaseArgs {
                query: Some("test".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchSkezArgs::default()
        };

        let payload = args.build_payload("SKEZ").unwrap();
        let parsed: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["limit"], 10);
        assert_eq!(parsed["offset"], 0);
    }

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

    #[test]
    fn sort_and_order_are_separate_from_payload() {
        let args = SearchSkezArgs {
            base: SearchBaseArgs {
                query: Some("test".to_string()),
                sort: Some("date".to_string()),
                order: Some("asc".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchSkezArgs::default()
        };

        let payload = args.build_payload("SKEZ").unwrap();
        let parsed: Value = serde_json::from_str(&payload).unwrap();
        // sort and order should NOT be in the JSON payload
        assert!(parsed.get("sort").is_none());
        assert!(parsed.get("order").is_none());
        // but accessible via trait methods
        assert_eq!(args.sort(), Some("date"));
        assert_eq!(args.order(), Some("asc"));
    }

    #[test]
    fn court_payload_uses_from_and_size_paging() {
        let base = SearchBaseArgs {
            query: Some("dovolanie".to_string()),
            limit: Some(5),
            offset: Some(20),
            ..SearchBaseArgs::default()
        };

        let payload = build_court_search_payload(&base, |payload| {
            insert_string_vec(payload, "kolegium", &["Trestnoprávne".to_string()]);
        })
        .unwrap();

        let parsed: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["query"], "dovolanie");
        assert_eq!(parsed["size"], 5);
        assert_eq!(parsed["from"], 20);
        assert_eq!(parsed["kolegium"], Value::from(vec!["Trestnoprávne"]));
        // Court sources must NOT emit the slov-lex limit/offset keys.
        assert!(parsed.get("limit").is_none());
        assert!(parsed.get("offset").is_none());
    }

    #[test]
    fn court_payload_defaults_size_to_twenty() {
        let base = SearchBaseArgs {
            query: Some("test".to_string()),
            ..SearchBaseArgs::default()
        };

        let payload = build_court_search_payload(&base, |_| {}).unwrap();
        let parsed: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["size"], 20);
        assert_eq!(parsed["from"], 0);
    }
}

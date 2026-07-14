use clap::Args;
use serde_json::{Map, Number, Value};

use crate::core::error::CliError;
pub(crate) use cdx_common::search::{insert_string, insert_u64, insert_bool, load_search_payload, get_non_negative_integer};

pub(crate) type JsonMap = Map<String, Value>;

const DEFAULT_RESULT_LIMIT: u64 = 20;
const DEFAULT_RESULT_OFFSET: u64 = 0;

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchBaseArgs {
    #[arg(long, short = 'q', help = "Fulltext query (predmet + party names)")]
    pub(crate) query: Option<String>,

    #[arg(long, help = "Result limit (default: 20, max: 100)")]
    pub(crate) limit: Option<u64>,

    #[arg(long, help = "Result offset (default: 0; offset + limit must be <= 10000)")]
    pub(crate) offset: Option<u64>,

    #[arg(long, help = "Sort field (relevance, date)")]
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

pub(crate) trait SearchPayloadArgs {
    fn base(&self) -> &SearchBaseArgs;
    fn extend_payload(&self, payload: &mut JsonMap);

    fn build_payload(&self) -> Result<String, CliError> {
        let mut payload = JsonMap::new();
        self.base().insert_into(&mut payload);
        self.extend_payload(&mut payload);
        merge_payload_override(&mut payload, self.base().payload.as_deref())?;
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
        // Backend fulltext field is `q` (SmlouvySearchRequest.q), not `query`.
        insert_string(payload, "q", &self.query);
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

/// Insert an optional numeric (contract value) filter. Skips NaN/inf so the payload
/// stays valid JSON (serde_json cannot serialize non-finite floats).
pub(crate) fn insert_number(payload: &mut JsonMap, key: &str, value: Option<f64>) {
    if let Some(number) = value.and_then(Number::from_f64) {
        payload.insert(key.to_string(), Value::Number(number));
    }
}

fn merge_payload_override(
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
    use crate::sources::smlouvy::SearchSmlouvyArgs;

    #[test]
    fn json_payload_overrides_matching_flag_values() {
        let args = SearchSmlouvyArgs {
            base: SearchBaseArgs {
                query: Some("from-flags".to_string()),
                limit: Some(5),
                payload: Some(r#"{"q":"from-json","limit":1,"offset":7}"#.to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchSmlouvyArgs::default()
        };

        let payload = args.build_payload().unwrap();
        assert_eq!(payload, r#"{"limit":1,"offset":7,"q":"from-json"}"#);
    }

    #[test]
    fn search_payload_can_be_built_from_flags_only() {
        let args = SearchSmlouvyArgs {
            base: SearchBaseArgs {
                query: Some("úklid".to_string()),
                limit: Some(5),
                ..SearchBaseArgs::default()
            },
            publisher_ico: Some("27074358".to_string()),
            ..SearchSmlouvyArgs::default()
        };

        let payload = args.build_payload().unwrap();
        let parsed: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["q"], "úklid");
        assert_eq!(parsed["limit"], 5);
        assert_eq!(parsed["offset"], 0);
        assert_eq!(parsed["publisherIco"], "27074358");
    }

    #[test]
    fn search_payload_includes_default_limit_and_offset_when_omitted() {
        let args = SearchSmlouvyArgs {
            base: SearchBaseArgs {
                query: Some("test".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchSmlouvyArgs::default()
        };

        let payload = args.build_payload().unwrap();
        let parsed: Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["limit"], 20);
        assert_eq!(parsed["offset"], 0);
    }

    #[test]
    fn search_payload_must_be_object() {
        let error = load_search_payload(Some(r#"["not","an","object"]"#)).unwrap_err();
        assert!(matches!(error, CliError::SearchPayloadMustBeObject));
    }

    #[test]
    fn search_without_query_is_accepted() {
        let payload = Map::from_iter([("limit".to_string(), Value::from(20_u64))]);
        assert!(validate_search_payload(&payload).is_ok());
    }

    #[test]
    fn sort_and_order_are_separate_from_payload() {
        let args = SearchSmlouvyArgs {
            base: SearchBaseArgs {
                query: Some("test".to_string()),
                sort: Some("date".to_string()),
                order: Some("asc".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchSmlouvyArgs::default()
        };

        let payload = args.build_payload().unwrap();
        let parsed: Value = serde_json::from_str(&payload).unwrap();
        // sort and order should NOT be in the JSON payload
        assert!(parsed.get("sort").is_none());
        assert!(parsed.get("order").is_none());
        // but accessible via trait methods
        assert_eq!(args.sort(), Some("date"));
        assert_eq!(args.order(), Some("asc"));
    }

    #[test]
    fn insert_number_writes_finite_and_skips_non_finite() {
        let mut payload = JsonMap::new();
        insert_number(&mut payload, "valueMin", Some(1000.0));
        insert_number(&mut payload, "valueMax", None);
        insert_number(&mut payload, "bad", Some(f64::NAN));
        assert_eq!(payload["valueMin"], 1000.0);
        assert!(payload.get("valueMax").is_none());
        assert!(payload.get("bad").is_none());
    }
}

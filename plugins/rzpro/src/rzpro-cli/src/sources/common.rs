use clap::Args;
use serde_json::{Map, Value};

use crate::core::error::CliError;
pub(crate) use cdx_common::search::{
    get_non_negative_integer, insert_bool, insert_string, insert_u64, load_search_payload,
};

pub(crate) type JsonMap = Map<String, Value>;

const DEFAULT_RESULT_LIMIT: u64 = 20;
const DEFAULT_RESULT_OFFSET: u64 = 0;

/// clap value parser for an `--ico` flag: normalize an IČO to its canonical
/// 8-digit, zero-padded form (rejecting non-digits / >8 digits). RZPRO stores
/// IČO zero-padded to 8 digits and the backend matches it exactly, so
/// canonicalizing here lets users omit leading zeros (e.g. `565474` matches
/// `00565474`). Invalid input is rejected at parse time with a clear message.
pub(crate) fn parse_ico(raw: &str) -> Result<String, String> {
    cdx_common::ico::normalize_ico(raw).map_err(|error| error.to_string())
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchBaseArgs {
    #[arg(long, short = 'q', help = "Fulltext query")]
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
    /// Inserts only the universal paging keys. The fulltext field differs per
    /// source (`q` for devices, `nazev` for persons), so each source inserts it
    /// in its own `extend_payload`.
    fn insert_into(&self, payload: &mut JsonMap) {
        insert_u64(payload, "limit", Some(self.limit.unwrap_or(DEFAULT_RESULT_LIMIT)));
        insert_u64(payload, "offset", Some(self.offset.unwrap_or(DEFAULT_RESULT_OFFSET)));
    }
}

fn merge_payload_override(payload: &mut JsonMap, raw_payload: Option<&str>) -> Result<(), CliError> {
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
    use super::*;
    use crate::sources::rzpro::SearchRzproArgs;
    use serde_json::Value;

    #[test]
    fn default_paging_and_query_key_q() {
        let args = SearchRzproArgs {
            base: SearchBaseArgs { query: Some("gliadin".to_string()), ..SearchBaseArgs::default() },
            ..SearchRzproArgs::default()
        };
        let p: Value = serde_json::from_str(&args.build_payload().unwrap()).unwrap();
        assert_eq!(p["limit"], 20);
        assert_eq!(p["offset"], 0);
        assert_eq!(p["q"], "gliadin");
    }

    #[test]
    fn json_override_wins_over_flags() {
        let args = SearchRzproArgs {
            base: SearchBaseArgs {
                query: Some("from-flags".to_string()),
                payload: Some(r#"{"q":"from-json","limit":1}"#.to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchRzproArgs::default()
        };
        let p: Value = serde_json::from_str(&args.build_payload().unwrap()).unwrap();
        assert_eq!(p["q"], "from-json");
        assert_eq!(p["limit"], 1);
    }

    #[test]
    fn parse_ico_pads_and_validates() {
        assert_eq!(parse_ico("565474").unwrap(), "00565474");
        assert_eq!(parse_ico("00565474").unwrap(), "00565474");
        assert!(parse_ico("12a45").is_err());
        assert!(parse_ico("123456789").is_err());
    }
}

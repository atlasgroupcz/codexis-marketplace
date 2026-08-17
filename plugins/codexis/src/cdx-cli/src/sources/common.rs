use clap::Args;
use serde_json::{Map, Value};

use crate::core::error::CliError;
use crate::core::http::SearchFacetMode;
pub(crate) use cdx_common::search::{
    get_non_negative_integer, insert_bool, insert_string, insert_u64, load_search_payload,
};

pub(crate) type JsonMap = Map<String, Value>;

const DEFAULT_RESULT_LIMIT: u64 = 10;
const DEFAULT_RESULT_OFFSET: u64 = 0;
const DEFAULT_SORT_MODE: &str = "RELEVANCE";
const DEFAULT_SORT_ORDER: &str = "DESC";

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchBaseArgs {
    #[arg(long, short = 'q', help = "Fulltext query")]
    pub(crate) query: Option<String>,

    #[arg(long, help = "Result limit (default: 10)")]
    pub(crate) limit: Option<u64>,

    #[arg(long, help = "Result offset, zero-based (default: 0)")]
    pub(crate) offset: Option<u64>,

    #[arg(
        value_name = "JSON_PAYLOAD",
        help = "Optional JSON request body, or '-' to read it from stdin. JSON booleans use true/false"
    )]
    pub(crate) payload: Option<String>,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct SearchFacetArgs {
    #[arg(
        long = "with-facets",
        conflicts_with = "with_full_facets",
        help = "Include available filters in search output"
    )]
    pub(crate) with_facets: bool,

    #[arg(
        long = "with-full-facets",
        conflicts_with = "with_facets",
        help = "Include all available filters in search output and request full facet set"
    )]
    pub(crate) with_full_facets: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct StandardSortArgs {
    #[arg(long, help = "Sort mode (default: RELEVANCE)", value_name = "SORT")]
    pub(crate) sort: Option<String>,

    #[arg(
        long = "sort-order",
        help = "Sort order (default: DESC)",
        value_name = "SORT_ORDER"
    )]
    pub(crate) sort_order: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct CrSortArgs {
    #[arg(
        long = "sort",
        help = "Sort mode (default: RELEVANCE)",
        value_name = "SORT"
    )]
    pub(crate) sort_by: Option<String>,

    #[arg(
        long = "sort-order",
        help = "Sort order (default: DESC)",
        value_name = "SORT_ORDER"
    )]
    pub(crate) sort_order: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct IssuedDateArgs {
    #[arg(long = "issued-from", help = "Issued from date (YYYY-MM-DD)")]
    pub(crate) issued_from: Option<String>,

    #[arg(long = "issued-to", help = "Issued to date (YYYY-MM-DD)")]
    pub(crate) issued_to: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct EffectiveDateArgs {
    #[arg(long = "effective-from", help = "Effective from date (YYYY-MM-DD)")]
    pub(crate) effective_from: Option<String>,

    #[arg(long = "effective-to", help = "Effective to date (YYYY-MM-DD)")]
    pub(crate) effective_to: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct ApprovedDateArgs {
    #[arg(long = "approved-from", help = "Approved from date (YYYY-MM-DD)")]
    pub(crate) approved_from: Option<String>,

    #[arg(long = "approved-to", help = "Approved to date (YYYY-MM-DD)")]
    pub(crate) approved_to: Option<String>,
}

#[derive(Args, Debug, Clone, Default)]
pub(crate) struct ChangedDateArgs {
    #[arg(long = "changed-from", help = "Changed from date (YYYY-MM-DD)")]
    pub(crate) changed_from: Option<String>,

    #[arg(long = "changed-to", help = "Changed to date (YYYY-MM-DD)")]
    pub(crate) changed_to: Option<String>,
}

pub(crate) trait SearchPayloadArgs {
    fn base(&self) -> &SearchBaseArgs;
    fn extend_payload(&self, payload: &mut JsonMap);
    fn facet_mode(&self) -> SearchFacetMode {
        SearchFacetMode::Hidden
    }

    fn build_payload(&self, source_code: &'static str) -> Result<String, CliError> {
        let mut payload = JsonMap::new();
        self.base().insert_into(&mut payload);
        self.extend_payload(&mut payload);
        merge_payload_override(source_code, &mut payload, self.base().payload.as_deref())?;
        validate_search_payload(source_code, &payload)?;
        serde_json::to_string(&Value::Object(payload))
            .map_err(|error| CliError::SerializePayload(error.to_string()))
    }

    fn dry_run(&self) -> bool {
        self.base().dry_run
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

impl SearchFacetArgs {
    pub(crate) fn mode(&self) -> SearchFacetMode {
        if self.with_full_facets {
            SearchFacetMode::Full
        } else if self.with_facets {
            SearchFacetMode::Summary
        } else {
            SearchFacetMode::Hidden
        }
    }
}

impl StandardSortArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string_value(
            payload,
            "sort",
            self.sort.as_deref().unwrap_or(DEFAULT_SORT_MODE),
        );
        insert_string_value(
            payload,
            "sortOrder",
            self.sort_order.as_deref().unwrap_or(DEFAULT_SORT_ORDER),
        );
    }
}

impl CrSortArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string_value(
            payload,
            "sortBy",
            self.sort_by.as_deref().unwrap_or(DEFAULT_SORT_MODE),
        );
        insert_string_value(
            payload,
            "sortOrder",
            self.sort_order.as_deref().unwrap_or(DEFAULT_SORT_ORDER),
        );
    }
}

impl IssuedDateArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string(payload, "issuedFrom", &self.issued_from);
        insert_string(payload, "issuedTo", &self.issued_to);
    }
}

impl EffectiveDateArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string(payload, "effectiveFrom", &self.effective_from);
        insert_string(payload, "effectiveTo", &self.effective_to);
    }
}

impl ApprovedDateArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string(payload, "approvedFrom", &self.approved_from);
        insert_string(payload, "approvedTo", &self.approved_to);
    }
}

impl ChangedDateArgs {
    pub(crate) fn insert_into(&self, payload: &mut JsonMap) {
        insert_string(payload, "changedFrom", &self.changed_from);
        insert_string(payload, "changedTo", &self.changed_to);
    }
}

pub(crate) fn insert_string_value(payload: &mut JsonMap, key: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        payload.insert(key.to_string(), Value::String(value.to_string()));
    }
}

pub(crate) fn insert_string_array(payload: &mut JsonMap, key: &str, values: &[String]) {
    let values = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| Value::String(value.to_string()))
        .collect::<Vec<_>>();

    if !values.is_empty() {
        payload.insert(key.to_string(), Value::Array(values));
    }
}

fn merge_payload_override(
    source_code: &str,
    payload: &mut JsonMap,
    raw_payload: Option<&str>,
) -> Result<(), CliError> {
    if let Some(mut override_payload) = load_search_payload(raw_payload)? {
        normalize_override_payload(source_code, &mut override_payload);
        payload.extend(override_payload);
    }

    Ok(())
}

fn normalize_override_payload(source_code: &str, payload: &mut JsonMap) {
    // Keep the external JSON surface stable (`sort`, `sortOrder`) and translate
    // to the source-specific request shape only at the API boundary.
    if source_code == "CR" {
        normalize_sort_alias(payload, "sortBy");
    } else {
        normalize_sort_alias(payload, "sort");
    }
}

fn normalize_sort_alias(payload: &mut JsonMap, target_key: &str) {
    let canonical_sort = payload.remove("sort");
    let legacy_sort_by = payload.remove("sortBy");

    if let Some(value) = canonical_sort.or(legacy_sort_by) {
        payload.insert(target_key.to_string(), value);
    }
}

fn validate_search_payload(source_code: &str, payload: &JsonMap) -> Result<(), CliError> {
    if let Some(limit) = get_non_negative_integer(payload, "limit")? {
        if !(1..=50).contains(&limit) {
            return Err(CliError::InvalidSearchArgument(
                "limit must be an integer between 1 and 50".to_string(),
            ));
        }
    }

    let _ = get_non_negative_integer(payload, "offset")?;

    if source_code == "CR"
        && payload_contains_true(payload, "validNow")?
        && payload.contains_key("validAt")
    {
        return Err(CliError::InvalidSearchArgument(
            "validNow and validAt cannot be combined in the final request payload".to_string(),
        ));
    }

    // Every source rejects a blank query with a bare "400 Invalid request
    // content" that names nothing. Read from the merged payload, not the flag, so
    // a query supplied through the JSON body counts. An agent reads this message
    // and acts on it, so it states the limit and offers no way around it.
    let has_query = payload
        .get("query")
        .and_then(Value::as_str)
        .is_some_and(|query| !query.trim().is_empty());
    if !has_query {
        return Err(CliError::InvalidSearchArgument(
            "query is required: filters narrow a search, they cannot be one".to_string(),
        ));
    }

    Ok(())
}

fn payload_contains_true(payload: &JsonMap, key: &str) -> Result<bool, CliError> {
    match payload.get(key) {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(CliError::InvalidSearchArgument(format!(
            "{key} must be a boolean"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};

    use super::*;
    use crate::sources::all::SearchAllArgs;
    use crate::sources::cr::SearchCrArgs;
    use crate::sources::jd::SearchJdArgs;

    #[test]
    fn json_payload_overrides_matching_flag_values() {
        let args = SearchJdArgs {
            base: SearchBaseArgs {
                query: Some("from-flags".to_string()),
                limit: Some(5),
                payload: Some(
                    r#"{"query":"from-json","limit":1,"offset":7,"sort":"DATE","sortOrder":"ASC"}"#
                        .to_string(),
                ),
                ..SearchBaseArgs::default()
            },
            sort: StandardSortArgs {
                sort: Some("NAME".to_string()),
                sort_order: Some("DESC".to_string()),
            },
            ..SearchJdArgs::default()
        };

        let payload = args.build_payload("JD").unwrap();
        assert_eq!(
            payload,
            r#"{"limit":1,"offset":7,"query":"from-json","sort":"DATE","sortOrder":"ASC"}"#
        );
    }

    #[test]
    fn cr_json_payload_uses_canonical_sort_key_and_maps_to_sort_by() {
        let args = SearchCrArgs {
            base: SearchBaseArgs {
                query: Some("from-flags".to_string()),
                payload: Some(
                    r#"{"query":"from-json","sort":"DATE","sortOrder":"ASC"}"#.to_string(),
                ),
                ..SearchBaseArgs::default()
            },
            ..SearchCrArgs::default()
        };

        let payload = args.build_payload("CR").unwrap();
        assert_eq!(
            payload,
            r#"{"limit":10,"offset":0,"query":"from-json","sortBy":"DATE","sortOrder":"ASC"}"#
        );
    }

    #[test]
    fn non_cr_json_payload_accepts_legacy_sort_by_and_normalizes_to_sort() {
        let args = SearchJdArgs {
            base: SearchBaseArgs {
                query: Some("from-flags".to_string()),
                payload: Some(
                    r#"{"query":"from-json","sortBy":"DATE","sortOrder":"ASC"}"#.to_string(),
                ),
                ..SearchBaseArgs::default()
            },
            ..SearchJdArgs::default()
        };

        let payload = args.build_payload("JD").unwrap();
        assert_eq!(
            payload,
            r#"{"limit":10,"offset":0,"query":"from-json","sort":"DATE","sortOrder":"ASC"}"#
        );
    }

    #[test]
    fn search_payload_can_be_built_from_flags_only() {
        let args = SearchCrArgs {
            base: SearchBaseArgs {
                query: Some("občanský zákoník".to_string()),
                limit: Some(5),
                ..SearchBaseArgs::default()
            },
            types: vec!["Zákon".to_string()],
            current: true,
            ..SearchCrArgs::default()
        };

        let payload = args.build_payload("CR").unwrap();
        assert_eq!(
            payload,
            r#"{"limit":5,"offset":0,"query":"občanský zákoník","sortBy":"RELEVANCE","sortOrder":"DESC","typ":["Zákon"],"validNow":true}"#
        );
    }

    #[test]
    fn search_payload_includes_default_limit_offset_and_sort_when_omitted() {
        let args = SearchAllArgs {
            base: SearchBaseArgs {
                query: Some("insolvence".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchAllArgs::default()
        };

        let payload = args.build_payload("ALL").unwrap();
        assert_eq!(
            payload,
            r#"{"limit":10,"offset":0,"query":"insolvence","sort":"RELEVANCE","sortOrder":"DESC"}"#
        );
    }

    #[test]
    fn search_payload_must_be_object() {
        let error = load_search_payload(Some(r#"["not","an","object"]"#)).unwrap_err();
        assert!(matches!(error, CliError::SearchPayloadMustBeObject));
    }

    #[test]
    fn jd_boolean_filters_are_sent_only_when_asked_for() {
        // An omitted boolean must stay out of the payload: sending false would
        // narrow the search to decisions that lack the trait, not widen it.
        let asked = SearchJdArgs {
            base: SearchBaseArgs {
                query: Some("náhrada škody".to_string()),
                ..SearchBaseArgs::default()
            },
            has_legal_sentence: true,
            has_citex: true,
            ..SearchJdArgs::default()
        };

        let payload = asked.build_payload("JD").unwrap();
        assert!(payload.contains(r#""hasLegalSentence":true"#));
        assert!(payload.contains(r#""hasCitex":true"#));
        assert!(!payload.contains("isCommentedCaseLaw"));
        assert!(!payload.contains("isOfficialSource"));
    }

    #[test]
    fn a_date_only_search_is_refused_before_it_reaches_the_backend() {
        // Filters alone are not a search: the backend answers 400 without naming
        // the cause, so the date sweep a user reaches for has to be named here.
        let args = SearchJdArgs {
            issued: IssuedDateArgs {
                issued_from: Some("2026-05-26".to_string()),
                issued_to: Some("2026-06-01".to_string()),
            },
            ..SearchJdArgs::default()
        };

        let error = args.build_payload("JD").unwrap_err();
        let CliError::InvalidSearchArgument(message) = error else {
            panic!("a query-less search must be refused as an invalid argument");
        };
        assert!(message.contains("query is required"), "got: {message}");
    }

    #[test]
    fn a_query_supplied_through_the_json_payload_satisfies_the_check() {
        // The check reads the merged payload, not the flag, so the escape hatch
        // for callers who build their own body keeps working.
        let args = SearchJdArgs {
            base: SearchBaseArgs {
                payload: Some(r#"{"query":"náhrada škody"}"#.to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchJdArgs::default()
        };

        let payload = args.build_payload("JD").unwrap();
        assert!(payload.contains(r#""query":"náhrada škody""#));
    }

    #[test]
    fn a_blank_query_is_refused_the_same_as_a_missing_one() {
        let args = SearchJdArgs {
            base: SearchBaseArgs {
                query: Some("   ".to_string()),
                ..SearchBaseArgs::default()
            },
            ..SearchJdArgs::default()
        };

        assert!(matches!(
            args.build_payload("JD").unwrap_err(),
            CliError::InvalidSearchArgument(_)
        ));
    }

    #[test]
    fn limit_must_be_within_range() {
        let error = validate_search_payload(
            "ALL",
            &Map::from_iter([
                ("query".to_string(), Value::String("test".to_string())),
                ("limit".to_string(), Value::from(51_u64)),
            ]),
        )
        .unwrap_err();

        assert!(matches!(error, CliError::InvalidSearchArgument(_)));
    }

    #[test]
    fn cr_valid_now_and_valid_at_cannot_be_combined() {
        let error = validate_search_payload(
            "CR",
            &Map::from_iter([
                ("query".to_string(), Value::String("test".to_string())),
                ("validNow".to_string(), Value::Bool(true)),
                (
                    "validAt".to_string(),
                    Value::String("2025-01-01".to_string()),
                ),
            ]),
        )
        .unwrap_err();

        assert!(matches!(error, CliError::InvalidSearchArgument(_)));
    }
}

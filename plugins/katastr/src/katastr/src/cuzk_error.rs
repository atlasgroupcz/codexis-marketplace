//! Parsing the ČÚZK REST API's error responses.
//!
//! The API returns RFC 7807 ProblemDetail: a 400 carries an `errors` map, a
//! 404 carries `detail`, a 401 only `title`. Verified across 12 endpoints. The
//! single exception is a non-numeric value in the path, which the framework
//! catches and answers with HTML - which is why parsing returns an `Option`
//! and the caller has a fallback.

use std::collections::BTreeMap;

use serde::Deserialize;

/// How many characters of the body are kept when the response will not parse.
/// Without it a whole HTML page, CSS links and all, would reach the model's
/// context.
const RAW_BODY_LIMIT: usize = 500;

/// Truncates an error body to [`RAW_BODY_LIMIT`] characters. `None` means an
/// empty body - the caller then ends the message instead of leaving it hanging
/// on a colon.
///
/// Shared with RÚIAN: both services can answer with an HTML page instead of
/// JSON, and the cap holds for both for the same reason.
pub fn truncate(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }
    let snippet: String = trimmed.chars().take(RAW_BODY_LIMIT).collect();
    let ellipsis = if trimmed.chars().count() > RAW_BODY_LIMIT {
        "…"
    } else {
        ""
    };
    Some(format!("{snippet}{ellipsis}"))
}

#[derive(Debug, Deserialize)]
pub struct CuzkError {
    #[serde(default)]
    pub status: u16,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub errors: Option<BTreeMap<String, Vec<String>>>,
    #[serde(rename = "traceId", default)]
    pub trace_id: Option<String>,
}

/// Parses an error body. `None` means the body is not a ČÚZK ProblemDetail -
/// typically an HTML page from the routing layer.
///
/// Note: checking that the body starts with `{` is not enough. Every field
/// carries `#[serde(default)]`, so `{}` parses as valid. The body has to be
/// confirmed to carry actual content (title, detail or errors), or the error
/// would be lost silently in `human_message`.
pub fn parse(status: u16, body: &str) -> Option<CuzkError> {
    if !body.trim_start().starts_with('{') {
        return None;
    }
    let mut parsed: CuzkError = serde_json::from_str(body).ok()?;
    let has_content = !parsed.title.is_empty()
        || parsed.detail.is_some()
        || parsed.errors.as_ref().is_some_and(|e| !e.is_empty());
    if !has_content {
        return None;
    }
    if parsed.status == 0 {
        parsed.status = status;
    }
    Some(parsed)
}

/// The message shown to the user. When the body parses it is assembled from
/// the fields; otherwise the truncated body is printed, which is the older
/// behaviour.
pub fn human_message(status: u16, body: &str, path: &str) -> String {
    match parse(status, body) {
        Some(e) => {
            let mut msg = format!("ČÚZK API vrátilo HTTP {status} pro {path}");
            if let Some(errors) = &e.errors {
                let joined: Vec<String> = errors
                    .iter()
                    .map(|(field, msgs)| format!("{field}: {}", msgs.join(", ")))
                    .collect();
                msg.push_str(&format!(": {}", joined.join("; ")));
            } else if let Some(detail) = &e.detail {
                msg.push_str(&format!(": {detail}"));
            } else if !e.title.is_empty() {
                msg.push_str(&format!(": {}", e.title));
            }
            if let Some(trace) = &e.trace_id {
                msg.push_str(&format!(" [traceId {trace}]"));
            }
            msg
        }
        None => match truncate(body) {
            Some(snippet) => format!("ČÚZK API vrátilo HTTP {status} pro {path}: {snippet}"),
            None => format!("ČÚZK API vrátilo HTTP {status} pro {path}"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real body from /api/v1/Rizeni/PrijateDne?TypRizeni=PD
    const BODY_400: &str = r#"{"title":"One or more validation errors occurred.","status":400,"errors":{"TypRizeni":["Must be one of [V,Z,PGP]"]},"traceId":"00-1b7e36a9-34bc931a-00"}"#;

    // Real body from /api/v1/PravaStavby/Parcela/2034508804
    const BODY_404_NO_RIGHT: &str = r#"{"type":"https://tools.ietf.org/html/rfc9110#section-15.5.5","title":"Not found","status":404,"detail":"Parcela [ID:2034508804] was found but without PravoStavby","traceId":"00-e3c8c4d6-f3bb6e3f-00"}"#;

    // Real body from /api/v1/Parcely/999999999999
    const BODY_404_MISSING: &str = r#"{"type":"https://tools.ietf.org/html/rfc9110#section-15.5.5","title":"Not found","status":404,"detail":"No data found","traceId":"00-a54174c0-e4d1b7fb-00"}"#;

    #[test]
    fn parses_validation_errors_into_field_map() {
        let e = parse(400, BODY_400).expect("400 body must parse");
        assert_eq!(e.status, 400);
        let errors = e.errors.expect("400 carries an errors map");
        assert_eq!(errors["TypRizeni"], vec!["Must be one of [V,Z,PGP]"]);
    }

    #[test]
    fn parses_not_found_detail() {
        let e = parse(404, BODY_404_MISSING).expect("404 body must parse");
        assert_eq!(e.detail.as_deref(), Some("No data found"));
    }

    #[test]
    fn keeps_trace_id_for_support_tickets() {
        let e = parse(400, BODY_400).expect("400 body must parse");
        assert_eq!(e.trace_id.as_deref(), Some("00-1b7e36a9-34bc931a-00"));
    }

    #[test]
    fn returns_none_for_html_body() {
        assert!(parse(404, "<!DOCTYPE html><html lang=\"cs\">").is_none());
    }

    #[test]
    fn human_message_lists_field_errors() {
        let msg = human_message(400, BODY_400, "/api/v1/Rizeni/PrijateDne");
        assert!(msg.contains("TypRizeni"), "got: {msg}");
        assert!(msg.contains("Must be one of [V,Z,PGP]"), "got: {msg}");
    }

    #[test]
    fn human_message_uses_detail_for_not_found() {
        let msg = human_message(404, BODY_404_NO_RIGHT, "/api/v1/PravaStavby/Parcela/1");
        assert!(msg.contains("without PravoStavby"), "got: {msg}");
    }

    #[test]
    fn human_message_truncates_unparseable_body() {
        let html = "<html>".to_string() + &"x".repeat(2000);
        let msg = human_message(404, &html, "/api/v1/Parcely/abc");
        assert!(msg.len() < 700, "unparseable body must be truncated, len {}", msg.len());
    }

    #[test]
    fn json_without_problem_detail_fields_is_not_a_match() {
        assert!(parse(500, "{}").is_none());
        assert!(parse(502, r#"{"error":"upstream timeout"}"#).is_none());
    }

    #[test]
    fn unknown_json_body_still_reaches_the_user() {
        let msg = human_message(502, r#"{"error":"upstream timeout"}"#, "/api/v1/Parcely/1");
        assert!(msg.contains("upstream timeout"), "body must not be lost: {msg}");
    }

    #[test]
    fn empty_body_yields_no_snippet() {
        assert_eq!(truncate(""), None);
        assert_eq!(truncate("   "), None);
    }

    #[test]
    fn short_body_passes_through_unchanged() {
        assert_eq!(
            truncate("upstream timeout"),
            Some("upstream timeout".to_string())
        );
    }

    #[test]
    fn long_body_is_truncated_with_an_ellipsis() {
        let body = "x".repeat(RAW_BODY_LIMIT + 50);
        let snippet = truncate(&body).expect("non-empty body must yield a snippet");
        assert!(snippet.ends_with('…'), "got: {snippet}");
        assert_eq!(snippet.chars().count(), RAW_BODY_LIMIT + 1);
    }

    #[test]
    fn empty_body_does_not_leave_a_dangling_colon() {
        let msg = human_message(500, "", "/api/v1/Parcely/1");
        assert!(!msg.trim_end().ends_with(':'), "got: {msg}");
    }
}

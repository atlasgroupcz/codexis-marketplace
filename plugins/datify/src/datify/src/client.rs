//! Thin HTTP client for the Datify public REST API (v0).
//!
//! Deliberately a pass-through: Datify record values come in sixteen property
//! types keyed by property uuid, and modelling them here would add a second
//! schema to keep in sync with theirs for no gain. Request and response bodies
//! stay `serde_json::Value`; this module owns only auth, URL building, and
//! turning a non-2xx into an error the user can act on.

use std::time::Duration;

use serde_json::{json, Value};

use crate::config;
use crate::error::{DatifyError, Result};

const API_PREFIX: &str = "/v0";
/// The cap `records query` declares in Datify's own OpenAPI spec.
const QUERY_LIMIT_MAX: u32 = 50;
const USER_AGENT: &str = concat!("datify-cli/", env!("CARGO_PKG_VERSION"));

pub struct DatifyClient {
    base_url: String,
    token: String,
    http: reqwest::blocking::Client,
}

/// Where an uploaded file lands.
#[derive(Clone, Copy)]
pub enum UploadTarget<'a> {
    /// Straight onto one record's attachment property.
    RecordProperty {
        record_id: &'a str,
        property_id: &'a str,
    },
    /// Into a dataset's attachment pool.
    Dataset { dataset_id: &'a str },
    /// Unattached, to be referenced later.
    Standalone,
}

impl DatifyClient {
    /// Client from stored settings. Fails when no token is configured, naming
    /// where to get one.
    pub fn new() -> Result<Self> {
        let token = config::read_api_key();
        if token.is_empty() {
            return Err(DatifyError::KeyMissing(config::read_api_key_url()?));
        }
        Self::build(config::read_api_url()?, token)
    }

    /// Client for an explicit token - used to verify one before it is stored.
    pub fn with_token(token: &str) -> Result<Self> {
        let token = token.trim();
        if token.is_empty() {
            return Err(DatifyError::Config("API token je prázdný.".into()));
        }
        Self::build(config::read_api_url()?, token.to_string())
    }

    fn build(base_url: String, token: String) -> Result<Self> {
        let http = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| DatifyError::Network(e.to_string()))?;
        Ok(Self {
            base_url,
            token,
            http,
        })
    }

    // ── collections & datasets ───────────────────────────────────────────────

    /// Every collection the token can see, each with its datasets.
    pub fn collections(&self) -> Result<Value> {
        self.request(reqwest::Method::GET, "/collections", &[], None)
    }

    /// Dataset metadata including its properties - the id-to-name map every
    /// other call needs, since record values are keyed by property uuid.
    pub fn dataset(&self, dataset_id: &str) -> Result<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/datasets/{dataset_id}"),
            &[],
            None,
        )
    }

    // ── records ──────────────────────────────────────────────────────────────

    pub fn list_records(
        &self,
        dataset_id: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
        sort: Option<&str>,
    ) -> Result<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/datasets/{dataset_id}/records"),
            &page_query(limit, cursor, sort),
            None,
        )
    }

    /// Filtered records. The filter is always one flat `match`/`conditions`
    /// group, even for a single condition - Datify does not support nesting.
    pub fn query_records(
        &self,
        dataset_id: &str,
        filter: Value,
        limit: Option<u32>,
        cursor: Option<&str>,
        sort: Option<&str>,
    ) -> Result<Value> {
        check_query_limit(limit)?;
        self.request(
            reqwest::Method::POST,
            &format!("/datasets/{dataset_id}/records/query"),
            &page_query(limit, cursor, sort),
            Some(normalize_filter(filter)),
        )
    }

    pub fn get_record(&self, record_id: &str) -> Result<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/records/{record_id}"),
            &[],
            None,
        )
    }

    /// Create one record (`{"values": {...}}`) or up to ten
    /// (`{"records": [...]}`); the body is passed through as given.
    pub fn create_records(&self, dataset_id: &str, body: Value) -> Result<Value> {
        self.request(
            reqwest::Method::POST,
            &format!("/datasets/{dataset_id}/records"),
            &[],
            Some(body),
        )
    }

    /// PATCH merges the given properties; `replace` switches to PUT, which
    /// replaces the whole record.
    pub fn update_record(&self, record_id: &str, body: Value, replace: bool) -> Result<Value> {
        let method = if replace {
            reqwest::Method::PUT
        } else {
            reqwest::Method::PATCH
        };
        self.request(method, &format!("/records/{record_id}"), &[], Some(body))
    }

    pub fn delete_record(&self, record_id: &str) -> Result<Value> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/records/{record_id}"),
            &[],
            None,
        )
    }

    /// Delete several records in one call. Irreversible.
    ///
    /// The body is a bare array of ids. Wrapping them in `{"ids": [...]}` -
    /// which reads like the obvious shape - is rejected with HTTP 400
    /// "Failed to read request".
    pub fn delete_records(&self, record_ids: &[String]) -> Result<Value> {
        self.request(
            reqwest::Method::DELETE,
            "/records",
            &[],
            Some(json!(record_ids)),
        )
    }

    /// One property's value. Paginated, because relation and rollup properties
    /// can hold more items than fit in a record payload.
    pub fn record_property(
        &self,
        record_id: &str,
        property_id: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/records/{record_id}/properties/{property_id}"),
            &page_query(limit, cursor, None),
            None,
        )
    }

    // ── attachments & users ──────────────────────────────────────────────────

    pub fn get_attachment(&self, attachment_id: &str) -> Result<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/attachments/{attachment_id}"),
            &[],
            None,
        )
    }

    /// Upload a file. Datify exposes three targets, differing only in where the
    /// attachment lands: straight onto a record property, into a dataset's pool,
    /// or unattached. `ai` asks Datify to run its extraction over the file.
    pub fn upload_attachment(
        &self,
        target: UploadTarget<'_>,
        file_path: &std::path::Path,
        ai: bool,
    ) -> Result<Value> {
        let path = match target {
            UploadTarget::RecordProperty {
                record_id,
                property_id,
            } => format!("/records/{record_id}/{property_id}/attachments"),
            UploadTarget::Dataset { dataset_id } => format!("/datasets/{dataset_id}/attachments"),
            UploadTarget::Standalone => "/attachments".to_string(),
        };
        // The record endpoint does not declare the parameter, so it is never sent
        // there. The CLI refuses that combination outright; this keeps the
        // library honest for any other caller.
        let ai = ai && !matches!(target, UploadTarget::RecordProperty { .. });
        let query = if ai {
            vec![("ai".to_string(), "true".to_string())]
        } else {
            Vec::new()
        };

        // Form::file reads the file, sets the filename and picks the part's
        // media type from the extension, so none of that is this crate's job.
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", file_path)
            .map_err(|e| {
                DatifyError::Config(format!(
                    "Přílohu {} nelze přečíst: {e}",
                    file_path.display()
                ))
            })?;

        self.send(
            self.authorized(reqwest::Method::POST, &path)
                .query(&query)
                .multipart(form),
            &path,
        )
    }

    pub fn user(&self, user_id: &str) -> Result<Value> {
        self.request(
            reqwest::Method::GET,
            &format!("/users/{user_id}"),
            &[],
            None,
        )
    }

    // ── plumbing ─────────────────────────────────────────────────────────────

    fn url(&self, path: &str) -> String {
        format!("{}{}{}", self.base_url, API_PREFIX, path)
    }

    /// A request carrying this client's auth. Query values are escaped by
    /// `RequestBuilder::query`, which matters for cursors - they are opaque
    /// base64 and do carry `+`, `/` and `=`.
    fn authorized(&self, method: reqwest::Method, path: &str) -> reqwest::blocking::RequestBuilder {
        self.http
            .request(method, self.url(path))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json")
    }

    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<Value> {
        let mut req = self.authorized(method, path).query(query);
        if let Some(body) = body {
            req = req.json(&body);
        }
        self.send(req, path)
    }

    /// Run the request and turn the response into JSON, or into the error
    /// variant that says what to do about it.
    fn send(&self, req: reqwest::blocking::RequestBuilder, path: &str) -> Result<Value> {
        let response = req
            .send()
            .map_err(|e| DatifyError::Network(e.to_string()))?;
        let status = response.status().as_u16();
        let text = response
            .text()
            .map_err(|e| DatifyError::Network(e.to_string()))?;
        if !(200..300).contains(&status) {
            return Err(classify(status, &text, path));
        }
        // A 204 or an empty body is a success with nothing to say - report that
        // rather than failing to parse it.
        if text.trim().is_empty() {
            return Ok(json!({ "ok": true, "status": status }));
        }
        serde_json::from_str(&text).map_err(|e| DatifyError::Http {
            status,
            message: format!("neplatné JSON v odpovědi: {e}"),
        })
    }
}

/// `records query` accepts a limit of 1 to 50. Out of range, Datify answers
/// HTTP 500, which reads as a fault on their side rather than a value the
/// caller has to change - so the bound is enforced here, where the message can
/// say what it is. Only `query` declares a maximum; `list` does not.
fn check_query_limit(limit: Option<u32>) -> Result<()> {
    match limit {
        Some(limit) if !(1..=QUERY_LIMIT_MAX).contains(&limit) => Err(DatifyError::Config(
            format!("records query přijímá --limit 1 až {QUERY_LIMIT_MAX}, zadáno {limit}."),
        )),
        _ => Ok(()),
    }
}

/// Wrap a lone condition in the one-item `and` group Datify expects.
///
/// A filter is always `match` + `conditions`; a bare condition comes back as
/// *"filter must contain at least one condition"*, which reads as if the list
/// were empty and sends the caller looking in the wrong place. The two shapes
/// are disjoint - a condition names a `property`, a group carries `conditions` -
/// and a lone condition means exactly a one-item `and` group, so this is
/// lossless. Anything else is passed through for the server to judge.
fn normalize_filter(mut body: Value) -> Value {
    let Some(filter) = body.get_mut("filter") else {
        return body;
    };
    if filter.get("conditions").is_none() && filter.get("property").is_some() {
        let condition = filter.clone();
        *filter = json!({ "match": "and", "conditions": [condition] });
    }
    body
}

/// `limit`/`cursor`/`sort`, omitting whatever the caller left unset.
fn page_query(
    limit: Option<u32>,
    cursor: Option<&str>,
    sort: Option<&str>,
) -> Vec<(String, String)> {
    let mut query = Vec::new();
    if let Some(limit) = limit {
        query.push(("limit".to_string(), limit.to_string()));
    }
    if let Some(cursor) = cursor {
        query.push(("cursor".to_string(), cursor.to_string()));
    }
    if let Some(sort) = sort {
        query.push(("sort".to_string(), sort.to_string()));
    }
    query
}

/// Map a non-2xx onto the variant that tells the user what to do about it.
fn classify(status: u16, body: &str, path: &str) -> DatifyError {
    // Only the arms that carry it pay for reading the body.
    let message = || error_message(body).unwrap_or_else(|| body.chars().take(300).collect());
    match status {
        401 => DatifyError::Auth,
        403 => DatifyError::Permission,
        404 => DatifyError::NotFound(path.to_string()),
        400 | 422 => DatifyError::Validation {
            status,
            message: message(),
        },
        429 => DatifyError::RateLimit,
        _ => DatifyError::Http {
            status,
            message: message(),
        },
    }
}

/// Pull a human-readable message out of an error body, whatever key it uses.
fn error_message(body: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(body).ok()?;
    for key in ["message", "error", "detail", "title"] {
        if let Some(text) = parsed.get(key).and_then(|v| v.as_str()) {
            if !text.is_empty() {
                return Some(text.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::stub;

    fn client() -> DatifyClient {
        DatifyClient::build("https://api.datify.cz".into(), "t".into()).unwrap()
    }

    /// A client pointed at a one-shot stub, plus the channel its request lands on.
    fn stub_client(
        status: u16,
        body: &'static str,
    ) -> (
        DatifyClient,
        std::sync::mpsc::Receiver<crate::test_support::Seen>,
    ) {
        let (base, seen) = stub(status, body);
        (
            DatifyClient::build(base, "secret-token".into()).unwrap(),
            seen,
        )
    }

    #[test]
    fn url_carries_the_v0_prefix() {
        assert_eq!(
            client().url("/collections"),
            "https://api.datify.cz/v0/collections"
        );
    }

    #[test]
    fn page_query_keeps_only_what_the_caller_set() {
        assert!(page_query(None, None, None).is_empty());
        assert_eq!(
            page_query(Some(25), None, Some("createdAt,desc")),
            vec![
                ("limit".to_string(), "25".to_string()),
                ("sort".to_string(), "createdAt,desc".to_string()),
            ]
        );
    }

    #[test]
    fn classify_maps_statuses_to_actionable_variants() {
        for (status, expect_http_status) in [(401, 401), (403, 403), (404, 404), (429, 429)] {
            let err = classify(status, "", "/p");
            assert_eq!(err.http_status(), expect_http_status);
        }
    }

    #[test]
    fn classify_keeps_the_servers_message_for_a_validation_error() {
        let err = classify(422, r#"{"message":"Amount musí být číslo"}"#, "/p");
        assert!(err.to_string().contains("Amount musí být číslo"), "{err}");
    }

    #[test]
    fn error_message_reads_whichever_key_the_body_uses() {
        assert_eq!(
            error_message(r#"{"error":"boom"}"#).as_deref(),
            Some("boom")
        );
        assert_eq!(
            error_message(r#"{"detail":"nope"}"#).as_deref(),
            Some("nope")
        );
        assert_eq!(error_message("not json"), None);
        assert_eq!(error_message(r#"{"message":""}"#), None);
    }

    #[test]
    fn with_token_rejects_a_blank_token() {
        assert!(DatifyClient::with_token("   ").is_err());
    }

    // ── query limit ──────────────────────────────────────────────────────────

    #[test]
    fn a_query_limit_outside_datifys_range_is_refused_before_the_call() {
        // Datify answers HTTP 500 for these, which does not tell the caller the
        // bound - so the message has to.
        for limit in [0, 51, 1000] {
            let err = check_query_limit(Some(limit)).unwrap_err();
            assert!(err.to_string().contains("1 až 50"), "{err}");
        }
    }

    #[test]
    fn a_query_limit_inside_the_range_or_unset_is_allowed() {
        for limit in [None, Some(1), Some(25), Some(50)] {
            assert!(check_query_limit(limit).is_ok(), "{limit:?}");
        }
    }

    #[test]
    fn an_out_of_range_limit_never_reaches_the_network() {
        // The stub server would hang waiting for a request that must not happen.
        let client = DatifyClient::build("http://127.0.0.1:1".into(), "t".into()).unwrap();
        assert!(client
            .query_records("d1", json!({}), Some(51), None, None)
            .is_err());
    }

    // ── filter normalisation ─────────────────────────────────────────────────

    #[test]
    fn a_lone_condition_is_wrapped_in_a_one_item_and_group() {
        let normalized = normalize_filter(
            json!({ "filter": { "property": "Title", "text": { "contains": "Q3" } } }),
        );
        assert_eq!(
            normalized,
            json!({ "filter": { "match": "and", "conditions": [
                { "property": "Title", "text": { "contains": "Q3" } }
            ]}})
        );
    }

    #[test]
    fn an_existing_group_is_left_exactly_as_written() {
        // Including its `match` - wrapping an `or` group would silently turn it
        // into an `and`.
        let group = json!({ "filter": { "match": "or", "conditions": [
            { "property": "A", "text": { "contains": "x" } },
            { "property": "B", "text": { "contains": "y" } }
        ]}});
        assert_eq!(normalize_filter(group.clone()), group);
    }

    #[test]
    fn a_body_without_a_filter_is_passed_through() {
        assert_eq!(normalize_filter(json!({})), json!({}));
        // Not a condition either - let the server say what is wrong with it.
        assert_eq!(
            normalize_filter(json!({ "filter": { "nonsense": 1 } })),
            json!({ "filter": { "nonsense": 1 } })
        );
    }

    // ── request path, against a real socket ──────────────────────────────────

    #[test]
    fn a_get_sends_the_bearer_token_and_the_built_url() {
        let (client, seen) = stub_client(200, r#"[{"id":"c1"}]"#);
        let result = client.collections().unwrap();

        let seen = seen.recv().unwrap();
        assert_eq!(seen.method, "GET");
        assert_eq!(seen.target, "/v0/collections");
        assert_eq!(seen.auth, "Bearer secret-token");
        assert_eq!(result, json!([{ "id": "c1" }]));
    }

    #[test]
    fn a_query_posts_the_filter_to_the_dataset_path() {
        let (client, seen) = stub_client(200, r#"{"records":[]}"#);
        let filter = json!({ "filter": { "match": "and", "conditions": [
            { "property": "Title", "text": { "contains": "Q3" } }
        ]}});
        client
            .query_records("d1", filter.clone(), Some(10), None, None)
            .unwrap();

        let seen = seen.recv().unwrap();
        assert_eq!(seen.method, "POST");
        assert_eq!(seen.target, "/v0/datasets/d1/records/query?limit=10");
        assert_eq!(serde_json::from_str::<Value>(&seen.body).unwrap(), filter);
    }

    #[test]
    fn update_uses_patch_by_default_and_put_when_replacing() {
        for (replace, expected) in [(false, "PATCH"), (true, "PUT")] {
            let (client, seen) = stub_client(200, "{}");
            client.update_record("r1", json!({}), replace).unwrap();
            let seen = seen.recv().unwrap();
            assert_eq!(seen.method, expected);
            assert_eq!(seen.target, "/v0/records/r1");
        }
    }

    #[test]
    fn delete_many_sends_a_bare_array_of_ids() {
        let (client, seen) = stub_client(200, "{}");
        client
            .delete_records(&["r1".to_string(), "r2".to_string()])
            .unwrap();

        let seen = seen.recv().unwrap();
        assert_eq!(seen.method, "DELETE");
        assert_eq!(seen.target, "/v0/records");
        // Not `{"ids": [...]}` - Datify rejects that with HTTP 400.
        assert_eq!(
            serde_json::from_str::<Value>(&seen.body).unwrap(),
            json!(["r1", "r2"])
        );
    }

    #[test]
    fn a_rejected_token_surfaces_as_an_auth_error_not_a_raw_status() {
        let (client, _seen) = stub_client(401, r#"{"message":"bad token"}"#);
        let err = client.collections().unwrap_err();
        assert!(matches!(err, DatifyError::Auth), "{err:?}");
    }

    /// Every endpoint's method and URL, against a real socket.
    ///
    /// These are one-line delegations, which is exactly why they are worth
    /// pinning: a wrong verb or a typo in a path compiles, and only shows up as
    /// a 404 or a silent no-op against the live API. The paths here are the
    /// ones in Datify's OpenAPI spec.
    #[test]
    fn every_endpoint_hits_the_method_and_path_the_spec_declares() {
        /// Expected method, expected request target, and the call that makes it.
        type Call = (&'static str, &'static str, Box<dyn Fn(&DatifyClient)>);
        let calls: Vec<Call> = vec![
            (
                "GET",
                "/v0/datasets/d1",
                Box::new(|c| drop(c.dataset("d1"))),
            ),
            (
                "GET",
                "/v0/records/r1",
                Box::new(|c| drop(c.get_record("r1"))),
            ),
            (
                "POST",
                "/v0/datasets/d1/records",
                Box::new(|c| drop(c.create_records("d1", json!({})))),
            ),
            (
                "DELETE",
                "/v0/records/r1",
                Box::new(|c| drop(c.delete_record("r1"))),
            ),
            (
                "GET",
                "/v0/records/r1/properties/p1",
                Box::new(|c| drop(c.record_property("r1", "p1", None, None))),
            ),
            (
                "GET",
                "/v0/records/r1/properties/p1?limit=5",
                Box::new(|c| drop(c.record_property("r1", "p1", Some(5), None))),
            ),
            (
                "GET",
                "/v0/attachments/a1",
                Box::new(|c| drop(c.get_attachment("a1"))),
            ),
            ("GET", "/v0/users/u1", Box::new(|c| drop(c.user("u1")))),
        ];
        for (method, target, call) in calls {
            let (client, seen) = stub_client(200, "{}");
            call(&client);
            let seen = seen.recv().unwrap();
            assert_eq!(seen.method, method, "for {target}");
            assert_eq!(seen.target, target);
        }
    }

    #[test]
    fn each_upload_target_has_its_own_endpoint_and_only_two_take_ai() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        std::fs::write(&file, b"x").unwrap();

        // `ai` is meaningless on the record-property endpoint, which does not
        // declare the parameter - so it must not be sent even when asked for.
        let cases = [
            (
                UploadTarget::RecordProperty {
                    record_id: "r1",
                    property_id: "p1",
                },
                true,
                "/v0/records/r1/p1/attachments",
            ),
            (
                UploadTarget::Dataset { dataset_id: "d1" },
                true,
                "/v0/datasets/d1/attachments?ai=true",
            ),
            (
                UploadTarget::Dataset { dataset_id: "d1" },
                false,
                "/v0/datasets/d1/attachments",
            ),
            (UploadTarget::Standalone, false, "/v0/attachments"),
        ];
        for (target, ai, expected) in cases {
            let (client, seen) = stub_client(200, "{}");
            client.upload_attachment(target, &file, ai).unwrap();
            let seen = seen.recv().unwrap();
            assert_eq!(seen.method, "POST");
            assert_eq!(seen.target, expected);
        }
    }

    #[test]
    fn a_missing_record_names_the_path_that_was_not_found() {
        let (client, _seen) = stub_client(404, "{}");
        let err = client.get_record("nope").unwrap_err();
        assert!(err.to_string().contains("/records/nope"), "{err}");
    }

    #[test]
    fn a_cursor_is_escaped_in_the_request_target() {
        // Datify cursors are opaque base64 and do carry + / = in practice; left
        // raw, `+` would reach the server as a space.
        let (client, seen) = stub_client(200, "{}");
        client
            .list_records("d1", None, Some("eyJwYWdlIjox+a/b="), None)
            .unwrap();

        let seen = seen.recv().unwrap();
        assert_eq!(
            seen.target,
            "/v0/datasets/d1/records?cursor=eyJwYWdlIjox%2Ba%2Fb%3D"
        );
    }

    #[test]
    fn an_empty_body_is_a_success_not_a_parse_failure() {
        let (client, _seen) = stub_client(204, "");
        assert_eq!(
            client.delete_record("r1").unwrap(),
            json!({ "ok": true, "status": 204 })
        );
    }

    #[test]
    fn an_upload_declares_the_files_own_media_type_and_name() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("smlouva.pdf");
        std::fs::write(&file, b"\x00PDF-bytes\xff").unwrap();

        let (client, seen) = stub_client(200, "{}");
        client
            .upload_attachment(UploadTarget::Standalone, &file, true)
            .unwrap();

        let seen = seen.recv().unwrap();
        assert_eq!(seen.method, "POST");
        assert_eq!(seen.target, "/v0/attachments?ai=true");
        // Datify reports the declared type back on the attachment, so a blanket
        // application/octet-stream would leave every upload an unknown blob.
        assert!(
            seen.body.contains("Content-Type: application/pdf"),
            "{}",
            seen.body
        );
        assert!(
            seen.body.contains(r#"filename="smlouva.pdf""#),
            "{}",
            seen.body
        );
        // The raw bytes survive verbatim - a text-only round trip would corrupt them.
        assert!(seen.body.contains("PDF-bytes"), "{}", seen.body);
    }
}

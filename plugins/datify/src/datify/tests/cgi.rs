//! `datify-cgi` as a process.
//!
//! The daemon runs this in place for the settings page, handing it the request
//! through the environment and stdin. That contract - CGI headers, a blank line,
//! then JSON - is the whole interface, so these drive the real binary the way
//! the daemon does.

mod common;

use common::{stdout, stub, Sandbox};

const CGI: &str = env!("CARGO_BIN_EXE_datify-cgi");

/// Split a CGI response into its headers and its body.
fn parse_cgi(raw: &str) -> (String, serde_json::Value) {
    let (headers, body) = raw
        .split_once("\r\n\r\n")
        .or_else(|| raw.split_once("\n\n"))
        .unwrap_or_else(|| panic!("no header/body separator in: {raw}"));
    let parsed = serde_json::from_str(body.trim())
        .unwrap_or_else(|e| panic!("body is not JSON ({e}): {body}"));
    (headers.to_string(), parsed)
}

/// POST a JSON body to the handler, the way the daemon does.
fn post(sandbox: &Sandbox, body: &str, api_url: Option<&str>, token: Option<&str>) -> String {
    let output = sandbox.run_with_stdin(
        CGI,
        &[],
        api_url,
        token,
        &[
            ("REQUEST_METHOD", "POST".to_string()),
            ("CONTENT_LENGTH", body.len().to_string()),
        ],
        body,
    );
    stdout(&output)
}

#[test]
fn a_get_reports_the_state_without_revealing_the_token() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(CGI, &[], None, Some("abcdefghijkl"));
    let raw = stdout(&output);

    let (headers, body) = parse_cgi(&raw);
    assert!(headers.contains("application/json"), "{headers}");
    assert_eq!(body["configured"], true);
    assert_eq!(body["maskedKey"], "abcd****ijkl");
    assert!(
        !raw.contains("abcdefghijkl"),
        "the token was echoed back: {raw}"
    );
    assert_eq!(body["generateUrl"], "https://app.datify.cz/tokens");
}

#[test]
fn a_get_without_a_token_reports_it_as_not_configured() {
    let sandbox = Sandbox::new();
    let (_, body) = parse_cgi(&stdout(&sandbox.run(CGI, &[], None, None)));
    assert_eq!(body["configured"], false);
    assert_eq!(body["maskedKey"], "");
    assert_eq!(body["keySource"], "");
}

#[test]
fn saving_a_token_verifies_it_against_datify_first() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, r#"[{"id":"c1"},{"id":"c2"}]"#);

    let raw = post(
        &sandbox,
        r#"{"action":"save_api_key","apiKey":"tok-123456789"}"#,
        Some(&api),
        None,
    );

    assert_eq!(seen.recv().unwrap().target, "/v0/collections");
    let (_, body) = parse_cgi(&raw);
    assert_eq!(body["ok"], true);
    assert_eq!(body["collectionCount"], 2);
    assert!(std::fs::read_to_string(sandbox.env_file())
        .unwrap()
        .contains("tok-123456789"));
}

#[test]
fn a_token_datify_rejects_is_not_stored_and_the_status_says_why() {
    let sandbox = Sandbox::new();
    let (api, _seen) = stub(401, r#"{"message":"bad token"}"#);

    let raw = post(
        &sandbox,
        r#"{"action":"save_api_key","apiKey":"wrong"}"#,
        Some(&api),
        None,
    );

    let (headers, body) = parse_cgi(&raw);
    assert!(headers.contains("401"), "{headers}");
    assert!(body["error"].is_string(), "{body}");
    assert!(!sandbox.env_file().exists(), "a rejected token was stored");
}

#[test]
fn an_empty_token_is_refused_without_calling_datify() {
    let sandbox = Sandbox::new();
    // Unroutable port: reaching the network would fail differently.
    let raw = post(
        &sandbox,
        r#"{"action":"save_api_key","apiKey":"   "}"#,
        Some("http://127.0.0.1:1"),
        None,
    );

    let (headers, body) = parse_cgi(&raw);
    assert!(headers.contains("400"), "{headers}");
    assert!(
        body["error"].as_str().unwrap().contains("prázdný"),
        "{body}"
    );
}

#[test]
fn delete_forgets_the_stored_token() {
    let sandbox = Sandbox::new();
    let (api, _seen) = stub(200, "[]");
    post(
        &sandbox,
        r#"{"action":"save_api_key","apiKey":"tok-123456789"}"#,
        Some(&api),
        None,
    );

    let (_, body) = parse_cgi(&post(
        &sandbox,
        r#"{"action":"delete_api_key"}"#,
        None,
        None,
    ));
    assert_eq!(body["ok"], true);

    let (_, state) = parse_cgi(&stdout(&sandbox.run(CGI, &[], None, None)));
    assert_eq!(state["configured"], false);
}

#[test]
fn test_api_key_checks_the_stored_token() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, r#"[{"id":"c1"}]"#);

    let raw = post(
        &sandbox,
        r#"{"action":"test_api_key"}"#,
        Some(&api),
        Some("t"),
    );

    assert_eq!(seen.recv().unwrap().target, "/v0/collections");
    let (_, body) = parse_cgi(&raw);
    assert_eq!(body["ok"], true);
    assert_eq!(body["collectionCount"], 1);
}

#[test]
fn an_unknown_action_is_a_client_error_naming_the_action() {
    let sandbox = Sandbox::new();
    let (headers, body) = parse_cgi(&post(
        &sandbox,
        r#"{"action":"drop_everything"}"#,
        None,
        None,
    ));
    assert!(headers.contains("400"), "{headers}");
    assert!(
        body["error"].as_str().unwrap().contains("drop_everything"),
        "{body}"
    );
}

#[test]
fn a_malformed_body_is_a_client_error_not_a_crash() {
    let sandbox = Sandbox::new();
    let (headers, body) = parse_cgi(&post(&sandbox, "{not json", None, None));
    assert!(headers.contains("400"), "{headers}");
    assert!(body["error"].as_str().unwrap().contains("JSON"), "{body}");
}

#[test]
fn an_empty_body_is_treated_as_an_absent_action() {
    let sandbox = Sandbox::new();
    let (headers, _) = parse_cgi(&post(&sandbox, "", None, None));
    assert!(headers.contains("400"), "{headers}");
}

//! `datify-cli` as a process.
//!
//! main.rs is argument parsing and dispatch, and neither can be reached from a
//! unit test - a subcommand wired to the wrong client method, or an argument
//! that never arrives, compiles cleanly. These run the real binary against a
//! stub Datify and check what came out the other end.

mod common;

use common::{stderr, stdout, stub, stub_raw, Sandbox};

const CLI: &str = env!("CARGO_BIN_EXE_datify-cli");

#[test]
fn version_reports_the_crate_version() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(CLI, &["--version"], None, None);
    assert!(output.status.success());
    assert!(
        stdout(&output).contains(env!("CARGO_PKG_VERSION")),
        "{}",
        stdout(&output)
    );
}

#[test]
fn without_a_token_it_fails_and_names_where_to_get_one() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(CLI, &["collections"], None, None);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("app.datify.cz/tokens"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_subcommand_prints_datifys_json_unchanged() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, r#"[{"id":"c1","name":"Klienti"}]"#);

    let output = sandbox.run(CLI, &["collections"], Some(&api), Some("t"));

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(seen.recv().unwrap().target, "/v0/collections");
    let parsed: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(parsed[0]["name"], "Klienti");
}

/// Each subcommand has to reach the endpoint it claims to. A mis-wired arm is
/// invisible until it hits the live API.
#[test]
fn each_subcommand_reaches_its_own_endpoint() {
    let cases: &[(&[&str], &str, &str)] = &[
        (&["dataset", "d1"], "GET", "/v0/datasets/d1"),
        (&["records", "list", "d1"], "GET", "/v0/datasets/d1/records"),
        (&["records", "get", "r1"], "GET", "/v0/records/r1"),
        (&["records", "delete", "r1"], "DELETE", "/v0/records/r1"),
        (
            &["records", "delete-many", "r1", "r2"],
            "DELETE",
            "/v0/records",
        ),
        (&["attachment", "a1"], "GET", "/v0/attachments/a1"),
        (
            &["records", "attachments", "r1"],
            "GET",
            "/v0/records/r1/attachments",
        ),
        (&["user", "u1"], "GET", "/v0/users/u1"),
        (
            &["property", "r1", "p1"],
            "GET",
            "/v0/records/r1/properties/p1",
        ),
    ];
    for (args, method, target) in cases {
        let sandbox = Sandbox::new();
        let (api, seen) = stub(200, "{}");
        let output = sandbox.run(CLI, args, Some(&api), Some("t"));
        assert!(output.status.success(), "{args:?}: {}", stderr(&output));
        let seen = seen.recv().unwrap();
        assert_eq!(seen.method, *method, "{args:?}");
        assert_eq!(seen.target, *target, "{args:?}");
    }
}

#[test]
fn paging_flags_reach_the_query_string() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, "{}");

    let output = sandbox.run(
        CLI,
        &[
            "records",
            "list",
            "d1",
            "--limit",
            "5",
            "--sort",
            "createdAt,desc",
        ],
        Some(&api),
        Some("t"),
    );

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        seen.recv().unwrap().target,
        "/v0/datasets/d1/records?limit=5&sort=createdAt%2Cdesc"
    );
}

#[test]
fn update_sends_patch_and_put_with_replace() {
    for (args, method) in [
        (vec!["records", "update", "r1", "--data", "{}"], "PATCH"),
        (
            vec!["records", "update", "r1", "--data", "{}", "--replace"],
            "PUT",
        ),
    ] {
        let sandbox = Sandbox::new();
        let (api, seen) = stub(200, "{}");
        let output = sandbox.run(CLI, &args, Some(&api), Some("t"));
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(seen.recv().unwrap().method, method);
    }
}

#[test]
fn a_body_argument_reaches_datify_verbatim() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, "{}");

    let output = sandbox.run(
        CLI,
        &[
            "records",
            "create",
            "d1",
            "--data",
            r#"{"values":{"p1":"Q3"}}"#,
        ],
        Some(&api),
        Some("t"),
    );

    assert!(output.status.success(), "{}", stderr(&output));
    let seen = seen.recv().unwrap();
    assert_eq!(seen.target, "/v0/datasets/d1/records");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&seen.body).unwrap(),
        serde_json::json!({ "values": { "p1": "Q3" } })
    );
}

#[test]
fn a_body_can_arrive_on_stdin() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, "{}");

    // `-` is the documented way to pass a body too big for an argument.
    let output = sandbox.run_with_stdin(
        CLI,
        &["records", "create", "d1", "--data", "-"],
        Some(&api),
        Some("t"),
        &[],
        r#"{"values":{"p1":"ze stdin"}}"#,
    );
    assert!(output.status.success(), "{}", stderr(&output));

    let seen = seen.recv().unwrap();
    assert_eq!(seen.target, "/v0/datasets/d1/records");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&seen.body).unwrap(),
        serde_json::json!({ "values": { "p1": "ze stdin" } })
    );
}

#[test]
fn each_upload_flag_combination_picks_its_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("smlouva.pdf");
    std::fs::write(&file, b"%PDF-1.4").unwrap();
    let path = file.to_str().unwrap();

    let cases: &[(&[&str], &str)] = &[
        (
            &["--record", "r1", "--property", "p1"],
            "/v0/records/r1/p1/attachments",
        ),
        (&["--dataset", "d1"], "/v0/datasets/d1/attachments"),
        (
            &["--dataset", "d1", "--ai"],
            "/v0/datasets/d1/attachments?ai=true",
        ),
        (&[], "/v0/attachments"),
    ];
    for (flags, target) in cases {
        let sandbox = Sandbox::new();
        let (api, seen) = stub(200, "{}");
        let mut args = vec!["upload", path];
        args.extend_from_slice(flags);

        let output = sandbox.run(CLI, &args, Some(&api), Some("t"));

        assert!(output.status.success(), "{flags:?}: {}", stderr(&output));
        let seen = seen.recv().unwrap();
        assert_eq!(seen.method, "POST", "{flags:?}");
        assert_eq!(seen.target, *target, "{flags:?}");
        // The part carries the file's own media type, not a blanket binary one.
        assert!(
            seen.body.contains("application/pdf"),
            "{flags:?}: {}",
            seen.body
        );
    }
}

#[test]
fn conflicting_upload_targets_are_refused_by_argument_parsing() {
    let sandbox = Sandbox::new();
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a.pdf");
    std::fs::write(&file, b"x").unwrap();

    for flags in [
        // Each of these used to fall through to an unattached upload, which
        // creates an attachment nothing in this API can ever delete.
        vec!["--property", "p1"], // --record missing
        vec!["--record", "r1"],   // --property missing
        vec!["--record", "r1", "--property", "p1", "--dataset", "d1"], // two targets
        // The record endpoint takes no `ai` parameter, so accepting the flag
        // would drop it and imply an extraction that never ran.
        vec!["--record", "r1", "--property", "p1", "--ai"],
    ] {
        let mut args = vec!["upload", file.to_str().unwrap()];
        args.extend_from_slice(&flags);
        let output = sandbox.run(CLI, &args, Some("http://127.0.0.1:1"), Some("t"));
        assert!(!output.status.success(), "{flags:?} was accepted");
    }
}

#[test]
fn a_missing_upload_file_fails_before_any_request() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(
        CLI,
        &["upload", "/nope/chybi.pdf"],
        Some("http://127.0.0.1:1"),
        Some("t"),
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("chybi.pdf"), "{}", stderr(&output));
}

#[test]
fn a_query_posts_its_filter() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, r#"{"records":[]}"#);

    let output = sandbox.run(
        CLI,
        &[
            "records",
            "query",
            "d1",
            "--filter",
            r#"{"filter":{"match":"and","conditions":[{"property":"T","text":{"contains":"Q3"}}]}}"#,
            "--limit",
            "10",
        ],
        Some(&api),
        Some("t"),
    );

    assert!(output.status.success(), "{}", stderr(&output));
    let seen = seen.recv().unwrap();
    assert_eq!(seen.method, "POST");
    assert_eq!(seen.target, "/v0/datasets/d1/records/query?limit=10");
    assert!(seen.body.contains("conditions"), "{}", seen.body);
}

#[test]
fn a_malformed_body_is_refused_locally() {
    let sandbox = Sandbox::new();
    // No stub: a request must not be attempted at all.
    let output = sandbox.run(
        CLI,
        &["records", "create", "d1", "--data", "{not json"],
        Some("http://127.0.0.1:1"),
        Some("t"),
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("--data"), "{}", stderr(&output));
}

#[test]
fn an_out_of_range_query_limit_is_refused_before_the_call() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(
        CLI,
        &["records", "query", "d1", "--filter", "{}", "--limit", "51"],
        Some("http://127.0.0.1:1"),
        Some("t"),
    );
    assert!(!output.status.success());
    assert!(stderr(&output).contains("1 až 50"), "{}", stderr(&output));
}

#[test]
fn a_rejected_token_is_reported_as_an_auth_failure() {
    let sandbox = Sandbox::new();
    let (api, _seen) = stub(401, r#"{"message":"bad token"}"#);

    let output = sandbox.run(CLI, &["collections"], Some(&api), Some("t"));

    assert!(!output.status.success());
    assert!(stderr(&output).contains("401"), "{}", stderr(&output));
}

// ── settings ─────────────────────────────────────────────────────────────────

#[test]
fn set_key_stores_the_token_only_after_datify_accepts_it() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, "[]");

    let output = sandbox.run(
        CLI,
        &["settings", "set-key", "tok-123456789"],
        Some(&api),
        None,
    );

    assert!(output.status.success(), "{}", stderr(&output));
    // It verifies by listing collections before asking for anything to be stored.
    assert_eq!(seen.recv().unwrap().target, "/v0/collections");
    assert_eq!(
        sandbox.cdxctl_calls(),
        vec!["appenv datify set DATIFY_API_KEY tok-123456789"]
    );
    assert!(
        !sandbox.env_file().exists(),
        "the CLI wrote the credential file itself"
    );
}

#[test]
fn set_key_writes_nothing_when_datify_rejects_the_token() {
    let sandbox = Sandbox::new();
    let (api, _seen) = stub(401, "{}");

    let output = sandbox.run(
        CLI,
        &["settings", "set-key", "wrong-token"],
        Some(&api),
        None,
    );

    assert!(!output.status.success());
    assert!(
        sandbox.cdxctl_calls().is_empty(),
        "a rejected token was stored: {:?}",
        sandbox.cdxctl_calls()
    );
}

#[test]
fn show_masks_the_token_and_names_its_source() {
    let sandbox = Sandbox::new();
    let (api, _seen) = stub(200, "[]");
    sandbox.store_key("abcdefghijkl");

    let output = sandbox.run(CLI, &["settings", "show"], Some(&api), None);

    assert!(output.status.success(), "{}", stderr(&output));
    let parsed: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(parsed["configured"], true);
    assert_eq!(parsed["maskedKey"], "abcd****ijkl");
    // The whole token must never be printed.
    assert!(
        !stdout(&output).contains("abcdefghijkl"),
        "{}",
        stdout(&output)
    );
    assert!(parsed["keySource"].as_str().unwrap().ends_with(".env"));
}

#[test]
fn clear_key_asks_the_platform_to_remove_the_token() {
    let sandbox = Sandbox::new();
    let (api, _seen) = stub(200, "[]");
    sandbox.store_key("abcdefghijkl");

    let output = sandbox.run(CLI, &["settings", "clear-key"], Some(&api), None);

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        sandbox.cdxctl_calls(),
        vec!["appenv datify delete DATIFY_API_KEY"]
    );
}

/// The platform refusing the write must reach the user, not be swallowed into a
/// success that stores nothing.
#[test]
fn a_failing_platform_write_fails_set_key() {
    let sandbox = Sandbox::new();
    let (api, _seen) = stub(200, "[]");
    sandbox.fail_cdxctl(7);

    let output = sandbox.run(
        CLI,
        &["settings", "set-key", "tok-123456789"],
        Some(&api),
        None,
    );

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("Uložení API tokenu selhalo"),
        "{}",
        stderr(&output)
    );
}

/// The one command whose result is a file rather than stdout, so nothing else
/// proves the bytes survive the process boundary intact.
#[test]
fn download_writes_the_file_under_the_name_datify_reports() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub_raw(
        200,
        &[
            ("Content-Type", "image/png"),
            (
                "Content-Disposition",
                "attachment; filename=\"AAA 1234 foto1.png\"",
            ),
        ],
        b"\x89PNG\r\n\x1a\n\xff\x00",
    );
    let target = sandbox.home().join("prilohy");

    let output = sandbox.run(
        CLI,
        &["download", "a1", "--output-dir", target.to_str().unwrap()],
        Some(&api),
        Some("t"),
    );

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(seen.recv().unwrap().target, "/v0/attachments/a1/content");

    let report: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(report["filename"], "AAA 1234 foto1.png");
    assert_eq!(report["contentType"], "image/png");
    assert_eq!(report["size"], 10);

    let written = std::fs::read(report["path"].as_str().unwrap()).unwrap();
    assert_eq!(written, b"\x89PNG\r\n\x1a\n\xff\x00");
}

/// `--output-dir` is required: without it the agent would have to guess a
/// filename, which is the thing this command exists to avoid.
#[test]
fn download_without_an_output_dir_is_refused_by_argument_parsing() {
    let sandbox = Sandbox::new();
    let output = sandbox.run(CLI, &["download", "a1"], None, Some("t"));
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("--output-dir"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn test_confirms_a_working_token_against_datify() {
    let sandbox = Sandbox::new();
    let (api, seen) = stub(200, r#"[{"id":"c1"},{"id":"c2"}]"#);

    let output = sandbox.run(CLI, &["settings", "test"], Some(&api), Some("t"));

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(seen.recv().unwrap().target, "/v0/collections");
}

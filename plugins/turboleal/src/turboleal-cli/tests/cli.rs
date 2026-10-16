use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;
use mockito::{Matcher, Server};
use predicates::prelude::*;

const WATCH_SET: &str = "0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a10";
const SECOND_WATCH_SET: &str = "0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a0f";

const API_KEY: &str = "api-key-AbCdEfGhIjKlMnOpQrSt";

fn isolated(home: &Path) -> Command {
    let mut command = Command::cargo_bin("turboleal-cli").unwrap();
    command
        .env_remove("TURBOLEAL_API_KEY")
        .env_remove("CODEXIS_PLUGIN_TURBOLEAL_API_URL")
        .env_remove("CODEXIS_PUBLIC_DAEMON_URL")
        .env_remove("CODEXIS_PUBLIC_SESSION_ID")
        .env_remove("CODEXIS_PUBLIC_TOOL_CALL_ID")
        .env_remove("CODEXIS_PUBLIC_SUBAGENT_RUN_ID")
        .env("CODEXIS_PUBLIC_USER_HOME", home)
        .env("HOME", home);
    command
}

fn turboleal(server: &Server) -> Command {
    let mut command = isolated(&tempfile_dir());
    command.args(["--api-url", &server.url(), "--api-key", "test-token"]);
    command
}

fn me_mock(server: &mut Server, key: &str) -> mockito::Mock {
    server
        .mock("GET", "/api/v1/me")
        .match_header("authorization", format!("Bearer {key}").as_str())
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"00000000-0000-0000-0000-000000000001","email":"jan@atlasgroup.cz","name":"Jan"}"#)
        .create()
}

fn fake_cdxctl(directory: &Path) -> std::path::PathBuf {
    let bin = directory.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("cdxctl");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n",
            directory.join("cdxctl.log").display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

fn store_key(home: &Path, key: &str) {
    let apps = home.join(".cdx").join("apps").join("turboleal");
    fs::create_dir_all(&apps).unwrap();
    fs::write(apps.join(".env"), format!("TURBOLEAL_API_KEY={key}\n")).unwrap();
}

fn watch_set_json(id: &str, name: &str) -> String {
    format!(
        r#"{{"id":"{id}","name":"{name}","brief":"Jsem pekař","extractionStatus":"PENDING","createdAt":"2026-10-15T10:00:00Z"}}"#
    )
}

#[test]
fn watch_sets_list_all_follows_the_cursor() {
    let mut server = Server::new();
    let first = server
        .mock("GET", "/api/v1/watch-sets")
        .match_header("authorization", "Bearer test-token")
        .match_query(Matcher::Missing)
        .with_body(format!(
            r#"{{"items":[{}],"nextCursor":"{WATCH_SET}"}}"#,
            watch_set_json(WATCH_SET, "Pekárna")
        ))
        .create();
    let second = server
        .mock("GET", "/api/v1/watch-sets")
        .match_query(Matcher::UrlEncoded("after".into(), WATCH_SET.into()))
        .with_body(format!(
            r#"{{"items":[{}],"nextCursor":null}}"#,
            watch_set_json(SECOND_WATCH_SET, "Cukrárna")
        ))
        .create();

    turboleal(&server)
        .args(["watch-sets", "list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pekárna").and(predicate::str::contains("Cukrárna")));

    first.assert();
    second.assert();
}

#[test]
fn watch_sets_create_reads_the_brief_from_a_file() {
    let mut server = Server::new();
    let directory = tempfile_dir();
    let brief = directory.join("brief.txt");
    fs::write(&brief, "Jsem pekař a hlídám hygienu").unwrap();
    let create = server
        .mock("POST", "/api/v1/watch-sets")
        .match_body(Matcher::Json(
            serde_json::json!({"name": "Pekárna", "brief": "Jsem pekař a hlídám hygienu"}),
        ))
        .with_status(201)
        .with_body(watch_set_json(WATCH_SET, "Pekárna"))
        .create();

    turboleal(&server)
        .args(["watch-sets", "create", "--name", "Pekárna", "--brief"])
        .arg(format!("@{}", brief.display()))
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Created watch set {WATCH_SET}"
        )));

    create.assert();
}

#[test]
fn files_upload_keeps_the_folder_structure() {
    let mut server = Server::new();
    let directory = tempfile_dir();
    let contracts = directory.join("contracts").join("2026");
    fs::create_dir_all(&contracts).unwrap();
    fs::write(contracts.join("lease.pdf"), b"%PDF-1.7").unwrap();
    let upload = server
        .mock("POST", format!("/api/v1/watch-sets/{WATCH_SET}/files").as_str())
        .match_header("content-type", Matcher::Regex("multipart/form-data".into()))
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex(r#"filename="lease.pdf""#.into()),
            Matcher::Regex("application/pdf".into()),
            Matcher::Regex("bakery/contracts/2026".into()),
        ]))
        .with_status(202)
        .with_body(r#"[{"id":"0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a11","name":"lease.pdf","path":"bakery/contracts/2026/","status":"PENDING","error":null}]"#)
        .create();

    turboleal(&server)
        .args(["files", "upload", WATCH_SET, "--folder", "bakery"])
        .arg(directory.join("contracts"))
        .assert()
        .success()
        .stdout(predicate::str::contains("lease.pdf"));

    upload.assert();
}

#[test]
fn alerts_get_prints_json_when_asked() {
    let mut server = Server::new();
    server
        .mock("GET", "/api/v1/alerts/0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a12")
        .with_body(r#"{"id":"0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a12","watchSetId":"0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a10",
            "type":"AMENDED","documentId":"CR26785","citation":"89/2012 Sb.","sectionId":"0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a13",
            "partId":"paragraf2","label":"§ 2","effectiveFrom":"2027-01-01","detectedAt":"2026-10-15T12:00:00Z",
            "amendments":[{"documentId":"CR40001","citation":"123/2026 Sb."}],"diff":"```diff\n-old\n+new\n```",
            "references":[],"acknowledgedAt":null}"#)
        .create();

    turboleal(&server)
        .args([
            "-o",
            "json",
            "alerts",
            "get",
            "0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a12",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(r#""type": "AMENDED""#)
                .and(predicate::str::contains("123/2026 Sb.")),
        );
}

const ALERT: &str = "0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a12";

fn alert_mock(server: &mut Server, diff: &str, diff_markdown: &str) -> mockito::Mock {
    server
        .mock("GET", format!("/api/v1/alerts/{ALERT}").as_str())
        .with_body(format!(
            r#"{{"id":"{ALERT}","watchSetId":"{WATCH_SET}","type":"AMENDED","documentId":"CR26785",
            "citation":"89/2012 Sb.","sectionId":"0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a13","partId":"paragraf2",
            "label":"§ 2","effectiveFrom":"2027-01-01","detectedAt":"2026-10-15T12:00:00Z","amendments":[],
            "diff":{diff},"diffMarkdown":{diff_markdown},"diffHtml":"<p><del>tři</del><ins>pět</ins></p>",
            "references":[],"acknowledgedAt":null}}"#
        ))
        .create()
}

#[test]
fn alerts_get_prints_the_markdown_diff() {
    let mut server = Server::new();
    alert_mock(
        &mut server,
        "null",
        r#""Lhůta činí {--tři--}{++pět++} dny.""#,
    );

    turboleal(&server)
        .args(["alerts", "get", ALERT])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Lhůta činí {--tři--}{++pět++} dny.")
                .and(predicate::str::contains("<del>").not()),
        );
}

#[test]
fn alerts_get_keeps_the_markdown_diff_in_json() {
    let mut server = Server::new();
    alert_mock(
        &mut server,
        "null",
        r#""Lhůta činí {--tři--}{++pět++} dny.""#,
    );

    turboleal(&server)
        .args(["-o", "json", "alerts", "get", ALERT])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(r#""diffMarkdown": "Lhůta činí {--tři--}{++pět++} dny.""#)
                .and(predicate::str::contains("diffHtml").not()),
        );
}

#[test]
fn alerts_get_falls_back_to_the_legacy_diff() {
    let mut server = Server::new();
    alert_mock(&mut server, r#""```diff\n-old\n+new\n```""#, "null");

    turboleal(&server)
        .args(["alerts", "get", ALERT])
        .assert()
        .success()
        .stdout(predicate::str::contains("-old\n+new").and(predicate::str::contains("```").not()));
}

fn in_chat(command: &mut Command, home: &Path, daemon: &Server) {
    let secret = home.join(".cdx").join("env");
    fs::create_dir_all(&secret).unwrap();
    fs::write(secret.join("secret"), "Bearer daemon-token").unwrap();
    command
        .env("HOME", home)
        .env("CODEXIS_PUBLIC_DAEMON_URL", daemon.url())
        .env("CODEXIS_PUBLIC_SESSION_ID", "chat-1")
        .env("CODEXIS_PUBLIC_TOOL_CALL_ID", "call-1");
}

fn alert_with_codexis_link(server: &mut Server) {
    server
        .mock("GET", format!("/api/v1/alerts/{ALERT}").as_str())
        .with_body(format!(
            r#"{{"id":"{ALERT}","watchSetId":"{WATCH_SET}","type":"AMENDED","documentId":"CR26785",
            "citation":"89/2012 Sb.","sectionId":"0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a13","partId":"paragraf2",
            "label":"§ 2","effectiveFrom":"2027-01-01","detectedAt":"2026-10-15T12:00:00Z","amendments":[],
            "diffMarkdown":"x","codexisUrl":"https://app.codexis.cz/doc/CR26785#paragraf2",
            "references":[],"acknowledgedAt":null}}"#
        ))
        .create();
}

#[test]
fn alerts_get_attaches_only_the_turboleal_alert_to_the_chat() {
    let mut server = Server::new();
    alert_with_codexis_link(&mut server);
    server
        .mock("GET", format!("/api/v1/watch-sets/{WATCH_SET}/documents").as_str())
        .with_body(
            r#"[{"documentId":"CR26785","citation":"89/2012 Sb.","title":"Občanský zákoník",
            "status":"VALID","codexisUrl":"https://app.codexis.cz/doc/CR26785","wholeDocument":true,"sections":[]}]"#,
        )
        .create();
    let published = server
        .mock(
            "POST",
            "/rest/v1/plugin/chats/chat-1/tool-calls/call-1/sources",
        )
        .match_header("authorization", "Bearer daemon-token")
        .match_body(Matcher::PartialJson(serde_json::json!({
            "sources": [
                {
                    "title": "Upozornění v Turbolealu: Občanský zákoník (89/2012 Sb.), § 2",
                    "snippet": "Změna znění (novela). Účinnost od 1. 1. 2027."
                }
            ]
        })))
        .create();
    let home = tempfile_dir();
    let mut command = turboleal(&server);
    in_chat(&mut command, &home, &server);

    command
        .args(["-o", "json", "alerts", "get", ALERT])
        .assert()
        .success()
        .stdout(predicate::str::contains("codexis").not())
        .stdout(predicate::str::contains(r#""turbolealUrl": "http"#))
        .stdout(predicate::str::contains("/alerts?alerts="));

    published.assert();
}

#[test]
fn a_failed_source_publish_never_fails_the_command() {
    let mut server = Server::new();
    alert_with_codexis_link(&mut server);
    server
        .mock(
            "GET",
            format!("/api/v1/watch-sets/{WATCH_SET}/documents").as_str(),
        )
        .with_status(500)
        .create();
    server
        .mock(
            "POST",
            "/rest/v1/plugin/chats/chat-1/tool-calls/call-1/sources",
        )
        .with_status(500)
        .create();
    let home = tempfile_dir();
    let mut command = turboleal(&server);
    in_chat(&mut command, &home, &server);

    command
        .args(["-o", "json", "alerts", "get", ALERT])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""type": "AMENDED""#));
}

#[test]
fn unauthorized_response_explains_how_to_get_an_api_key() {
    let mut server = Server::new();
    server.mock("GET", "/api/v1/me").with_status(401).create();

    turboleal(&server).arg("me").assert().failure().stderr(
        predicate::str::contains("HTTP 401")
            .and(predicate::str::contains("API klíče"))
            .and(predicate::str::contains("turboleal-cli auth login")),
    );
}

#[test]
fn validation_problem_lists_the_invalid_fields() {
    let mut server = Server::new();
    server
        .mock("POST", "/api/v1/watch-sets")
        .with_status(400)
        .with_body(r#"{"title":"Bad Request","detail":"Invalid request content.","errors":[{"field":"name","message":"must not be blank"}]}"#)
        .create();

    turboleal(&server)
        .args(["watch-sets", "create", "--name", " ", "--brief", "x"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("name: must not be blank"));
}

#[test]
fn api_commands_without_a_key_point_to_the_api_keys_page() {
    isolated(&tempfile_dir())
        .args(["--api-url", "https://turboleal.example", "me"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("no Turboleal API key").and(predicate::str::contains(
                "https://turboleal.example/api-keys",
            )),
        );
}

#[test]
fn completions_work_without_a_key() {
    isolated(&tempfile_dir())
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef turboleal"));
}

fn tempfile_dir() -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("turboleal-cli-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn the_shared_codexis_token_is_never_sent() {
    let mut server = Server::new();
    let me = me_mock(&mut server, API_KEY);
    let home = tempfile_dir();
    store_key(&home, API_KEY);

    isolated(&home)
        .env("CODEXIS_USER_API_TOKEN", "shared-jwt")
        .env("CODEXIS_PLUGIN_TURBOLEAL_API_URL", server.url())
        .args(["-o", "json", "me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("jan@atlasgroup.cz"));
    me.assert();
}

#[test]
fn the_environment_key_wins_over_the_saved_one() {
    let mut server = Server::new();
    let me = me_mock(&mut server, "api-key-from-env");
    let home = tempfile_dir();
    store_key(&home, API_KEY);

    isolated(&home)
        .env("TURBOLEAL_API_KEY", "api-key-from-env")
        .args(["--api-url", &server.url(), "me"])
        .assert()
        .success();
    me.assert();
}

#[test]
fn auth_login_verifies_the_key_from_stdin_and_saves_it_through_cdxctl() {
    let mut server = Server::new();
    let me = me_mock(&mut server, API_KEY);
    let home = tempfile_dir();
    let bin = fake_cdxctl(&home);

    isolated(&home)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .args(["--api-url", &server.url(), "auth", "login"])
        .write_stdin(format!("{API_KEY}\n"))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Saved API key api-key-AbCd…")
                .and(predicate::str::contains("Jan <jan@atlasgroup.cz>"))
                .and(predicate::str::contains("EfGh").not()),
        );

    me.assert();
    assert_eq!(
        fs::read_to_string(home.join("cdxctl.log")).unwrap(),
        format!("appenv turboleal set TURBOLEAL_API_KEY {API_KEY}\n")
    );
}

#[test]
fn auth_login_does_not_save_a_key_turboleal_rejects() {
    let mut server = Server::new();
    server.mock("GET", "/api/v1/me").with_status(401).create();
    let home = tempfile_dir();
    let bin = fake_cdxctl(&home);

    isolated(&home)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .args(["--api-url", &server.url(), "auth", "login", API_KEY])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("rejected the API key")
                .and(predicate::str::contains("EfGh").not()),
        );

    assert!(!home.join("cdxctl.log").exists());
}

#[test]
fn auth_status_reports_the_masked_key_and_the_user() {
    let mut server = Server::new();
    let me = me_mock(&mut server, API_KEY);
    let home = tempfile_dir();
    store_key(&home, API_KEY);

    isolated(&home)
        .args(["--api-url", &server.url(), "-o", "json", "auth", "status"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(r#""maskedKey": "api-key-AbCd…""#)
                .and(predicate::str::contains(r#""email": "jan@atlasgroup.cz""#))
                .and(predicate::str::contains("EfGh").not()),
        );
    me.assert();
}

#[test]
fn auth_status_without_a_key_fails_with_the_api_keys_link() {
    isolated(&tempfile_dir())
        .args(["--api-url", "https://turboleal.example", "auth", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "https://turboleal.example/api-keys",
        ));
}

#[test]
fn auth_logout_deletes_the_saved_key_through_cdxctl() {
    let home = tempfile_dir();
    let bin = fake_cdxctl(&home);

    isolated(&home)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .args(["auth", "logout"])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(home.join("cdxctl.log")).unwrap(),
        "appenv turboleal delete TURBOLEAL_API_KEY\n"
    );
}

fn listed_alert(id: u32, overrides: serde_json::Value) -> serde_json::Value {
    let mut alert = serde_json::json!({
        "id": format!("00000000-0000-0000-0000-{id:012}"),
        "watchSetId": WATCH_SET,
        "type": "UPCOMING_AMENDMENT",
        "documentId": "CR1",
        "citation": "586/1992 Sb.",
        "sectionId": "00000000-0000-0000-0000-000000000200",
        "partId": "paragraf38g",
        "label": "§ 38g",
        "effectiveFrom": "2027-01-01",
        "detectedAt": "2026-10-15T12:00:00Z",
        "amendments": [{ "documentId": "CR2", "citation": "120/2026 Sb." }],
        "diff": null,
        "diffHtml": "<p>x</p>",
        "diffMarkdown": "(1) Přiznání podá, kdo měl příjmy nad {--50--}{++100++} 000 Kč.",
        "codexisUrl": "https://next.codexis.cz/doc/CR1",
        "references": [],
        "remindedAt": null,
        "acknowledgedAt": null
    });
    alert
        .as_object_mut()
        .unwrap()
        .extend(overrides.as_object().unwrap().clone());
    alert
}

fn documents_mock(server: &mut Server) -> mockito::Mock {
    server
        .mock("GET", format!("/api/v1/watch-sets/{WATCH_SET}/documents").as_str())
        .with_body(
            serde_json::json!([
                { "documentId": "CR1", "citation": "586/1992 Sb.", "title": "Zákon o daních z příjmů",
                  "status": "VALID", "wholeDocument": false,
                  "sections": [{ "id": "00000000-0000-0000-0000-000000000001", "partId": "paragraf38g", "label": "§ 38g" }] },
                { "documentId": "CR3", "citation": "262/2006 Sb.", "title": "Zákoník práce",
                  "status": "VALID", "wholeDocument": true, "sections": [] }
            ])
            .to_string(),
        )
        .create()
}

fn alerts_mock(server: &mut Server, alerts: Vec<serde_json::Value>) -> mockito::Mock {
    server
        .mock("GET", "/api/v1/alerts")
        .match_query(Matcher::Any)
        .with_body(serde_json::json!({ "items": alerts, "nextCursor": null }).to_string())
        .create()
}

fn summary_ids(stdout: &[u8]) -> Vec<String> {
    let summaries: serde_json::Value = serde_json::from_slice(stdout).unwrap();
    summaries
        .as_array()
        .unwrap()
        .iter()
        .map(|summary| summary["id"].as_str().unwrap()[35..].to_owned())
        .collect()
}

#[test]
fn alerts_list_summary_follows_the_cursor_names_the_law_and_links_only_turboleal() {
    let mut server = Server::new();
    let first = server
        .mock("GET", "/api/v1/alerts")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("acknowledged".into(), "false".into()),
            Matcher::UrlEncoded("limit".into(), "200".into()),
        ]))
        .match_request(|request| !request.path_and_query().contains("after="))
        .with_body(
            serde_json::json!({
                "items": [listed_alert(1, serde_json::json!({}))],
                "nextCursor": "00000000-0000-0000-0000-000000000001"
            })
            .to_string(),
        )
        .create();
    let second = server
        .mock("GET", "/api/v1/alerts")
        .match_query(Matcher::UrlEncoded(
            "after".into(),
            "00000000-0000-0000-0000-000000000001".into(),
        ))
        .with_body(
            serde_json::json!({ "items": [listed_alert(2, serde_json::json!({}))], "nextCursor": null })
                .to_string(),
        )
        .create();
    let documents = documents_mock(&mut server).expect(1);

    let assert = turboleal(&server)
        .args([
            "-o",
            "json",
            "alerts",
            "list",
            "--unacknowledged",
            "--summary",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("codexis").not());

    let summaries: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(summary_ids(&assert.get_output().stdout), ["1", "2"]);
    assert_eq!(summaries[0]["law"], "Zákon o daních z příjmů");
    assert_eq!(
        summaries[0]["type"],
        "připravovaná změna, zatím nenabyla účinnosti"
    );
    assert_eq!(
        summaries[0]["turbolealUrl"],
        format!(
            "{}/alerts?alerts=%7B%22acknowledged%22%3A%22all%22%2C%22alert%22%3A%22QWxlcnQ6MDAwMDAwMDAtMDAwMC0wMDAwLTAwMDAtMDAwMDAwMDAwMDAx%22%7D",
            server.url()
        )
    );
    assert_eq!(summaries[0]["overview"]["items"][0]["kind"], "changed");
    first.assert();
    second.assert();
    documents.assert();
}

#[test]
fn alerts_list_filters_by_law_section_dates_and_type() {
    let mut server = Server::new();
    alerts_mock(
        &mut server,
        vec![
            listed_alert(1, serde_json::json!({})),
            listed_alert(2, serde_json::json!({ "label": "§ 38ga" })),
            listed_alert(
                3,
                serde_json::json!({ "effectiveFrom": "2026-07-01", "type": "AMENDED" }),
            ),
            listed_alert(
                4,
                serde_json::json!({ "documentId": "CR3", "citation": "262/2006 Sb.", "label": "§ 35" }),
            ),
        ],
    );
    documents_mock(&mut server);
    let ids = |args: &[&str]| {
        let assert = turboleal(&server)
            .args(["-o", "json", "alerts", "list", "--summary"])
            .args(args)
            .assert()
            .success();
        summary_ids(&assert.get_output().stdout)
    };

    assert_eq!(
        ids(&["--law", "danich z prijmu", "--section", "38g"]),
        ["3", "1"]
    );
    assert_eq!(
        ids(&[
            "--effective-from",
            "2026-01-01",
            "--effective-to",
            "2026-12-31"
        ]),
        ["3"]
    );
    assert_eq!(
        ids(&["--type", "UPCOMING_AMENDMENT", "--law", "zákoník práce"]),
        ["4"]
    );
    assert_eq!(
        ids(&[
            "--detected-from",
            "2026-10-15",
            "--detected-to",
            "2026-10-15"
        ]),
        ["3", "1", "2", "4"]
    );
    assert_eq!(
        ids(&["--detected-from", "2026-10-16"]),
        Vec::<String>::new()
    );
}

#[test]
fn alerts_list_with_a_filter_keeps_the_page_shape_without_summary() {
    let mut server = Server::new();
    alerts_mock(
        &mut server,
        vec![
            listed_alert(1, serde_json::json!({})),
            listed_alert(2, serde_json::json!({ "label": "§ 38ga" })),
        ],
    );
    documents_mock(&mut server);

    let assert = turboleal(&server)
        .args(["-o", "json", "alerts", "list", "--section", "§ 38g"])
        .assert()
        .success();

    let page: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["lawTitle"], "Zákon o daních z příjmů");
    assert_eq!(page["nextCursor"], serde_json::Value::Null);
}

#[test]
fn alerts_list_rejects_a_date_that_is_not_iso() {
    let server = Server::new();

    turboleal(&server)
        .args(["alerts", "list", "--effective-to", "31. 12. 2026"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("YYYY-MM-DD"));
}

#[test]
fn alerts_list_cites_every_returned_alert_in_the_chat() {
    let mut server = Server::new();
    alerts_mock(
        &mut server,
        (1..=12)
            .map(|id| listed_alert(id, serde_json::json!({})))
            .collect(),
    );
    documents_mock(&mut server);
    let published = server
        .mock(
            "POST",
            "/rest/v1/plugin/chats/chat-1/tool-calls/call-1/sources",
        )
        .match_request(|request| {
            let body: serde_json::Value = serde_json::from_slice(request.body().unwrap()).unwrap();
            body["sources"]
                .as_array()
                .is_some_and(|sources| sources.len() == 12)
        })
        .create();
    let home = tempfile_dir();
    let mut command = turboleal(&server);
    in_chat(&mut command, &home, &server);

    command
        .args(["-o", "json", "alerts", "list", "--summary"])
        .assert()
        .success();

    published.assert();
}

#[test]
fn alerts_get_summary_keeps_the_alert_when_its_law_name_cannot_be_looked_up() {
    let mut server = Server::new();
    alert_mock(
        &mut server,
        "null",
        r#""(1) Lhůta činí {--tři--}{++pět++} dny.""#,
    );

    let assert = turboleal(&server)
        .args(["-o", "json", "alerts", "get", ALERT, "--summary"])
        .assert()
        .success();

    let summary: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    assert_eq!(summary["law"], serde_json::Value::Null);
    assert_eq!(summary["citation"], "89/2012 Sb.");
    assert_eq!(summary["effectiveFrom"], "1. 1. 2027");
    assert_eq!(summary["overview"]["items"][0]["where"], "odst. 1");
    assert_eq!(
        summary["diffMarkdown"],
        "(1) Lhůta činí {--tři--}{++pět++} dny."
    );
}

#[test]
fn watched_finds_the_watch_sets_that_watch_a_law_or_section() {
    let mut server = Server::new();
    server
        .mock("GET", "/api/v1/watch-sets")
        .match_query(Matcher::UrlEncoded("limit".into(), "200".into()))
        .with_body(format!(
            r#"{{"items":[{},{}],"nextCursor":null}}"#,
            watch_set_json(WATCH_SET, "Pekárna"),
            watch_set_json(SECOND_WATCH_SET, "IT")
        ))
        .create();
    documents_mock(&mut server);
    server
        .mock(
            "GET",
            format!("/api/v1/watch-sets/{SECOND_WATCH_SET}/documents").as_str(),
        )
        .with_body("[]")
        .create();
    let watched = |args: &[&str]| {
        let assert = turboleal(&server)
            .args(["-o", "json", "watched"])
            .args(args)
            .assert()
            .success();
        serde_json::from_slice::<serde_json::Value>(&assert.get_output().stdout).unwrap()
    };

    let income = watched(&["--law", "586/1992", "--section", "38g"]);
    let labour = watched(&["--law", "zakonik prace", "--section", "§ 52"]);
    let missing = watched(&["--law", "586/1992", "--section", "§ 7a"]);

    assert_eq!(income.as_array().unwrap().len(), 1);
    assert_eq!(income[0]["watchSetName"], "Pekárna");
    assert_eq!(income[0]["sections"], serde_json::json!(["§ 38g"]));
    assert!(
        income[0]["turbolealUrl"]
            .as_str()
            .unwrap()
            .contains("/watch-sets/")
    );
    assert_eq!(labour[0]["wholeDocument"], true);
    assert_eq!(missing, serde_json::json!([]));
}

use std::fs;

use assert_cmd::Command;
use mockito::{Matcher, Server};
use predicates::prelude::*;

const WATCH_SET: &str = "0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a10";
const SECOND_WATCH_SET: &str = "0199e7a4-3c2f-7b8e-9a1d-5f3c2e8b4a0f";

fn turboleal(server: &Server) -> Command {
    let mut command = Command::cargo_bin("turboleal-cli").unwrap();
    command.env_remove("CODEXIS_USER_API_TOKEN").args([
        "--api-url",
        &server.url(),
        "--token",
        "test-token",
    ]);
    command
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

#[test]
fn unauthorized_response_explains_the_token() {
    let mut server = Server::new();
    server.mock("GET", "/api/v1/me").with_status(401).create();

    turboleal(&server).arg("me").assert().failure().stderr(
        predicate::str::contains("HTTP 401")
            .and(predicate::str::contains("CODEXIS_USER_API_TOKEN")),
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
fn api_commands_need_a_token() {
    Command::cargo_bin("turboleal-cli")
        .unwrap()
        .env_remove("CODEXIS_USER_API_TOKEN")
        .arg("me")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no access token"));
}

#[test]
fn completions_work_without_a_token() {
    Command::cargo_bin("turboleal-cli")
        .unwrap()
        .env_remove("CODEXIS_USER_API_TOKEN")
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
fn runtime_token_from_environment_is_sent_once_as_bearer() {
    let mut server = Server::new();
    let me = server
        .mock("GET", "/api/v1/me")
        .match_header("authorization", "Bearer runtime-jwt")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"00000000-0000-0000-0000-000000000001","email":"jan@atlasgroup.cz","name":"Jan"}"#)
        .create();

    Command::cargo_bin("turboleal-cli")
        .unwrap()
        .env("CODEXIS_USER_API_TOKEN", "Bearer runtime-jwt")
        .env("CODEXIS_PLUGIN_TURBOLEAL_API_URL", server.url())
        .args(["-o", "json", "me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("jan@atlasgroup.cz"));
    me.assert();
}

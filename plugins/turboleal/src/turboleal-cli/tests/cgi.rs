use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use mockito::Server;

const API_KEY: &str = "api-key-AbCdEfGhIjKlMnOpQrSt";

fn home() -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("turboleal-cgi-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn cgi(home: &Path, api_url: &str) -> Command {
    let bin = home.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let script = bin.join("cdxctl");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\n",
            home.join("cdxctl.log").display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let mut command = Command::cargo_bin("turboleal-cgi").unwrap();
    command
        .env_remove("TURBOLEAL_API_KEY")
        .env("CODEXIS_PLUGIN_TURBOLEAL_API_URL", api_url)
        .env("CODEXIS_PUBLIC_USER_HOME", home)
        .env("HOME", home)
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()));
    command
}

fn body(raw: &[u8]) -> (String, serde_json::Value) {
    let raw = String::from_utf8(raw.to_vec()).unwrap();
    let (headers, body) = raw.split_once("\r\n\r\n").unwrap();
    (headers.to_owned(), serde_json::from_str(body).unwrap())
}

fn post(home: &Path, api_url: &str, payload: &str) -> (String, serde_json::Value, String) {
    let output = cgi(home, api_url)
        .env("REQUEST_METHOD", "POST")
        .env("CONTENT_LENGTH", payload.len().to_string())
        .write_stdin(payload)
        .output()
        .unwrap();
    let raw = String::from_utf8(output.stdout.clone()).unwrap();
    let (headers, body) = body(&output.stdout);
    (headers, body, raw)
}

#[test]
fn a_get_reports_where_to_create_a_key() {
    let home = home();
    let output = cgi(&home, "https://turboleal.example").output().unwrap();

    let (headers, body) = body(&output.stdout);
    assert!(headers.contains("application/json"), "{headers}");
    assert_eq!(body["configured"], false);
    assert_eq!(body["generateUrl"], "https://turboleal.example/api-keys");
}

#[test]
fn saving_a_key_verifies_it_with_turboleal_and_stores_it_through_cdxctl() {
    let mut server = Server::new();
    let me = server
        .mock("GET", "/api/v1/me")
        .match_header("authorization", format!("Bearer {API_KEY}").as_str())
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"00000000-0000-0000-0000-000000000001","email":"jan@atlasgroup.cz","name":"Jan"}"#)
        .create();
    let home = home();

    let (_, body, raw) = post(
        &home,
        &server.url(),
        &format!(r#"{{"action":"save_api_key","apiKey":"{API_KEY}"}}"#),
    );

    me.assert();
    assert_eq!(body["ok"], true);
    assert_eq!(body["maskedKey"], "api-key-AbCd…");
    assert!(!raw.contains("EfGhIjKl"), "{raw}");
    assert_eq!(
        fs::read_to_string(home.join("cdxctl.log")).unwrap(),
        format!("appenv turboleal set TURBOLEAL_API_KEY {API_KEY}\n")
    );
}

#[test]
fn a_rejected_key_is_reported_and_not_stored() {
    let mut server = Server::new();
    server.mock("GET", "/api/v1/me").with_status(401).create();
    let home = home();

    let (headers, body, _) = post(
        &home,
        &server.url(),
        &format!(r#"{{"action":"save_api_key","apiKey":"{API_KEY}"}}"#),
    );

    assert!(headers.starts_with("Status: 422"), "{headers}");
    assert_eq!(body["ok"], false);
    assert!(!home.join("cdxctl.log").exists());
}

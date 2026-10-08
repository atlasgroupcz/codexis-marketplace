#[path = "../src/test_support.rs"]
mod test_support;

use serde_json::{json, Value};
use std::fs;
use std::process::{Command, Output};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tempfile::TempDir;
use test_support::{stub_daemon, stub_daemon_responses};

const PROJECT_UUID: &str = "8212f86c-fcef-45a6-9e05-fd37c56551fe";
const PROJECT_ID: &str = "B1Byb2plY3QkODIxMmY4NmMtZmNlZi00NWE2LTllMDUtZmQzN2M1NjU1MWZl";
const PROJECT_NAME: &str = "Právní agenda";

fn command(address: &str) -> (Command, TempDir) {
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join(".cdx/env")).unwrap();
    fs::write(home.path().join(".cdx/env/secret"), "test-vm-token\n").unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_cdxctl"));
    command
        .env_clear()
        .env("HOME", home.path())
        .env("CODEXIS_PUBLIC_USER_HOME", home.path())
        .env(
            "CODEXIS_PUBLIC_DAEMON_URL",
            format!("http://{address}/graphql"),
        );
    for name in ["LLVM_PROFILE_FILE", "CARGO_LLVM_COV", "LD_LIBRARY_PATH"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    (command, home)
}

fn run(address: &str, args: &[&str]) -> Output {
    let (mut command, _home) = command(address);
    command.args(args).output().unwrap()
}

fn request(requests: &Receiver<String>) -> Value {
    let raw = requests.recv_timeout(Duration::from_secs(3)).unwrap();
    let (headers, body) = raw.split_once("\r\n\r\n").unwrap();
    assert!(headers.starts_with("POST /graphql HTTP/1.1\r\n"));
    assert!(headers
        .lines()
        .any(|line| line.eq_ignore_ascii_case("authorization: Bearer test-vm-token")));
    serde_json::from_str(body).unwrap()
}

fn success(output: &Output) {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

fn failure(output: &Output, code: i32, message: &str) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(message),
        "{output:?}"
    );
}

fn hits() -> Value {
    json!([
        {"id":"chat-b","title":"Nájem","snippet":"<mark>nájem</mark>\nVýpověď","modifiedAt":"2026-10-08T10:00:00Z","automation":true,"project":{"id":PROJECT_ID,"name":PROJECT_NAME}},
        {"id":"chat-a","title":null,"snippet":null,"modifiedAt":null,"automation":false,"project":null}
    ])
}

fn projects() -> Value {
    json!({"data":{"projects":[{"id":PROJECT_ID,"name":PROJECT_NAME}]}})
}

#[test]
fn global_search_prints_only_json_and_preserves_backend_results() {
    let (address, requests) = stub_daemon_responses(vec![(
        "HTTP/1.1 200 OK",
        json!({"data":{"search":hits()}}).to_string(),
    )]);

    let output = run(&address, &["chat", "search", "  nájem OR \"výpověď\"  "]);

    success(&output);
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        hits()
    );
    assert_eq!(
        request(&requests)["variables"],
        json!({"query":"  nájem OR \"výpověď\"  ","limit":20,"offset":0})
    );
}

#[test]
fn project_name_uuid_and_node_id_forward_the_same_scope_and_pagination() {
    let scoped_hits = json!([hits()[0]]);
    for identifier in [PROJECT_NAME, PROJECT_UUID, PROJECT_ID] {
        let (address, requests) = stub_daemon_responses(vec![
            ("HTTP/1.1 200 OK", projects().to_string()),
            (
                "HTTP/1.1 200 OK",
                json!({"data":{"search":scoped_hits}}).to_string(),
            ),
        ]);

        let output = run(
            &address,
            &[
                "chat",
                "search",
                "nájem",
                "--project",
                identifier,
                "--limit",
                "1",
                "--offset",
                "7",
            ],
        );

        success(&output);
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            scoped_hits
        );
        let lookup = request(&requests);
        assert!(lookup["query"].as_str().unwrap().contains("projects"));
        assert_eq!(lookup["variables"], json!({}));
        assert_eq!(
            request(&requests)["variables"],
            json!({"query":"nájem","projectId":PROJECT_ID,"limit":1,"offset":7})
        );
    }
}

#[test]
fn project_names_remain_case_sensitive() {
    let (address, requests) =
        stub_daemon_responses(vec![("HTTP/1.1 200 OK", projects().to_string())]);

    let output = run(
        &address,
        &["chat", "search", "nájem", "--project", "právní agenda"],
    );

    failure(&output, 4, "No such project: právní agenda");
    request(&requests);
    assert!(requests.try_recv().is_err());
}

#[test]
fn pagination_accepts_its_upper_boundaries() {
    let (address, requests) = stub_daemon("HTTP/1.1 200 OK", r#"{"data":{"search":[]}}"#);

    let output = run(
        &address,
        &[
            "chat",
            "search",
            "nájem",
            "--limit",
            "100",
            "--offset",
            "2147483647",
        ],
    );

    success(&output);
    assert_eq!(
        request(&requests)["variables"],
        json!({"query":"nájem","limit":100,"offset":2147483647})
    );
}

#[test]
fn uuid_project_identity_takes_precedence_over_names_in_any_letter_case() {
    for identifier in [
        PROJECT_UUID,
        "8212F86C-FCEF-45A6-9E05-FD37C56551FE",
        "8212f86C-fCeF-45a6-9E05-Fd37C56551fE",
    ] {
        let (address, requests) = stub_daemon_responses(vec![
            (
                "HTTP/1.1 200 OK",
                json!({"data":{"projects":[
                    {"id":"B1Byb2plY3QkYjM1YmIwOGYtY2RiMy00MWFmLWI1MWItYWQ1MjdkNjNjMzJm","name":identifier},
                    {"id":PROJECT_ID,"name":PROJECT_NAME}
                ]}}).to_string(),
            ),
            ("HTTP/1.1 200 OK", r#"{"data":{"search":[]}}"#.into()),
        ]);

        let output = run(
            &address,
            &["chat", "search", "nájem", "--project", identifier],
        );

        success(&output);
        request(&requests);
        assert_eq!(
            request(&requests)["variables"]["projectId"],
            PROJECT_ID,
            "{identifier}"
        );
    }
}

#[test]
fn empty_search_is_successful_json() {
    let (address, _requests) = stub_daemon("HTTP/1.1 200 OK", r#"{"data":{"search":[]}}"#);

    let output = run(&address, &["chat", "search", "absent"]);

    success(&output);
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!([])
    );
}

#[test]
fn table_output_displays_project_names_and_unassigned_chats() {
    let (address, _requests) = stub_daemon_responses(vec![(
        "HTTP/1.1 200 OK",
        json!({"data":{"search":[
            {"id":"chat-b","title":"Nájem","project":{"id":PROJECT_ID,"name":PROJECT_NAME}},
            {"id":"chat-a","title":"Výpověď","project":null}
        ]}})
        .to_string(),
    )]);

    let output = run(&address, &["chat", "search", "nájem", "--table"]);

    success(&output);
    let text = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4, "{text}");
    assert!(lines[0].contains("PROJECT"), "{text}");
    assert!(lines[2].contains("chat-b"), "{text}");
    assert!(lines[2].contains(PROJECT_NAME), "{text}");
    assert!(lines[3].contains("chat-a"), "{text}");
    assert_eq!(lines[3].split_whitespace().nth(1), Some("-"), "{text}");
    assert!(!text.contains(PROJECT_ID), "{text}");
}

#[test]
fn missing_or_ambiguous_project_fails_without_printing_results() {
    let other_id = "B1Byb2plY3QkYjM1YmIwOGYtY2RiMy00MWFmLWI1MWItYWQ1MjdkNjNjMzJm";
    for (projects, expected) in [
        (json!([]), "No such project"),
        (
            json!([
                {"id":PROJECT_ID,"name":PROJECT_NAME},
                {"id":other_id,"name":PROJECT_NAME}
            ]),
            "Ambiguous project",
        ),
    ] {
        let (address, requests) = stub_daemon_responses(vec![(
            "HTTP/1.1 200 OK",
            json!({"data":{"projects":projects}}).to_string(),
        )]);

        let output = run(
            &address,
            &["chat", "search", "nájem", "--project", PROJECT_NAME],
        );

        failure(&output, 4, expected);
        assert!(request(&requests)["query"]
            .as_str()
            .unwrap()
            .contains("projects"));
        assert!(requests.try_recv().is_err());
        if expected == "Ambiguous project" {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains(PROJECT_ID), "{stderr}");
            assert!(stderr.contains(other_id), "{stderr}");
        }
    }
}

#[test]
fn graphql_errors_fail_even_when_partial_results_are_present() {
    let (address, _requests) = stub_daemon_responses(vec![(
        "HTTP/1.1 200 OK",
        json!({"data":{"search":hits()},"errors":[{"message":"Search unavailable"}]}).to_string(),
    )]);

    let output = run(&address, &["chat", "search", "nájem"]);

    failure(&output, 3, "Search unavailable");
}

#[test]
fn project_lookup_errors_stop_the_search() {
    let (address, requests) = stub_daemon(
        "HTTP/1.1 200 OK",
        r#"{"data":{"projects":[]},"errors":[{"message":"Project access denied"}]}"#,
    );

    let output = run(
        &address,
        &["chat", "search", "nájem", "--project", PROJECT_NAME],
    );

    failure(&output, 3, "Project access denied");
    assert!(request(&requests)["query"]
        .as_str()
        .unwrap()
        .contains("projects"));
    assert!(requests.try_recv().is_err());
}

#[test]
fn unavailable_backend_exits_with_network_error() {
    let (address, _requests) = stub_daemon("HTTP/1.1 503 Service Unavailable", "Search offline");

    let output = run(&address, &["chat", "search", "nájem"]);

    failure(&output, 2, "503");
    assert!(String::from_utf8_lossy(&output.stderr).contains("Search offline"));
}

#[test]
fn malformed_search_response_exits_with_parse_error() {
    for body in [
        "not json",
        r#"{"data":{}}"#,
        r#"{"data":{"search":null}}"#,
        r#"{"data":{"search":{}}}"#,
    ] {
        let (address, _requests) = stub_daemon("HTTP/1.1 200 OK", body);

        let output = run(&address, &["chat", "search", "nájem"]);

        failure(&output, 4, "Parse error");
    }
}

#[test]
fn invalid_arguments_are_rejected_before_loading_credentials() {
    for args in [
        vec!["chat", "search"],
        vec!["chat", "search", "  "],
        vec!["chat", "search", "nájem", "--project", " "],
        vec!["chat", "search", "nájem", "--limit", "0"],
        vec!["chat", "search", "nájem", "--limit", "101"],
        vec!["chat", "search", "nájem", "--offset", "-1"],
        vec!["chat", "search", "nájem", "--offset", "2147483648"],
    ] {
        let (mut command, home) = command("127.0.0.1:0");
        fs::remove_file(home.path().join(".cdx/env/secret")).unwrap();

        let output = command.args(args).output().unwrap();

        failure(&output, 2, "error:");
        assert!(String::from_utf8_lossy(&output.stderr).contains("--help"));
        assert!(!String::from_utf8_lossy(&output.stderr).contains("no auth available"));
    }
}

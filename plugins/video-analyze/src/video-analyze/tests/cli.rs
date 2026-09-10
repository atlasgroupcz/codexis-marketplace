use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Output};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Duration;

const JOB: &str = "8f1c2a4e-1b2c-4d3e-9f10-1234567890ab";
const SESSION: &str = "0c3d5e7f-9a1b-4c2d-8e3f-a4b5c6d7e8f9";

struct Request {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

fn daemon(responses: Vec<(u16, serde_json::Value)>) -> (String, Receiver<Request>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/graphql", listener.local_addr().unwrap());
    let (sender, receiver) = channel();
    thread::spawn(move || {
        for (status, body) in responses {
            let (stream, _) = listener.accept().unwrap();
            let request = read(&stream);
            sender.send(request).unwrap();
            respond(stream, status, &body.to_string());
        }
    });
    (url, receiver)
}

fn read(stream: &TcpStream) -> Request {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap().to_string();
    let path = parts.next().unwrap().to_string();
    let mut headers = Vec::new();
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        let header = header.trim_end();
        if header.is_empty() {
            break;
        }
        let (name, value) = header.split_once(':').unwrap();
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }
    let length: usize = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map(|(_, value)| value.parse().unwrap())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).unwrap();
    Request {
        method,
        path,
        headers,
        body: String::from_utf8(body).unwrap(),
    }
}

fn respond(mut stream: TcpStream, status: u16, body: &str) {
    let reason = match status {
        200 => "OK",
        202 => "Accepted",
        404 => "Not Found",
        _ => "Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    stream.flush().unwrap();
}

fn job(status: &str) -> serde_json::Value {
    serde_json::json!({"id": JOB, "status": status, "text": null, "transcriptPath": null, "error": null})
}

fn home(root: &Path) -> std::path::PathBuf {
    let home = root.join("home");
    fs::create_dir_all(home.join(".cdx/env")).unwrap();
    fs::write(home.join(".cdx/env/secret"), "vm.token.x\n").unwrap();
    home
}

fn video_analyze(url: &str, home: &Path, cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_video-analyze"))
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap())
        .env("HOME", home)
        .env("CODEXIS_PUBLIC_DAEMON_URL", url)
        .env("CODEXIS_PUBLIC_SESSION_ID", SESSION)
        .output()
        .unwrap()
}

fn next(requests: &Receiver<Request>) -> Request {
    requests.recv_timeout(Duration::from_secs(30)).unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}

#[test]
fn transcript_submits_the_job_polls_it_and_prints_the_transcript_path() {
    let root = tempfile::tempdir().unwrap();
    let home = home(root.path());
    let mut finished = job("SUCCEEDED");
    finished["transcriptPath"] = "/home/codexis/.transcripts/meeting-abc.transcript.txt".into();
    let (url, requests) = daemon(vec![
        (202, job("QUEUED")),
        (200, job("RUNNING")),
        (200, finished),
    ]);

    let output = video_analyze(&url, &home, root.path(), &["transcript", "meeting.mp4"]);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        "/home/codexis/.transcripts/meeting-abc.transcript.txt"
    );
    assert_eq!(text(&output.stderr), format!("job {JOB}"));

    let submit = next(&requests);
    assert_eq!(submit.method, "POST");
    assert_eq!(submit.path, "/rest/v1/plugin/media/transcripts");
    assert_eq!(submit.header("Authorization"), Some("Bearer vm.token.x"));
    assert_eq!(submit.header("X-CDX-Session-Id"), Some(SESSION));
    assert_eq!(submit.header("Content-Type"), Some("application/json"));
    let body: serde_json::Value = serde_json::from_str(&submit.body).unwrap();
    assert_eq!(
        body.as_object().unwrap().keys().collect::<Vec<_>>(),
        ["source"]
    );
    let source = Path::new(body["source"].as_str().unwrap());
    assert_eq!(source.file_name().unwrap(), "meeting.mp4");
    assert_eq!(
        source.parent().unwrap().canonicalize().unwrap(),
        root.path().canonicalize().unwrap()
    );
    for _ in 0..2 {
        let poll = next(&requests);
        assert_eq!(poll.method, "GET");
        assert_eq!(poll.path, format!("/rest/v1/plugin/media/jobs/{JOB}"));
        assert_eq!(poll.header("Authorization"), Some("Bearer vm.token.x"));
        assert_eq!(poll.header("X-CDX-Session-Id"), Some(SESSION));
    }
}

#[test]
fn query_prints_the_answer_of_a_job_that_finished_at_once() {
    let root = tempfile::tempdir().unwrap();
    let home = home(root.path());
    let mut finished = job("SUCCEEDED");
    finished["text"] = "The logo is blue.".into();
    let (url, requests) = daemon(vec![(202, finished)]);

    let output = video_analyze(
        &url,
        &home,
        root.path(),
        &[
            "https://youtu.be/jNQXAC9IVRw",
            "What",
            "colour",
            "is",
            "the",
            "logo?",
        ],
    );

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout), "The logo is blue.");
    let submit = next(&requests);
    assert_eq!(submit.path, "/rest/v1/plugin/media/answers");
    let body: serde_json::Value = serde_json::from_str(&submit.body).unwrap();
    assert_eq!(body["source"], "https://youtu.be/jNQXAC9IVRw");
    assert_eq!(body["question"], "What colour is the logo?");
    assert_eq!(body.get("mode"), None);
}

#[test]
fn failed_job_reports_its_error_and_exits_non_zero() {
    let root = tempfile::tempdir().unwrap();
    let home = home(root.path());
    let mut failed = job("FAILED");
    failed["error"] =
        "Unsupported media format: text/plain. Use MP4, WebM, MOV, MP3 or WAV.".into();
    let (url, _requests) = daemon(vec![(202, job("QUEUED")), (200, failed)]);

    let output = video_analyze(&url, &home, root.path(), &["transcript", "notes.txt"]);

    assert!(!output.status.success());
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        text(&output.stderr),
        format!("job {JOB}\nerror: Unsupported media format: text/plain. Use MP4, WebM, MOV, MP3 or WAV.")
    );
}

#[test]
fn backend_error_body_is_passed_on() {
    let root = tempfile::tempdir().unwrap();
    let home = home(root.path());
    let (url, _requests) = daemon(vec![(
        404,
        serde_json::json!({"message": "Media file does not exist: /home/codexis/missing.mp4"}),
    )]);

    let output = video_analyze(&url, &home, root.path(), &["transcript", "missing.mp4"]);

    assert!(!output.status.success());
    assert!(
        text(&output.stderr).contains("Media file does not exist"),
        "{}",
        text(&output.stderr)
    );
}

#[test]
fn wait_finishes_a_job_started_earlier_and_prints_its_result() {
    let root = tempfile::tempdir().unwrap();
    let home = home(root.path());
    let mut finished = job("SUCCEEDED");
    finished["text"] = "[00:01] Hello".into();
    finished["transcriptPath"] = "/home/codexis/.transcripts/hello-abc.transcript.txt".into();
    let (url, requests) = daemon(vec![(200, job("RUNNING")), (200, finished)]);

    let output = video_analyze(&url, &home, root.path(), &["wait", JOB]);

    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        "/home/codexis/.transcripts/hello-abc.transcript.txt"
    );
    assert_eq!(text(&output.stderr), format!("job {JOB}"));
    for _ in 0..2 {
        let poll = next(&requests);
        assert_eq!(poll.method, "GET");
        assert_eq!(poll.path, format!("/rest/v1/plugin/media/jobs/{JOB}"));
        assert_eq!(poll.header("Content-Type"), None);
    }
}

#[test]
fn unknown_job_status_is_reported_rather_than_waited_for() {
    let root = tempfile::tempdir().unwrap();
    let home = home(root.path());
    let (url, _requests) = daemon(vec![(202, job("CANCELLED"))]);

    let output = video_analyze(&url, &home, root.path(), &["transcript", "meeting.mp4"]);

    assert!(!output.status.success());
    assert!(
        text(&output.stderr).contains("unknown job status"),
        "{}",
        text(&output.stderr)
    );
    assert!(text(&output.stderr).contains("CANCELLED"));
}

#[test]
fn help_needs_no_daemon() {
    let output = Command::new(env!("CARGO_BIN_EXE_video-analyze"))
        .arg("--help")
        .env_clear()
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(text(&output.stdout).contains("video-analyze transcript <source>"));
}

#[test]
fn missing_environment_is_named_before_any_request() {
    let root = tempfile::tempdir().unwrap();
    let home = home(root.path());
    let bin = env!("CARGO_BIN_EXE_video-analyze");

    let output = Command::new(bin)
        .args(["transcript", "meeting.mp4"])
        .env_clear()
        .env("HOME", &home)
        .env("CODEXIS_PUBLIC_SESSION_ID", SESSION)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        text(&output.stderr),
        "error: CODEXIS_PUBLIC_DAEMON_URL is required"
    );

    let output = Command::new(bin)
        .args(["transcript", "meeting.mp4"])
        .env_clear()
        .env("HOME", &home)
        .env("CODEXIS_PUBLIC_DAEMON_URL", "http://127.0.0.1:9/graphql")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        text(&output.stderr),
        "error: CODEXIS_PUBLIC_SESSION_ID is required"
    );

    let output = Command::new(bin)
        .args(["transcript", "meeting.mp4"])
        .env_clear()
        .env("HOME", root.path().join("empty"))
        .env("CODEXIS_PUBLIC_DAEMON_URL", "http://127.0.0.1:9/graphql")
        .env("CODEXIS_PUBLIC_SESSION_ID", SESSION)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        text(&output.stderr),
        "error: no daemon auth (missing ~/.cdx/env/secret)"
    );
}

use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use cdx_common::config::{read_env, CODEXIS_PUBLIC_DAEMON_URL_ENV, CODEXIS_PUBLIC_SESSION_ID_ENV};
use cdx_common::daemon_rest::daemon_bearer;
use serde_json::{json, Value};

const USAGE: &str = "Usage:
  video-analyze transcript <source>    Transcribe media; prints the transcript file path
  video-analyze <source> <query>       Ask a question about media; prints the answer
  video-analyze wait <job-id>          Finish waiting for a job whose call was cut off

  <source>  Local file path or YouTube URL";

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const WAIT_LIMIT: Duration = Duration::from_secs(8 * 60);
const REQUEST_TIMEOUT_SECONDS: &str = "60";

#[derive(Debug, PartialEq)]
enum Action {
    Help,
    Wait(String),
    Submit {
        source: String,
        query: Option<String>,
    },
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let (source, query) = match parse_args(args)? {
        Action::Help => return Ok(USAGE.to_string()),
        Action::Wait(id) => {
            let daemon = Daemon::from_env()?;
            return wait(&daemon, daemon.job(&id)?).and_then(|job| output(&job));
        }
        Action::Submit { source, query } => (source, query),
    };
    let daemon = Daemon::from_env()?;
    let directory = env::current_dir().map_err(|error| error.to_string())?;
    let (path, body) = request(&absolute(&source, &directory)?, query.as_deref());
    let job = daemon.submit(path, &body)?;
    wait(&daemon, job).and_then(|job| output(&job))
}

fn parse_args(args: &[String]) -> Result<Action, String> {
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(Action::Help);
    }
    match args {
        [command, id] if command == "wait" => {
            require_job_id(id)?;
            Ok(Action::Wait(id.clone()))
        }
        [command, ..] if command == "wait" => Err(format!("expected one job id\n{USAGE}")),
        [command, source] if command == "transcript" => Ok(Action::Submit {
            source: source.clone(),
            query: None,
        }),
        [command, ..] if command == "transcript" => Err(format!("expected one source\n{USAGE}")),
        [source, query @ ..] if !query.is_empty() => Ok(Action::Submit {
            source: source.clone(),
            query: Some(query.join(" ")),
        }),
        _ => Err(format!("source and query are required\n{USAGE}")),
    }
}

fn require_job_id(id: &str) -> Result<(), String> {
    let shaped = id.len() == 36
        && id.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        });
    if shaped {
        Ok(())
    } else {
        Err(format!("{id} is not a media job id"))
    }
}

fn absolute(source: &str, directory: &Path) -> Result<String, String> {
    if source.contains("://") {
        return Ok(source.to_string());
    }
    directory
        .join(source)
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| format!("{source} is not a valid UTF-8 path"))
}

fn request(source: &str, question: Option<&str>) -> (&'static str, Value) {
    match question {
        None => ("transcripts", json!({"source": source})),
        Some(question) => ("answers", json!({"source": source, "question": question})),
    }
}

fn wait(daemon: &Daemon, first: Value) -> Result<Value, String> {
    let id = first["id"]
        .as_str()
        .ok_or("the media API returned no job id")?
        .to_string();
    eprintln!("job {id}");
    let started = Instant::now();
    let mut job = first;
    loop {
        match job["status"].as_str() {
            Some("SUCCEEDED") => return Ok(job),
            Some("FAILED") => {
                return Err(job["error"]
                    .as_str()
                    .unwrap_or("media analysis failed")
                    .to_string())
            }
            Some("QUEUED" | "RUNNING") if started.elapsed() < WAIT_LIMIT => {}
            Some("QUEUED" | "RUNNING") => {
                return Err(format!(
                    "still running after {} minutes; keep waiting with: video-analyze wait {id}",
                    WAIT_LIMIT.as_secs() / 60
                ))
            }
            _ => {
                return Err(format!(
                    "the media API returned an unknown job status: {job}"
                ))
            }
        }
        thread::sleep(POLL_INTERVAL);
        job = daemon.job(&id)?;
    }
}

fn output(job: &Value) -> Result<String, String> {
    ["transcriptPath", "text"]
        .iter()
        .find_map(|field| job[*field].as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "the finished job carries neither a transcript path nor a text".to_string())
}

struct Daemon {
    base: String,
    bearer: String,
    session: String,
}

impl Daemon {
    fn from_env() -> Result<Self, String> {
        let base = base_url(read_env(CODEXIS_PUBLIC_DAEMON_URL_ENV).as_deref())?;
        let session = read_env(CODEXIS_PUBLIC_SESSION_ID_ENV)
            .ok_or(format!("{CODEXIS_PUBLIC_SESSION_ID_ENV} is required"))?;
        let bearer = daemon_bearer().ok_or("no daemon auth (missing ~/.cdx/env/secret)")?;
        Ok(Self {
            base,
            bearer,
            session,
        })
    }

    fn submit(&self, path: &str, body: &Value) -> Result<Value, String> {
        let url = format!("{}/rest/v1/plugin/media/{path}", self.base);
        self.call(&url, Some(body))
    }

    fn job(&self, id: &str) -> Result<Value, String> {
        require_job_id(id)?;
        let url = format!("{}/rest/v1/plugin/media/jobs/{id}", self.base);
        self.call(&url, None)
    }

    fn call(&self, url: &str, body: Option<&Value>) -> Result<Value, String> {
        let mut curl = Command::new("curl");
        curl.args([
            "--silent",
            "--show-error",
            "--fail-with-body",
            "--max-time",
            REQUEST_TIMEOUT_SECONDS,
            "--request",
            if body.is_some() { "POST" } else { "GET" },
            url,
            "--header",
            &format!("Authorization: {}", self.bearer),
            "--header",
            &format!("X-CDX-Session-Id: {}", self.session),
        ]);
        if body.is_some() {
            curl.args([
                "--header",
                "Content-Type: application/json",
                "--data-binary",
                "@-",
            ]);
        }
        let mut child = curl
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("failed to run curl: {error}"))?;
        if let Some(body) = body {
            let mut stdin = child.stdin.take().ok_or("curl stdin is unavailable")?;
            stdin
                .write_all(body.to_string().as_bytes())
                .map_err(|error| format!("failed to send the request: {error}"))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|error| format!("failed to run curl: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "media API request failed: {} {}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        serde_json::from_slice(&output.stdout)
            .map_err(|error| format!("the media API returned no JSON: {error}"))
    }
}

fn base_url(graphql: Option<&str>) -> Result<String, String> {
    let graphql = graphql.ok_or(format!("{CODEXIS_PUBLIC_DAEMON_URL_ENV} is required"))?;
    let base = graphql.trim_end_matches('/');
    let base = base.strip_suffix("/graphql").unwrap_or(base);
    if !base.starts_with("http://") && !base.starts_with("https://") {
        return Err(format!(
            "{CODEXIS_PUBLIC_DAEMON_URL_ENV} must be an HTTP or HTTPS URL"
        ));
    }
    Ok(base.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn submit(source: &str, query: Option<&str>) -> Action {
        Action::Submit {
            source: source.into(),
            query: query.map(str::to_string),
        }
    }

    #[test]
    fn transcript_takes_one_source() {
        assert_eq!(
            parse_args(&args(&["transcript", "meeting.mp4"])),
            Ok(submit("meeting.mp4", None))
        );
    }

    #[test]
    fn transcript_refuses_more_than_one_source() {
        assert!(parse_args(&args(&["transcript", "a.mp4", "b.mp4"]))
            .unwrap_err()
            .starts_with("expected one source"));
    }

    #[test]
    fn query_joins_the_words_after_the_source() {
        assert_eq!(
            parse_args(&args(&["clip.mp4", "what", "is", "shown?"])),
            Ok(submit("clip.mp4", Some("what is shown?")))
        );
    }

    #[test]
    fn a_source_without_a_query_is_refused() {
        assert!(parse_args(&args(&["clip.mp4"]))
            .unwrap_err()
            .starts_with("source and query are required"));
    }

    #[test]
    fn wait_takes_a_job_id() {
        let id = "8f1c2a4e-1b2c-4d3e-9f10-1234567890ab";
        assert_eq!(
            parse_args(&args(&["wait", id])),
            Ok(Action::Wait(id.into()))
        );
    }

    #[test]
    fn wait_refuses_anything_but_a_uuid() {
        assert_eq!(
            parse_args(&args(&["wait", "../jobs"])),
            Err("../jobs is not a media job id".into())
        );
    }

    #[test]
    fn help_wins_over_everything_else() {
        assert_eq!(parse_args(&args(&["clip.mp4", "--help"])), Ok(Action::Help));
    }

    #[test]
    fn base_url_strips_the_graphql_suffix() {
        assert_eq!(
            base_url(Some("http://daemon:8086/graphql")),
            Ok("http://daemon:8086".into())
        );
        assert_eq!(
            base_url(Some("https://daemon.example/")),
            Ok("https://daemon.example".into())
        );
    }

    #[test]
    fn base_url_must_be_http() {
        assert!(base_url(Some("daemon:8086")).unwrap_err().contains("HTTP"));
    }

    #[test]
    fn a_relative_file_is_made_absolute_against_the_working_directory() {
        assert_eq!(
            absolute("a.mp4", Path::new("/home/codexis/meetings")),
            Ok("/home/codexis/meetings/a.mp4".into())
        );
    }

    #[test]
    fn an_absolute_file_and_a_url_are_sent_as_they_are() {
        assert_eq!(
            absolute("/tmp/a.mp4", Path::new("/home/codexis")),
            Ok("/tmp/a.mp4".into())
        );
        assert_eq!(
            absolute("https://youtu.be/jNQXAC9IVRw", Path::new("/home/codexis")),
            Ok("https://youtu.be/jNQXAC9IVRw".into())
        );
    }

    #[test]
    fn a_transcript_is_asked_for_at_the_transcripts_endpoint() {
        assert_eq!(
            request("/home/codexis/a.mp4", None),
            ("transcripts", json!({"source": "/home/codexis/a.mp4"}))
        );
    }

    #[test]
    fn a_question_is_asked_at_the_answers_endpoint() {
        assert_eq!(
            request("/home/codexis/a.mp4", Some("who speaks?")),
            (
                "answers",
                json!({"source": "/home/codexis/a.mp4", "question": "who speaks?"})
            )
        );
    }

    #[test]
    fn output_prefers_the_transcript_path_over_the_text() {
        let job =
            json!({"text": "[00:01] Hello", "transcriptPath": "/home/codexis/.transcripts/a.txt"});
        assert_eq!(output(&job), Ok("/home/codexis/.transcripts/a.txt".into()));
    }

    #[test]
    fn output_falls_back_to_the_text_of_an_answer() {
        assert_eq!(
            output(&json!({"text": " answer ", "transcriptPath": null})),
            Ok("answer".into())
        );
    }

    #[test]
    fn output_of_an_empty_job_is_an_error() {
        assert!(output(&json!({"status": "SUCCEEDED"})).is_err());
    }
}

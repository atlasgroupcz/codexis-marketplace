use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;
use std::process::Command;

use cdx_common::daemon_rest::daemon_bearer;

const MODEL_QUERY: &str = "gemini-3.1-flash-lite-preview";
const MODEL_TRANSCRIPT: &str = "gemini-3.1-pro-preview";
const VERTEX_PROJECT: &str = "gen-lang-client-0126863821";
const VERTEX_LOCATION: &str = "global";
// Upload endpoint is derived at runtime from CODEXIS_PUBLIC_DAEMON_URL - see upload_endpoint().

const TRANSCRIPT_PROMPT: &str = "Transcribe this media file with full detail.\n\
    Use this exact format:\n\n\
    [MM:SS] Speaker text here\n\
    [MM:SS] (scene) Description of what is shown on screen\n\n\
    Rules:\n\
    - One line per speech segment or scene change\n\
    - Timestamp in [MM:SS] format at the start of each line\n\
    - Verbatim transcription of all spoken dialogue\n\
    - Identify speakers if distinguishable (e.g. [00:05] Moderator: ...)\n\
    - Mark visual descriptions with (scene) prefix: on-screen text, slides, graphics, camera changes\n\
    - Note significant audio cues with (audio) prefix: music, sound effects, silence\n\
    - No extra commentary, headers, or formatting - just timestamped lines\n\
    - Do not use markdown, code fences, or any wrapper";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        std::process::exit(if args.iter().any(|a| a == "-h" || a == "--help") {
            0
        } else {
            1
        });
    }

    let is_transcript = args.first().map(|a| a == "transcript").unwrap_or(false);

    if is_transcript {
        if args.len() < 2 {
            eprintln!("error: missing source");
            eprintln!("Usage: video-analyze transcript <source>");
            std::process::exit(1);
        }
        let source = &args[1];
        let api_key = load_api_key();
        let daemon_auth = daemon_bearer().unwrap_or_else(|| {
            eprintln!("error: no daemon auth (missing ~/.cdx/env/secret)");
            std::process::exit(2);
        });
        let is_youtube = source.contains("youtube.com/") || source.contains("youtu.be/");

        let (file_uri, mime_type) = upload_source(source, &daemon_auth, is_youtube);
        let response = query_gemini(
            &api_key,
            &file_uri,
            &mime_type,
            TRANSCRIPT_PROMPT,
            is_youtube,
            MODEL_TRANSCRIPT,
        );
        let text = extract_response_text(&response);
        if text.is_empty() {
            eprintln!("{}", response);
            std::process::exit(1);
        }

        let transcript_dir = Path::new(".transcripts");
        if !transcript_dir.exists() {
            if let Err(e) = fs::create_dir_all(transcript_dir) {
                eprintln!("error: failed to create .transcripts directory: {}", e);
                std::process::exit(1);
            }
        }

        let base_name = transcript_filename(source, is_youtube);
        let transcript_path = transcript_dir.join(format!("{}.transcript.txt", base_name));
        if let Err(e) = fs::write(&transcript_path, &text) {
            eprintln!("error: failed to write transcript: {}", e);
            std::process::exit(1);
        }

        println!("{}", transcript_path.display());
    } else {
        if args.len() < 2 {
            eprintln!("error: missing query");
            print_usage();
            std::process::exit(1);
        }

        let source = &args[0];
        let query = args[1..].join(" ");
        let api_key = load_api_key();
        let daemon_auth = daemon_bearer().unwrap_or_else(|| {
            eprintln!("error: no daemon auth (missing ~/.cdx/env/secret)");
            std::process::exit(2);
        });
        let is_youtube = source.contains("youtube.com/") || source.contains("youtu.be/");

        let (file_uri, mime_type) = upload_source(source, &daemon_auth, is_youtube);
        let response = query_gemini(
            &api_key,
            &file_uri,
            &mime_type,
            &query,
            is_youtube,
            MODEL_QUERY,
        );
        let text = extract_response_text(&response);
        if text.is_empty() {
            eprintln!("{}", response);
            std::process::exit(1);
        }
        println!("{}", text);
    }
}

fn upload_source(source: &str, daemon_auth: &str, is_youtube: bool) -> (String, String) {
    if is_youtube {
        (source.to_string(), String::new())
    } else {
        let upload_json = upload_file(source, daemon_auth);
        let parsed: serde_json::Value = serde_json::from_str(&upload_json).unwrap_or_default();
        let uri = parsed
            .get("uri")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let mime = parsed
            .get("mimeType")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        if uri.is_empty() {
            eprintln!("error: upload failed: {}", upload_json);
            std::process::exit(1);
        }
        (uri, mime)
    }
}

fn transcript_filename(source: &str, is_youtube: bool) -> String {
    if is_youtube {
        let video_id = source
            .split("v=")
            .nth(1)
            .map(|s| s.split('&').next().unwrap_or(s))
            .or_else(|| {
                source
                    .split("youtu.be/")
                    .nth(1)
                    .map(|s| s.split('?').next().unwrap_or(s))
            })
            .unwrap_or("unknown");
        format!("youtube-{}", video_id)
    } else {
        Path::new(source)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  video-analyze transcript <source>    Generate detailed transcript");
    eprintln!("  video-analyze <source> <query>       Ask a question about media");
    eprintln!();
    eprintln!("  <source>  Local file path or YouTube URL");
    eprintln!("  <query>   What to analyze (e.g. \"Summarize the key points\")");
}

fn load_api_key() -> String {
    if let Ok(val) = env::var("CODEXIS_USER_LITELLM_API_KEY") {
        if !val.is_empty() {
            return val;
        }
    }
    eprintln!("error: CODEXIS_USER_LITELLM_API_KEY not set");
    std::process::exit(2);
}

fn litellm_base() -> String {
    // The LiteLLM gateway base URL is injected into the VM as
    // CODEXIS_PUBLIC_LITELLM_BASE_URL (e.g. http://litellm.cdx.internal:4000).
    // Fall back to localhost:4000 only for local dev outside the VM.
    env::var("CODEXIS_PUBLIC_LITELLM_BASE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .map(|v| v.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "http://localhost:4000".to_string())
}

fn upload_endpoint() -> String {
    // CODEXIS_PUBLIC_DAEMON_URL is the daemon GraphQL URL (…/graphql); the gemini
    // upload REST endpoint is its sibling on the same host. Strip /graphql and
    // append the REST path. Fall back to localhost:8086 only for local dev.
    let graphql = env::var("CODEXIS_PUBLIC_DAEMON_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "http://localhost:8086/graphql".to_string());
    let base = graphql.trim_end_matches('/');
    let base = base.strip_suffix("/graphql").unwrap_or(base);
    format!("{}/rest/llm/gemini/upload", base)
}

fn upload_file(path: &str, daemon_auth: &str) -> String {
    let encoded_path = cdx_common::url::encode_path_value(path);
    let url = format!("{}?path={}", upload_endpoint(), encoded_path);

    let auth_header = format!("Authorization: {}", daemon_auth);
    let output = Command::new("curl")
        .args([
            "-s",
            "--fail-with-body",
            "-X",
            "POST",
            &url,
            "-H",
            &auth_header,
            "-H",
            "Content-Type: application/json",
        ])
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                eprintln!("error: upload failed: {} {}", stdout, stderr);
                std::process::exit(1);
            }
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        Err(e) => {
            eprintln!("error: failed to run curl: {}", e);
            std::process::exit(1);
        }
    }
}

fn query_gemini(
    api_key: &str,
    file_uri: &str,
    mime_type: &str,
    query: &str,
    is_youtube: bool,
    model: &str,
) -> String {
    let url = format!(
        "{}/vertex_ai/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
        litellm_base(),
        VERTEX_PROJECT,
        VERTEX_LOCATION,
        model
    );

    let file_data = if is_youtube {
        format!(
            r#"{{"fileData":{{"fileUri":"{}","mimeType":"video/webm"}}}}"#,
            escape_json(file_uri)
        )
    } else {
        format!(
            r#"{{"fileData":{{"fileUri":"{}","mimeType":"{}"}}}}"#,
            escape_json(file_uri),
            escape_json(mime_type)
        )
    };

    let body = format!(
        r#"{{"contents":[{{"role":"user","parts":[{{"text":"{}"}},{}]}}]}}"#,
        escape_json(query),
        file_data
    );

    let auth_header = format!("Authorization: Bearer {}", api_key);

    let mut args = vec![
        "-s".to_string(),
        "--fail-with-body".to_string(),
        "-X".to_string(),
        "POST".to_string(),
        url.clone(),
        "-H".to_string(),
        auth_header,
        "-H".to_string(),
        "Content-Type: application/json".to_string(),
    ];
    if let Ok(chat_id) = env::var("CODEXIS_PUBLIC_SESSION_ID") {
        if !chat_id.is_empty() {
            args.extend([
                "-H".to_string(),
                format!(
                    "x-litellm-spend-logs-metadata: {{\"sessionId\":\"{}\"}}",
                    chat_id
                ),
            ]);
        }
    }
    args.extend(["-d".to_string(), body]);

    let output = Command::new("curl").args(&args).output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                eprintln!("error: gemini query failed: {} {}", stdout, stderr);
                std::process::exit(1);
            }
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        Err(e) => {
            eprintln!("error: failed to run curl: {}", e);
            std::process::exit(1);
        }
    }
}

/// Extract candidates[0].content.parts[0].text from a Gemini response.
fn extract_response_text(json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| {
            v.get("candidates")?
                .get(0)?
                .get("content")?
                .get("parts")?
                .get(0)?
                .get("text")?
                .as_str()
                .map(str::to_string)
        })
        .unwrap_or_default()
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            _ => out.push(c),
        }
    }
    out
}

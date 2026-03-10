use std::env;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::process::Command;

const MODEL: &str = "gemini-3.1-flash-lite-preview";
const VERTEX_PROJECT: &str = "gen-lang-client-0126863821";
const VERTEX_LOCATION: &str = "global";
const UPLOAD_ENDPOINT: &str = "http://localhost:38083/rest/llm/gemini/upload";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        std::process::exit(if args.iter().any(|a| a == "-h" || a == "--help") { 0 } else { 1 });
    }

    if args.len() < 2 {
        eprintln!("error: missing query");
        print_usage();
        std::process::exit(1);
    }

    let source = &args[0];
    let query = args[1..].join(" ");
    let api_key = load_api_key();
    let is_youtube = source.contains("youtube.com/") || source.contains("youtu.be/");

    let (file_uri, mime_type) = if is_youtube {
        (source.clone(), String::new())
    } else {
        let upload_json = upload_file(source);
        let uri = extract_json_field(&upload_json, "uri");
        let mime = extract_json_field(&upload_json, "mimeType");
        if uri.is_empty() {
            eprintln!("error: upload failed: {}", upload_json);
            std::process::exit(1);
        }
        (uri, mime)
    };

    let response = query_gemini(&api_key, &file_uri, &mime_type, &query, is_youtube);
    let text = extract_response_text(&response);
    if text.is_empty() {
        eprintln!("{}", response);
        std::process::exit(1);
    }
    println!("{}", text);
}

fn print_usage() {
    eprintln!("Usage: video-analyze <source> <query>");
    eprintln!();
    eprintln!("  <source>  Local file path or YouTube URL");
    eprintln!("  <query>   What to analyze (e.g. \"Transcribe this video\")");
}

fn load_api_key() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/codexis".to_string());
    let env_file = format!("{}/.cdx/.env", home);
    if let Ok(content) = fs::read_to_string(&env_file) {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("LITELLM_API_KEY=") {
                let val = val.trim();
                if !val.is_empty() {
                    return val.to_string();
                }
            }
        }
    }
    if let Ok(val) = env::var("LITELLM_API_KEY") {
        if !val.is_empty() {
            return val;
        }
    }
    eprintln!("error: LITELLM_API_KEY not found in ~/.cdx/.env or environment");
    std::process::exit(2);
}

fn upload_file(path: &str) -> String {
    let encoded_path = percent_encode(path.as_bytes());
    let url = format!("{}?path={}", UPLOAD_ENDPOINT, encoded_path);

    let output = Command::new("curl")
        .args(["-s", "--fail-with-body", "-X", "POST", &url, "-H", "Content-Type: application/json"])
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

fn query_gemini(api_key: &str, file_uri: &str, mime_type: &str, query: &str, is_youtube: bool) -> String {
    let url = format!(
        "http://localhost:4000/vertex_ai/v1/projects/{}/locations/{}/publishers/google/models/{}:generateContent",
        VERTEX_PROJECT, VERTEX_LOCATION, MODEL
    );

    let file_data = if is_youtube {
        format!(r#"{{"fileData":{{"fileUri":"{}","mimeType":"video/webm"}}}}"#, escape_json(file_uri))
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

    let output = Command::new("curl")
        .args([
            "-s", "--fail-with-body",
            "-X", "POST",
            &url,
            "-H", &auth_header,
            "-H", "Content-Type: application/json",
            "-d", &body,
        ])
        .output();

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

/// Extract candidates[0].content.parts[0].text from Gemini response.
fn extract_response_text(json: &str) -> String {
    if let Some(parts_pos) = json.find("\"parts\"") {
        let after_parts = &json[parts_pos..];
        if let Some(text_pos) = after_parts.find("\"text\"") {
            let after_text = &after_parts[text_pos + 6..];
            let trimmed = after_text.trim_start();
            if let Some(rest) = trimmed.strip_prefix(':') {
                let rest = rest.trim_start();
                if rest.starts_with('"') {
                    return parse_json_string(&rest[1..]);
                }
            }
        }
    }
    String::new()
}

fn parse_json_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return result,
            '\\' => {
                if let Some(esc) = chars.next() {
                    match esc {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        '/' => result.push('/'),
                        'n' => result.push('\n'),
                        'r' => result.push('\r'),
                        't' => result.push('\t'),
                        'u' => {
                            let hex: String = chars.by_ref().take(4).collect();
                            if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                if let Some(ch) = char::from_u32(code) {
                                    result.push(ch);
                                }
                            }
                        }
                        _ => {
                            result.push('\\');
                            result.push(esc);
                        }
                    }
                }
            }
            _ => result.push(c),
        }
    }
    result
}

fn extract_json_field(json: &str, field: &str) -> String {
    let pattern = format!("\"{}\"", field);
    if let Some(pos) = json.find(&pattern) {
        let after = &json[pos + pattern.len()..];
        let trimmed = after.trim_start();
        if let Some(rest) = trimmed.strip_prefix(':') {
            let rest = rest.trim_start();
            if rest.starts_with('"') {
                return parse_json_string(&rest[1..]);
            }
        }
    }
    String::new()
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

fn percent_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for &b in input {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{:02X}", b);
            }
        }
    }
    out
}

use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

// OCR endpoint is derived at runtime from CODEXIS_PUBLIC_DAEMON_URL — see ocr_endpoint().
const SECRET_FILE_RELATIVE: &str = ".cdx/env/secret";
const CODEXIS_USER_API_TOKEN_ENV: &str = "CODEXIS_USER_API_TOKEN";

fn percent_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for &b in input {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{:02X}", b);
            }
        }
    }
    out
}

fn load_api_jwt_auth() -> String {
    // Match cdxctl/all plugins: read the daemon JWT from the per-user secret file
    // (~/.cdx/env/secret) first, fall back to the env var, and always send it as a
    // Bearer token.
    if let Some(secret) = read_secret_file() {
        return ensure_bearer_prefix(&secret);
    }
    if let Ok(val) = env::var(CODEXIS_USER_API_TOKEN_ENV) {
        if !val.is_empty() {
            return ensure_bearer_prefix(&val);
        }
    }
    eprintln!(
        "error: no auth (tried ~/{} and ${})",
        SECRET_FILE_RELATIVE, CODEXIS_USER_API_TOKEN_ENV
    );
    std::process::exit(2);
}

fn read_secret_file() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let contents = fs::read_to_string(Path::new(&home).join(SECRET_FILE_RELATIVE)).ok()?;
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn ensure_bearer_prefix(value: &str) -> String {
    let mut s = value.trim();
    if let Some(stripped) = s.strip_prefix("Authorization:") {
        s = stripped.trim();
    }
    if s.len() >= 7 && s[..7].eq_ignore_ascii_case("Bearer ") {
        s.to_string()
    } else {
        format!("Bearer {}", s)
    }
}

fn ocr_endpoint() -> String {
    // CODEXIS_PUBLIC_DAEMON_URL is the daemon GraphQL URL (…/graphql); the OCR
    // REST endpoint is its sibling. Strip /graphql, append /rest/ocr. Fall back
    // to localhost:8086 only for local dev.
    let graphql = env::var("CODEXIS_PUBLIC_DAEMON_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "http://localhost:8086/graphql".to_string());
    let base = graphql.trim_end_matches('/');
    let base = base.strip_suffix("/graphql").unwrap_or(base);
    format!("{}/rest/ocr", base)
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

fn parse_json_string(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            '\\' => {
                if let Some(e) = chars.next() {
                    match e {
                        'n' => out.push('\n'),
                        't' => out.push('\t'),
                        'r' => out.push('\r'),
                        '/' => out.push('/'),
                        other => out.push(other),
                    }
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("usage: ocr <path>");
        std::process::exit(if args.iter().any(|a| a == "-h" || a == "--help") { 0 } else { 1 });
    }

    let daemon_auth = load_api_jwt_auth();

    let path = &args[0];
    let encoded_path = percent_encode(path.as_bytes());
    let url = format!("{}?path={}", ocr_endpoint(), encoded_path);
    let auth_header = format!("Authorization: {}", daemon_auth);

    // The endpoint OCRs the file and returns {path, resultPath}; the recognized
    // text is written to resultPath. Capture the JSON, then print the result text.
    let output = Command::new("curl")
        .args(["-s", "--fail-with-body", "-X", "POST", &url, "-H", &auth_header])
        .output();
    let resp = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        Ok(o) => {
            eprintln!(
                "error: ocr request failed: {}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("failed to run curl: {}", err);
            std::process::exit(1);
        }
    };

    let result_path = extract_json_field(&resp, "resultPath");
    if result_path.is_empty() {
        eprintln!("error: ocr response has no resultPath: {}", resp);
        std::process::exit(1);
    }
    match fs::read_to_string(&result_path) {
        Ok(text) => print!("{}", text),
        Err(err) => {
            eprintln!("error: cannot read OCR result {}: {}", result_path, err);
            std::process::exit(1);
        }
    }
}

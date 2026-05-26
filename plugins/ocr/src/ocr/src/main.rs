use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const OCR_ENDPOINT: &str = "http://localhost:8086/rest/ocr";
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

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("usage: ocr <path>");
        std::process::exit(if args.iter().any(|a| a == "-h" || a == "--help") { 0 } else { 1 });
    }

    let daemon_auth = load_api_jwt_auth();

    let path = &args[0];
    let encoded_path = percent_encode(path.as_bytes());
    let url = format!("{}?path={}", OCR_ENDPOINT, encoded_path);

    let auth_header = format!("Authorization: {}", daemon_auth);
    let status = Command::new("curl")
        .args([
            "-s",
            "--fail-with-body",
            "-X", "POST",
            &url,
            "-H", &auth_header,
        ])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(err) => {
            eprintln!("failed to run curl: {}", err);
            std::process::exit(1);
        }
    }
}

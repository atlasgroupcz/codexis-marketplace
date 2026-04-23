use std::env;
use std::fmt::Write;
use std::fs;
use std::process::{Command, Stdio};

const OCR_ENDPOINT: &str = "http://localhost:8086/rest/ocr";

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

fn load_daemon_auth() -> String {
    let home = env::var("HOME").unwrap_or_else(|_| "/home/codexis".to_string());
    let env_file = format!("{}/.cdx/.daemon.env", home);
    if let Ok(content) = fs::read_to_string(&env_file) {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("CDX_DAEMON_AUTH=") {
                let val = val.trim();
                if !val.is_empty() {
                    return val.to_string();
                }
            }
        }
    }
    if let Ok(val) = env::var("CDX_DAEMON_AUTH") {
        if !val.is_empty() {
            return val;
        }
    }
    eprintln!("error: CDX_DAEMON_AUTH not found in ~/.cdx/.daemon.env or environment");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("usage: ocr <path>");
        std::process::exit(if args.iter().any(|a| a == "-h" || a == "--help") { 0 } else { 1 });
    }

    let daemon_auth = load_daemon_auth();

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

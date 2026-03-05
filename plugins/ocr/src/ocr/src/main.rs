use std::env;
use std::fmt::Write;
use std::process::{Command, Stdio};

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

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("usage: ocr <path>");
        std::process::exit(if args.iter().any(|a| a == "-h" || a == "--help") { 0 } else { 1 });
    }

    let daemon_url = match env::var("CDX_DAEMON_URL") {
        Ok(value) if !value.trim().is_empty() => value.trim().trim_end_matches('/').to_string(),
        _ => {
            eprintln!("CDX_DAEMON_URL must be set");
            std::process::exit(2);
        }
    };

    let daemon_auth = match env::var("CDX_DAEMON_AUTH") {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => {
            eprintln!("CDX_DAEMON_AUTH must be set");
            std::process::exit(2);
        }
    };

    let path = &args[0];
    let encoded_path = percent_encode(path.as_bytes());
    let url = format!("{}/rest/ocr?path={}", daemon_url, encoded_path);

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

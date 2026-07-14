use std::env;
use std::fmt::Write;
use std::fs;
use std::process::Command;

use cdx_common::daemon_rest::daemon_bearer;

// OCR endpoint is derived at runtime from CODEXIS_PUBLIC_DAEMON_URL — see ocr_endpoint().

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

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("usage: ocr <path>");
        std::process::exit(if args.iter().any(|a| a == "-h" || a == "--help") { 0 } else { 1 });
    }

    let daemon_auth = daemon_bearer().unwrap_or_else(|| {
        eprintln!("error: no daemon auth (missing ~/.cdx/env/secret)");
        std::process::exit(2);
    });

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

    let result_path = serde_json::from_str::<serde_json::Value>(&resp)
        .ok()
        .and_then(|v| v.get("resultPath").and_then(|s| s.as_str()).map(str::to_string))
        .unwrap_or_default();
    if result_path.is_empty() {
        eprintln!("error: ocr response has no resultPath: {}", resp);
        std::process::exit(1);
    }
    // resultPath is either a ZIP of per-page markdown (<doc>_page_N.md) or a plain
    // text/markdown file. Stream the recognized text to stdout either way. Read as
    // bytes (the result may be non-UTF-8) and write through unchanged.
    let bytes = if result_path.ends_with(".zip") {
        match Command::new("unzip").args(["-p", &result_path]).output() {
            Ok(o) if o.status.success() => o.stdout,
            Ok(o) => {
                eprintln!(
                    "error: could not unzip OCR result {}: {}",
                    result_path,
                    String::from_utf8_lossy(&o.stderr)
                );
                std::process::exit(1);
            }
            Err(err) => {
                eprintln!("error: failed to run unzip on {}: {}", result_path, err);
                std::process::exit(1);
            }
        }
    } else {
        match fs::read(&result_path) {
            Ok(b) => b,
            Err(err) => {
                eprintln!("error: cannot read OCR result {}: {}", result_path, err);
                std::process::exit(1);
            }
        }
    };
    use std::io::Write;
    let _ = std::io::stdout().write_all(&bytes);
}

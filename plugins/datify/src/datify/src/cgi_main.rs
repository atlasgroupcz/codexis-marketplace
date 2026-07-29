//! `datify-cgi` - the settings component's request handler, run in place by the
//! daemon.
//!
//! The page has one job: hold the Datify API token. GET reports the current
//! state (never the token itself, only a mask and where it came from); POST
//! saves, clears or re-checks it. The operations live in `datify_core::settings`
//! so this and the CLI cannot drift; the CGI plumbing lives in `cdx_cgi`.

use std::process::ExitCode;

use datify_core::{settings, DatifyError};
use serde_json::{json, Value};

fn main() -> ExitCode {
    let method = std::env::var("REQUEST_METHOD").unwrap_or_else(|_| "GET".into());
    match method.to_uppercase().as_str() {
        "POST" => handle_post(),
        _ => cdx_cgi::emit_json(&settings::payload()),
    }
}

fn handle_post() -> ExitCode {
    let body = match cdx_cgi::read_body() {
        Ok(body) => body,
        Err(e) => return cdx_cgi::emit_error(400, &format!("Tělo požadavku nelze přečíst: {e}")),
    };
    let parsed: Value = if body.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(&body) {
            Ok(parsed) => parsed,
            Err(e) => return cdx_cgi::emit_error(400, &format!("Neplatné JSON: {e}")),
        }
    };

    match parsed.get("action").and_then(|v| v.as_str()).unwrap_or("") {
        "save_api_key" => {
            let token = parsed.get("apiKey").and_then(|v| v.as_str()).unwrap_or("");
            if token.trim().is_empty() {
                return cdx_cgi::emit_error(400, "API token je prázdný.");
            }
            emit(settings::set_key(token))
        }
        "test_api_key" => emit(settings::test()),
        "delete_api_key" => emit(settings::clear_key()),
        other => cdx_cgi::emit_error(400, &format!("Neznámá akce: {other}")),
    }
}

fn emit(result: datify_core::Result<Value>) -> ExitCode {
    match result {
        Ok(value) => cdx_cgi::emit_json(&value),
        Err(e) => fail(e),
    }
}

fn fail(error: DatifyError) -> ExitCode {
    cdx_cgi::emit_error(error.http_status(), &error.to_string())
}

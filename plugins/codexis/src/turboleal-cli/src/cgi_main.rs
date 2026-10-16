use std::process::ExitCode;

use serde_json::{Value, json};
use turboleal::config;
use turboleal::settings::{self, SettingsResult};

fn main() -> ExitCode {
    let api_url = config::api_url();
    let method = std::env::var("REQUEST_METHOD").unwrap_or_else(|_| "GET".into());
    match method.to_uppercase().as_str() {
        "POST" => handle_post(&api_url),
        _ => cdx_cgi::emit_json(&settings::payload(&api_url, None)),
    }
}

fn handle_post(api_url: &str) -> ExitCode {
    let body = match cdx_cgi::read_body() {
        Ok(body) => body,
        Err(error) => {
            return cdx_cgi::emit_error(400, &format!("Tělo požadavku nelze přečíst: {error}"));
        }
    };
    let parsed: Value = if body.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(&body) {
            Ok(parsed) => parsed,
            Err(error) => return cdx_cgi::emit_error(400, &format!("Neplatné JSON: {error}")),
        }
    };

    match parsed.get("action").and_then(Value::as_str).unwrap_or("") {
        "save_api_key" => {
            let key = parsed.get("apiKey").and_then(Value::as_str).unwrap_or("");
            emit(settings::login(api_url, key))
        }
        "test_api_key" => emit(settings::status(api_url, None)),
        "delete_api_key" => emit(settings::logout()),
        other => cdx_cgi::emit_error(400, &format!("Neznámá akce: {other}")),
    }
}

fn emit(result: SettingsResult<Value>) -> ExitCode {
    match result {
        Ok(value) => cdx_cgi::emit_json(&value),
        Err(error) => cdx_cgi::emit_error(error.http_status(), &error.to_string()),
    }
}

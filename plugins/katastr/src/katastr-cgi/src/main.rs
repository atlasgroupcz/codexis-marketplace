//! CGI handler for the katastr plugin component.
//!
//! Replaces the Python `katastr.py`. Handles JSON overview/detail/settings/
//! notify endpoints and POST action dispatch (add/remove/check/confirm/
//! set-label/save-api-key/save-notify-settings) - full parity with the Python
//! implementation, sharing business logic with `katastr-cli` via the
//! `katastr-core` crate. The SPA shell (`index.html`) is served by cdx-daemon's
//! SpaCgi runtime directly, so this binary only runs for JSON GET / action POST.

use std::env;
use std::process::ExitCode;

use cdx_cgi::{emit_error, emit_json, parse_query, read_body};
use katastr_core::state::{find_by_uuid, Change, ProceedingState};
use katastr_core::tracking::{self, now_utc, OverviewEntry};
use katastr_core::{notify, notify_settings, settings, KatastrError};
use serde::Serialize;
use serde_json::Value;

fn main() -> ExitCode {
    let method = env::var("REQUEST_METHOD").unwrap_or_else(|_| "GET".into());
    let query = env::var("QUERY_STRING").unwrap_or_default();

    // Keep the user settings file present + migrated, like the Python handler.
    let _ = notify_settings::ensure_settings();

    match method.as_str() {
        "GET" => handle_get_json(&query),
        "POST" => handle_post(),
        other => emit_error(405, &format!("Method {other} not allowed")),
    }
}

// ── GET JSON dispatch ────────────────────────────────────────────────────────

fn handle_get_json(query: &str) -> ExitCode {
    let params = parse_query(query);

    if params.iter().any(|(k, _)| k == "settings") {
        return handle_settings();
    }

    if let Some((_, view)) = params.iter().find(|(k, _)| k == "view") {
        if view.trim() == "notify" {
            return handle_notify_view();
        }
    }

    if let Some((_, uuid)) = params.iter().find(|(k, _)| k == "uuid") {
        let uuid = uuid.trim();
        if !uuid.is_empty() {
            return handle_detail(uuid);
        }
    }

    handle_overview()
}

fn handle_settings() -> ExitCode {
    let status = settings::key_status();
    if status.configured {
        emit_json(&SettingsResponse {
            configured: true,
            masked_key: Some(status.masked),
        })
    } else {
        emit_json(&SettingsResponse {
            configured: false,
            masked_key: None,
        })
    }
}

fn handle_notify_view() -> ExitCode {
    emit_json(&NotifyResponse {
        mode: "notify",
        settings: notify_settings::get_notify(),
        user_email: notify::query_me_email().unwrap_or_default(),
    })
}

fn handle_overview() -> ExitCode {
    let entries = tracking::list_overview();
    let status = settings::key_status();
    emit_json(&OverviewResponse {
        mode: "overview",
        generated_at: now_utc(),
        proceedings: entries,
        api_key_configured: status.configured,
        api_key_masked: status.masked,
    })
}

fn handle_detail(uuid: &str) -> ExitCode {
    match find_by_uuid(uuid) {
        Some(state) => emit_json(&DetailResponse {
            mode: "detail",
            generated_at: now_utc(),
            proceeding: DetailEntry::from_state(&state),
        }),
        None => emit_error(404, "Proceeding not found"),
    }
}

// ── POST dispatch ───────────────────────────────────────────────────────────

fn handle_post() -> ExitCode {
    let body = match read_body() {
        Ok(b) => b,
        Err(e) => return emit_error(400, &format!("Failed to read body: {e}")),
    };
    let parsed: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) if body.trim().is_empty() => Value::Object(Default::default()),
        Err(_) => return emit_error(400, "Invalid JSON body"),
    };

    let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("");

    match action {
        "save_api_key" => post_save_api_key(&parsed),
        "delete_api_key" => post_delete_api_key(),
        "add_proceeding" => post_add(&parsed),
        "remove" => post_remove(&parsed),
        "check_all" => post_check_all(),
        "confirm_change" => post_confirm(&parsed),
        "set_label" => post_set_label(&parsed),
        "save_notify_settings" => post_save_notify_settings(&parsed),
        "" => emit_error(400, "Missing action"),
        other => emit_error(400, &format!("Unknown action: {other}")),
    }
}

fn post_save_api_key(body: &Value) -> ExitCode {
    let api_key = body.get("apiKey").and_then(|v| v.as_str()).unwrap_or("").trim();
    if api_key.is_empty() {
        return emit_error(400, "API klíč je prázdný.");
    }
    match settings::set_api_key(api_key) {
        Ok(_) => emit_json(&serde_json::json!({"ok": true, "saved": true})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_delete_api_key() -> ExitCode {
    match settings::write_api_key("") {
        Ok(_) => emit_json(&serde_json::json!({"ok": true})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_add(body: &Value) -> ExitCode {
    let cislo = body.get("cislo_rizeni").and_then(|v| v.as_str()).unwrap_or("").trim();
    let label = body.get("label").and_then(|v| v.as_str()).unwrap_or("").trim();
    match tracking::add(cislo, label) {
        Ok(state) => emit_json(&serde_json::json!({
            "ok": true,
            "proceeding": OverviewEntry::from_state(&state),
        })),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_remove(body: &Value) -> ExitCode {
    let state = match resolve_uuid(body) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match tracking::remove(&state.cislo_rizeni) {
        Ok(_) => emit_json(&serde_json::json!({"ok": true})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_check_all() -> ExitCode {
    let results = tracking::check_all();

    // Push e-mail + in-app for freshly detected changes - mirrors the CLI check
    // so a change is notified regardless of which route (automation, chat, FE)
    // first catches it. Best-effort.
    notify_results(&results);

    let checked = results.len();
    let new_changes = results.iter().filter(|r| r.has_change()).count();
    let errors: Vec<_> = results
        .iter()
        .filter(|r| !r.ok)
        .map(|r| {
            serde_json::json!({
                "cislo_rizeni": r.cislo_rizeni,
                "error": r.error.clone().unwrap_or_default(),
            })
        })
        .collect();
    emit_json(&serde_json::json!({
        "ok": true,
        "checked": checked,
        "new_changes": new_changes,
        "errors": errors,
    }))
}

fn post_confirm(body: &Value) -> ExitCode {
    let state = match resolve_uuid(body) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let change_index: Option<usize> = body
        .get("change_index")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    match tracking::confirm(&state.cislo_rizeni, change_index) {
        Ok(marked) => emit_json(&serde_json::json!({"ok": true, "marked": marked})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_set_label(body: &Value) -> ExitCode {
    let state = match resolve_uuid(body) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let label = body.get("label").and_then(|v| v.as_str()).unwrap_or("");
    match tracking::set_label(&state.cislo_rizeni, label) {
        Ok(_) => emit_json(&serde_json::json!({"ok": true})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_save_notify_settings(body: &Value) -> ExitCode {
    // Drop the signed-in user's own address (they always receive it); guard
    // None so a save that omits `recipients` leaves the stored list untouched.
    let recipients: Option<Vec<String>> = body.get("recipients").and_then(|v| v.as_array()).map(|arr| {
        let raw: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(String::from)).collect();
        notify::without_self(&raw)
    });
    let email = body.get("email").and_then(|v| v.as_bool());
    let in_app = body.get("inApp").and_then(|v| v.as_bool());

    match notify_settings::set_notify(email, in_app, recipients) {
        Ok(cfg) => emit_json(&serde_json::json!({"ok": true, "settings": cfg})),
        Err(e) => emit_katastr_error(&e),
    }
}

/// Push e-mail + in-app for freshly detected changes. Best-effort: never fails
/// the request.
fn notify_results(results: &[tracking::CheckResult]) {
    let cfg = notify_settings::get_notify();
    if cfg.email {
        let to = notify::resolve_to(&cfg.recipients);
        let _ = notify::email_change_summary(results, to);
    }
    if cfg.in_app {
        notify::inapp_change_summary(results);
    }
}

// ── shared uuid resolution ────────────────────────────────────────────────────

/// Look up a proceeding by `uuid` from the body, emitting the right error
/// response (and returning `Err(ExitCode)`) when missing/not found.
fn resolve_uuid(body: &Value) -> Result<ProceedingState, ExitCode> {
    let uuid = body.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
    if uuid.is_empty() {
        return Err(emit_error(400, "uuid required"));
    }
    match find_by_uuid(uuid) {
        Some(s) => Ok(s),
        None => Err(emit_error(404, "Proceeding not found")),
    }
}

// ── response types ───────────────────────────────────────────────────────────

#[derive(Serialize)]
struct OverviewResponse {
    mode: &'static str,
    generated_at: String,
    proceedings: Vec<OverviewEntry>,
    api_key_configured: bool,
    api_key_masked: String,
}

#[derive(Serialize)]
struct DetailResponse {
    mode: &'static str,
    generated_at: String,
    proceeding: DetailEntry,
}

#[derive(Serialize)]
struct DetailEntry {
    #[serde(flatten)]
    overview: OverviewEntry,
    changes: Vec<Change>,
}

impl DetailEntry {
    fn from_state(state: &ProceedingState) -> Self {
        Self {
            overview: OverviewEntry::from_state(state),
            changes: state.changes.clone(),
        }
    }
}

#[derive(Serialize)]
struct SettingsResponse {
    configured: bool,
    #[serde(rename = "maskedKey", skip_serializing_if = "Option::is_none")]
    masked_key: Option<String>,
}

#[derive(Serialize)]
struct NotifyResponse {
    mode: &'static str,
    settings: notify_settings::NotifyConfig,
    #[serde(rename = "userEmail")]
    user_email: String,
}

fn emit_katastr_error(err: &KatastrError) -> ExitCode {
    let (status, msg) = match err {
        KatastrError::ApiKeyMissing(m) => (412, m.as_str()),
        KatastrError::ApiKeyInvalid(m) => (401, m.as_str()),
        KatastrError::ApiNetwork(m) => (503, m.as_str()),
        KatastrError::ApiHttp { status, message } => (*status, message.as_str()),
        KatastrError::InvalidProceedingNumber(m) => (400, m.as_str()),
        KatastrError::InvalidRecipient(m) => (400, m.as_str()),
        KatastrError::ProceedingNotFound(m) => (404, m.as_str()),
        KatastrError::ProceedingNotTracked(m) => (404, m.as_str()),
        KatastrError::ProceedingAlreadyTracked(m) => (409, m.as_str()),
        KatastrError::Notify(_) | KatastrError::Io(_) | KatastrError::Json(_) => {
            (500, "Internal error")
        }
    };
    emit_error(status, msg)
}

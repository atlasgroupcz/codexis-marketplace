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
use katastr_core::state::{Change, Operace, ProceedingState};
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

    if let Some((_, view)) = params.iter().find(|(k, _)| k == "view") {
        if view.trim() == "notify" {
            return handle_notify_view();
        }
    }

    if let Some((_, cislo)) = params.iter().find(|(k, _)| k == "cislo_rizeni") {
        let cislo = cislo.trim();
        if !cislo.is_empty() {
            return handle_detail(cislo);
        }
    }

    handle_overview()
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

fn handle_detail(cislo: &str) -> ExitCode {
    let canonical = match tracking::normalize_number(cislo) {
        Ok(c) => c,
        Err(_) => return emit_error(400, "Neplatné číslo řízení"),
    };
    match ProceedingState::load(&canonical) {
        Some(state) => emit_json(&DetailResponse {
            mode: "detail",
            generated_at: now_utc(),
            proceeding: DetailEntry::from_state(state),
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
            "proceeding": OverviewEntry::from_state(state),
        })),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_remove(body: &Value) -> ExitCode {
    let cislo = match required_cislo(body) {
        Ok(c) => c,
        Err(code) => return code,
    };
    match tracking::remove(&cislo) {
        Ok(_) => emit_json(&serde_json::json!({"ok": true})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_check_all() -> ExitCode {
    let results = tracking::check_all();

    // Push e-mail + in-app for freshly detected changes - mirrors the CLI check
    // so a change is notified regardless of which route (automation, chat, FE)
    // first catches it. Best-effort; the dispatch outcome only matters to the CLI.
    let _ = notify::dispatch(&results);

    let checked = results.len();
    let new_changes = results.iter().filter(|r| r.has_change()).count();
    let errors: Vec<_> = results
        .into_iter()
        .filter(|r| !r.ok)
        .map(|r| {
            serde_json::json!({
                "cislo_rizeni": r.display_name,
                "error": r.error.unwrap_or_default(),
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
    let cislo = match required_cislo(body) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let change_index: Option<usize> = body
        .get("change_index")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    match tracking::confirm(&cislo, change_index) {
        Ok(marked) => emit_json(&serde_json::json!({"ok": true, "marked": marked})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_set_label(body: &Value) -> ExitCode {
    let cislo = match required_cislo(body) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let label = body.get("label").and_then(|v| v.as_str()).unwrap_or("");
    match tracking::set_label(&cislo, label) {
        Ok(_) => emit_json(&serde_json::json!({"ok": true})),
        Err(e) => emit_katastr_error(&e),
    }
}

fn post_save_notify_settings(body: &Value) -> ExitCode {
    // Drops the signed-in user's own address; `None` when `recipients` is
    // omitted, leaving the stored list untouched.
    let recipients = notify::recipients_from_body(body);
    let email = body.get("email").and_then(|v| v.as_bool());
    let in_app = body.get("inApp").and_then(|v| v.as_bool());

    match notify_settings::set_notify(email, in_app, recipients) {
        Ok(cfg) => emit_json(&serde_json::json!({"ok": true, "settings": cfg})),
        Err(e) => emit_katastr_error(&e),
    }
}

// ── shared request parsing ────────────────────────────────────────────────────

/// Pull the required `cislo_rizeni` (canonical proceeding number) from a POST
/// body, emitting a 400 (and returning `Err(ExitCode)`) when it's missing. The
/// downstream `tracking::*` call normalizes it and reports "not tracked" itself,
/// so no lookup is needed here.
fn required_cislo(body: &Value) -> Result<String, ExitCode> {
    let cislo = body
        .get("cislo_rizeni")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if cislo.is_empty() {
        return Err(emit_error(400, "cislo_rizeni required"));
    }
    Ok(cislo.to_string())
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
    // The overview projection no longer carries the operations list (the list
    // view only needs the count); the detail view does, so re-attach it here at
    // the same flattened JSON level the FE already reads.
    provedene_operace: Vec<Operace>,
    changes: Vec<Change>,
}

impl DetailEntry {
    fn from_state(state: ProceedingState) -> Self {
        // Single-item detail path, so cloning the two vecs the overview
        // projection would otherwise consume is cheap here (not the hot list).
        let provedene_operace = state.provedene_operace.clone();
        let changes = state.changes.clone();
        Self {
            overview: OverviewEntry::from_state(state),
            provedene_operace,
            changes,
        }
    }
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

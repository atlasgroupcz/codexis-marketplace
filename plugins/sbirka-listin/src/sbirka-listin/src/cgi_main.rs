//! CGI handler for the sbirka-listin plugin component.
//!
//! Serves the dashboard's JSON: GET returns the overview / one subject's detail
//! / the notify settings; POST dispatches actions (add, remove, check, confirm,
//! save notify settings, download a document). Business logic is shared with
//! `sbirka-listin-cli` through the crate lib, so a change is handled identically
//! whichever route (chat, CLI, automation, dashboard) drives it. The SPA shell
//! (`index.html`) is served by the daemon directly; this binary only runs for
//! JSON GET / action POST.

use std::env;
use std::fmt::Display;
use std::process::ExitCode;

use cdx_cgi::{emit_error, emit_json, parse_query, read_body};
use cdx_common::ico::normalize_ico;
use cdx_common::time::now_utc;
use cdx_notify::notify::{query_me_email, recipients_from_body};
use cdx_notify::notify_settings::NotifySettings;
use cdx_watch::change::{confirm_changes, unconfirmed_count};
use cdx_watch::track::check_all;
use serde_json::{json, Value};

use sbirka_listin_core::api;
use sbirka_listin_core::dokument;
use sbirka_listin_core::http::PortalClient;
use sbirka_listin_core::notify;
use sbirka_listin_core::watch::{self, OverviewRow, SbirkaListinWatcher, Snapshot};

fn main() -> ExitCode {
    let method = env::var("REQUEST_METHOD").unwrap_or_else(|_| "GET".into());
    let query = env::var("QUERY_STRING").unwrap_or_default();

    // Keep the user settings file present + migrated on the dashboard entry too,
    // matching the CLI's startup. An app-only user never runs the CLI, so the
    // CGI must ensure it itself. Best-effort: a settings write failure must not
    // fail the request.
    let _ = NotifySettings::new(watch::APP_ID).ensure_settings();

    match method.as_str() {
        "GET" => handle_get(&query),
        "POST" => handle_post(),
        other => emit_error(405, &format!("Method {other} not allowed")),
    }
}

/// A domain error as a CGI error response with the given status. Used with
/// `.map_err(|e| fail(status, e))?` so each handler reads as a linear happy path.
fn fail(status: u16, e: impl Display) -> ExitCode {
    emit_error(status, &format!("{e:#}"))
}

// ── GET ──────────────────────────────────────────────────────────────────────

fn handle_get(query: &str) -> ExitCode {
    let params = parse_query(query);

    if params.iter().any(|(k, v)| k == "view" && v.trim() == "notify") {
        return handle_notify_view();
    }
    if let Some((_, ico)) = params.iter().find(|(k, _)| k == "ico") {
        if !ico.trim().is_empty() {
            return handle_detail(ico.trim());
        }
    }
    handle_overview()
}

fn handle_overview() -> ExitCode {
    emit_json(&json!({
        "mode": "overview",
        "generated_at": now_utc(),
        "subjekty": watch::overview_rows(&watch::store()),
    }))
}

fn handle_detail(ico: &str) -> ExitCode {
    let ico8 = match normalize_ico(ico) {
        Ok(v) => v,
        Err(e) => return emit_error(400, &e.to_string()),
    };
    let Some(s) = watch::store().load::<Snapshot>(&ico8) else {
        return emit_error(404, &format!("IČO {ico8} není sledované"));
    };
    // The stored listiny map is keyed by číslo; hand the SPA a stable array.
    // `path` is present when the document is already downloaded (→ the dashboard
    // shows "Otevřít" instead of "Stáhnout"); re-checked on every load.
    let listiny: Vec<Value> = s
        .listiny
        .iter()
        .map(|(cislo, l)| {
            let path = dokument::stored_path(&ico8, cislo).map(|p| p.to_string_lossy().into_owned());
            json!({ "cislo": cislo, "typ": l.typ, "zalozeno": l.zalozeno, "path": path })
        })
        .collect();
    emit_json(&json!({
        "mode": "detail",
        "generated_at": now_utc(),
        "subjekt": {
            "ico": s.ico,
            "nazev": s.nazev,
            "spisovaznacka": s.spisovaznacka,
            "sidlo": s.sidlo,
            "listiny": listiny,
            "changes": s.changes,
            "added_on": s.added_on,
            "last_check_at": s.last_check_at,
        },
    }))
}

fn handle_notify_view() -> ExitCode {
    emit_json(&json!({
        "mode": "notify",
        "settings": NotifySettings::new(watch::APP_ID).get_notify(),
        "userEmail": query_me_email().unwrap_or_default(),
    }))
}

// ── POST ─────────────────────────────────────────────────────────────────────

fn handle_post() -> ExitCode {
    let body = match read_body() {
        Ok(b) => b,
        Err(e) => return emit_error(400, &format!("Nepodařilo se přečíst tělo: {e}")),
    };
    let parsed: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) if body.trim().is_empty() => Value::Object(Default::default()),
        Err(_) => return emit_error(400, "Neplatné JSON tělo"),
    };
    let action = parsed.get("action").and_then(Value::as_str).unwrap_or("");
    // Each handler returns its response either way; `Err` just carries the
    // early-return error response, so `?` inside them reads as a happy path.
    let outcome = match action {
        "add" => post_add(&parsed),
        "remove" => post_remove(&parsed),
        "check_all" => post_check_all(),
        "confirm_change" => post_confirm(&parsed),
        "save_notify_settings" => post_save_notify_settings(&parsed),
        "download" => post_download(&parsed),
        "" => Ok(emit_error(400, "Chybí action")),
        other => Ok(emit_error(400, &format!("Neznámá action: {other}"))),
    };
    outcome.unwrap_or_else(|response| response)
}

type Handled = Result<ExitCode, ExitCode>;

/// The canonical IČO from a body field, or a 400 response.
fn body_ico(body: &Value) -> Result<String, ExitCode> {
    let raw = body.get("ico").and_then(Value::as_str).unwrap_or("").trim();
    normalize_ico(raw).map_err(|e| emit_error(400, &e.to_string()))
}

fn post_add(body: &Value) -> Handled {
    let ico8 = body_ico(body)?;
    let store = watch::store();
    if store.exists(&ico8) {
        return Ok(emit_error(409, &format!("IČO {ico8} už je sledované")));
    }
    let client = PortalClient::new().map_err(|e| fail(500, e))?;
    let detail = api::fetch_detail(&client, &ico8).map_err(|e| fail(502, e))?;
    let snapshot = watch::seed(&detail);
    store.save(&ico8, &snapshot).map_err(|e| fail(500, e))?;
    Ok(emit_json(
        &json!({ "ok": true, "subjekt": OverviewRow::from_snapshot(snapshot, 0) }),
    ))
}

fn post_remove(body: &Value) -> Handled {
    let ico8 = body_ico(body)?;
    let store = watch::store();
    if !store.exists(&ico8) {
        return Ok(emit_error(404, &format!("IČO {ico8} není sledované")));
    }
    store.remove(&ico8).map_err(|e| fail(500, e))?;
    Ok(emit_json(&json!({ "ok": true })))
}

fn post_check_all() -> Handled {
    let client = PortalClient::new().map_err(|e| fail(500, e))?;
    let watcher = SbirkaListinWatcher { client: &client };
    let store = watch::store();
    let results = check_all(&store, &watcher);
    // Detect-and-notify, same as the CLI: a change is notified whichever route
    // catches it first. Best-effort - never fails the request.
    let notifikace = notify::dispatch(&results);

    let errors: Vec<Value> = results
        .iter()
        .filter(|r| !r.ok)
        .map(|r| json!({ "ico": r.slug, "error": r.error.clone().unwrap_or_default() }))
        .collect();
    Ok(emit_json(&json!({
        "ok": true,
        "checked": results.len(),
        "new_changes": results.iter().filter(|r| r.has_change()).count(),
        "errors": errors,
        "notifikace": notifikace,
    })))
}

fn post_confirm(body: &Value) -> Handled {
    let ico8 = body_ico(body)?;
    let store = watch::store();
    let Some(mut snapshot) = store.load::<Snapshot>(&ico8) else {
        return Ok(emit_error(404, &format!("IČO {ico8} není sledované")));
    };
    let index = body
        .get("change_index")
        .and_then(Value::as_u64)
        .map(|n| n as usize);
    let marked = confirm_changes(&mut snapshot.changes, index, &now_utc()).map_err(|e| fail(400, e))?;
    store.save(&ico8, &snapshot).map_err(|e| fail(500, e))?;
    Ok(emit_json(&json!({
        "ok": true,
        "marked": marked,
        "nepotvrzenych_zmen": unconfirmed_count(&snapshot.changes),
    })))
}

fn post_save_notify_settings(body: &Value) -> Handled {
    let email = body.get("email").and_then(Value::as_bool);
    let in_app = body.get("inApp").and_then(Value::as_bool);
    // Drops the signed-in user's own address; `None` when `recipients` is
    // omitted, leaving the stored list untouched.
    let recipients = recipients_from_body(body);
    match NotifySettings::new(watch::APP_ID).set_notify(email, in_app, recipients) {
        Ok(cfg) => Ok(emit_json(&json!({ "ok": true, "settings": cfg }))),
        Err(e) => Ok(emit_error(e.http_status(), &e.to_string())),
    }
}

fn post_download(body: &Value) -> Handled {
    let ico8 = body_ico(body)?;
    let cislo = body.get("cislo").and_then(Value::as_str).unwrap_or("").trim();
    if cislo.is_empty() {
        return Ok(emit_error(400, "Chybí číslo listiny"));
    }
    let client = PortalClient::new().map_err(|e| fail(500, e))?;
    let f = dokument::fetch(&client, &ico8, cislo).map_err(|e| fail(502, e))?;
    Ok(emit_json(&json!({
        "ok": true,
        "ico": ico8,
        "cislo": f.cislo,
        "path": f.path,
        "url": f.url,
        "reused": f.reused,
    })))
}

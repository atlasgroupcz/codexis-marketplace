use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};
use std::thread::sleep;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_secs(4);
const POLL_DEADLINE: Duration = Duration::from_secs(600);

/// Import chats, agents and files from Codexis 1.0.
///
/// The whole import runs inside the daemon. `startCodexisImport` kicks it off
/// asynchronously and returns immediately; we then poll `codexisImportStatus`
/// (up to 10 minutes) until the run reports SUCCEEDED or FAILED. This keeps each
/// HTTP request short, so the import is not bound by client/proxy read timeouts.
/// Already-present chats and agents are skipped, so re-running is idempotent.
pub fn sync(
    client: &GraphQLClient,
    dry_run: bool,
    format: OutputFormat,
) -> Result<(), CdxctlError> {
    if dry_run {
        eprintln!("--dry-run is no longer supported: the import now runs inside the daemon.");
        return Ok(());
    }

    eprintln!("Starting Codexis 1.0 import ...");
    let started = client.execute(graphql::START_CODEXIS_IMPORT, json!({}))?;
    let state = started
        .get("startCodexisImport")
        .ok_or_else(|| CdxctlError::Parse("response missing 'startCodexisImport'".into()))?;
    if status_of(state) == "FAILED" {
        return Err(CdxctlError::Network(format!(
            "Codexis import failed: {}",
            error_of(state)
        )));
    }

    let deadline = Instant::now() + POLL_DEADLINE;
    loop {
        let data = client.execute(graphql::CODEXIS_IMPORT_STATUS, json!({}))?;
        if let Some(state) = data
            .get("codexisImportStatus")
            .filter(|value| !value.is_null())
        {
            match status_of(state).as_str() {
                "SUCCEEDED" => {
                    eprintln!("Codexis 1.0 import finished.");
                    print_output(&summary(state), format);
                    return Ok(());
                }
                "FAILED" => {
                    return Err(CdxctlError::Network(format!(
                        "Codexis import failed: {}",
                        error_of(state)
                    )));
                }
                _ => {}
            }
        }
        if Instant::now() >= deadline {
            return Err(CdxctlError::Network(
                "Codexis import did not finish within 10 minutes".into(),
            ));
        }
        eprintln!("  … still importing");
        sleep(POLL_INTERVAL);
    }
}

fn summary(state: &Value) -> Value {
    json!({
        "snapshotId": field(state, "snapshotId"),
        "chatsImported": field(state, "chatsImported"),
        "chatsSkipped": field(state, "chatsSkipped"),
        "messagesImported": field(state, "messagesImported"),
        "agentsImported": field(state, "agentsImported"),
        "agentsSkipped": field(state, "agentsSkipped"),
        "filesImported": field(state, "filesImported"),
    })
}

fn status_of(state: &Value) -> String {
    state
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string()
}

fn error_of(state: &Value) -> String {
    state
        .get("error")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown error")
        .to_string()
}

fn field<'a>(state: &'a Value, key: &str) -> &'a Value {
    state.get(key).unwrap_or(&Value::Null)
}

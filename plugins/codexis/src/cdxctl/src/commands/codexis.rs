use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};

/// Import chats, agents and files from Codexis 1.0.
///
/// The whole import runs inside the daemon: it exports a fresh snapshot, then
/// rebuilds each chat, message, custom GPT (as an agent) and attachment through
/// the daemon's own repositories, so the data lands in the exact on-disk layout
/// the app expects. Already-present chats and agents are skipped, so re-running
/// is idempotent.
pub fn sync(client: &GraphQLClient, dry_run: bool, format: OutputFormat) -> Result<(), CdxctlError> {
    if dry_run {
        eprintln!("--dry-run is no longer supported: the import now runs inside the daemon.");
        return Ok(());
    }

    eprintln!("Importing Codexis 1.0 data ...");
    let data = client.execute(graphql::START_CODEXIS_IMPORT, json!({}))?;
    let result = data
        .get("startCodexisImport")
        .ok_or_else(|| CdxctlError::Parse("response missing 'startCodexisImport'".into()))?;
    print_output(&summary(result), format);
    Ok(())
}

fn summary(result: &Value) -> Value {
    json!({
        "snapshotId": field(result, "snapshotId"),
        "chatsImported": field(result, "chatsImported"),
        "chatsSkipped": field(result, "chatsSkipped"),
        "messagesImported": field(result, "messagesImported"),
        "agentsImported": field(result, "agentsImported"),
        "agentsSkipped": field(result, "agentsSkipped"),
        "filesImported": field(result, "filesImported"),
    })
}

fn field<'a>(result: &'a Value, key: &str) -> &'a Value {
    result.get(key).unwrap_or(&Value::Null)
}

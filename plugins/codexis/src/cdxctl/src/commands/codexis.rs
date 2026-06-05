use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Import chats and files from Codexis 1.0.
///
/// Asks the daemon for a one-hour presigned snapshot (`startCodexisExport`),
/// downloads + unzips it, then writes each chat into the daemon's on-disk
/// layout under `~/.cdx/chats/<slug>/<chatId>/` so it shows up in chat history.
/// Already-present chats are skipped, so re-running is idempotent.
pub fn sync(client: &GraphQLClient, dry_run: bool, format: OutputFormat) -> Result<(), CdxctlError> {
    let (snapshot_id, archive_url) = request_export(client)?;

    let work_dir = std::env::temp_dir().join(format!("codexis-sync-{snapshot_id}"));
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir)?;
    let zip_path = work_dir.join("snapshot.zip");
    let extract_dir = work_dir.join("extracted");
    fs::create_dir_all(&extract_dir)?;

    eprintln!("Downloading Codexis 1.0 snapshot {snapshot_id} ...");
    download(&archive_url, &zip_path)?;
    extract_zip(&zip_path, &extract_dir)?;

    let manifest = read_json(&extract_dir.join("manifest.json"))?;
    let home = home_dir()?;
    let chats = import_chats(&manifest, &extract_dir, &home, dry_run)?;
    let files_dest = home.join("codexis-1.0-import");
    let files_imported = import_files(&manifest, &extract_dir, &files_dest, dry_run)?;

    if !dry_run {
        // Chats were written straight to disk, bypassing the daemon's history index.
        // The daemon only rebuilds that index when it is missing, so drop it here to
        // force a rebuild from every chat on disk (old chats included) on next access.
        invalidate_history_index(&home);
        let _ = fs::remove_dir_all(&work_dir);
    }

    let summary = json!({
        "snapshotId": snapshot_id,
        "dryRun": dry_run,
        "chatsImported": chats.imported,
        "chatsSkipped": chats.skipped,
        "messagesImported": chats.messages,
        "filesImported": files_imported,
        "filesLocation": files_dest.to_string_lossy(),
    });
    print_output(&summary, format);
    Ok(())
}

fn request_export(client: &GraphQLClient) -> Result<(String, String), CdxctlError> {
    let data = client.execute(graphql::START_CODEXIS_EXPORT, json!({}))?;
    let export = data
        .get("startCodexisExport")
        .ok_or_else(|| CdxctlError::Parse("response missing 'startCodexisExport'".into()))?;
    let snapshot_id = export
        .get("snapshotId")
        .and_then(Value::as_str)
        .unwrap_or("snapshot")
        .to_string();
    let archive_url = export
        .get("archiveUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| CdxctlError::Parse("response missing 'archiveUrl'".into()))?
        .to_string();
    Ok((snapshot_id, archive_url))
}

#[derive(Default)]
struct ChatImport {
    imported: u64,
    skipped: u64,
    messages: u64,
}

fn import_chats(
    manifest: &Value,
    extract_dir: &Path,
    home: &Path,
    dry_run: bool,
) -> Result<ChatImport, CdxctlError> {
    let working_directory = home.to_string_lossy().to_string();
    let chats_root = home
        .join(".cdx")
        .join("chats")
        .join(working_directory.replace('/', "-"));

    let messages_by_chat = group_message_files_by_chat(manifest);
    let mut stats = ChatImport::default();
    let Some(chats) = manifest.get("chats").and_then(Value::as_array) else {
        return Ok(stats);
    };
    for chat_ref in chats {
        let Some(file) = chat_ref.get("file").and_then(Value::as_str) else { continue };
        let Ok(chat) = read_json(&extract_dir.join(file)) else { continue };
        let Some(id) = chat.get("id").and_then(Value::as_str) else { continue };

        let chat_dir = chats_root.join(id);
        if chat_dir.exists() {
            stats.skipped += 1;
            continue;
        }

        let messages = collect_chat_messages(messages_by_chat.get(id), extract_dir);
        stats.messages += messages.len() as u64;
        stats.imported += 1;
        if !dry_run {
            write_chat(&chat_dir, id, &chat, &messages, &working_directory)?;
        }
    }
    Ok(stats)
}

/// Group exported message file paths by their chat id, from the manifest.
fn group_message_files_by_chat(manifest: &Value) -> HashMap<String, Vec<String>> {
    let mut by_chat: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(items) = manifest.get("chatMessages").and_then(Value::as_array) {
        for m in items {
            if let (Some(chat_id), Some(file)) = (
                m.get("chatId").and_then(Value::as_str),
                m.get("file").and_then(Value::as_str),
            ) {
                by_chat.entry(chat_id.to_string()).or_default().push(file.to_string());
            }
        }
    }
    by_chat
}

/// Read + map a chat's exported messages into `(role, text, createdAt)`, oldest first.
fn collect_chat_messages(files: Option<&Vec<String>>, extract_dir: &Path) -> Vec<(String, String, String)> {
    let mut messages: Vec<(String, String, String)> = Vec::new();
    let Some(files) = files else { return messages };
    for file in files {
        let Ok(message) = read_json(&extract_dir.join(file)) else { continue };
        let Some(role) = map_role(message.get("authorType").and_then(Value::as_str)) else {
            continue;
        };
        let text = message
            .get("payload")
            .and_then(|p| p.get("text"))
            .and_then(Value::as_str)
            .unwrap_or("");
        if text.is_empty() {
            continue;
        }
        let created_at = message
            .get("createdAt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        messages.push((role.to_string(), text.to_string(), created_at));
    }
    messages.sort_by(|a, b| a.2.cmp(&b.2));
    messages
}

/// Write one chat into the daemon's on-disk layout (chat.json, messages.jsonl, model-config.json).
fn write_chat(
    chat_dir: &Path,
    id: &str,
    chat: &Value,
    messages: &[(String, String, String)],
    working_directory: &str,
) -> Result<(), CdxctlError> {
    let created_at = chat.get("createdAt").and_then(Value::as_str).unwrap_or("");
    let updated_at = chat
        .get("updatedAt")
        .and_then(Value::as_str)
        .unwrap_or(created_at);
    let title = chat
        .get("name")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("Codexis 1.0 chat");

    fs::create_dir_all(chat_dir)?;

    let chat_record = json!({
        "chatId": id,
        "workingDirectory": working_directory,
        "storageDirectory": chat_dir.to_string_lossy(),
        "createdAt": created_at,
        "modifiedAt": updated_at,
        "title": title,
        "status": "READY",
        "contextTokens": 0,
        "origin": { "type": "USER" }
    });
    fs::write(chat_dir.join("chat.json"), serde_json::to_vec(&chat_record)?)?;

    let mut messages_jsonl = String::new();
    for (role, content, created_at) in messages {
        let line = json!({
            "role": role,
            "messageId": Uuid::new_v4().to_string(),
            "status": "DONE",
            "parts": [{ "type": "text", "partId": Uuid::new_v4().to_string(), "content": content }],
            "createdAt": created_at
        });
        messages_jsonl.push_str(&serde_json::to_string(&line)?);
        messages_jsonl.push('\n');
    }
    fs::write(chat_dir.join("messages.jsonl"), messages_jsonl.as_bytes())?;

    let model_config = json!({
        "chatId": id,
        "model": "GPT_5_4",
        "json": "{\"version\":2,\"targetModel\":\"gpt-5.4\",\"verbosity\":\"LOW\",\"reasoningEffort\":\"MEDIUM\",\"serviceTier\":\"STANDARD\",\"contextSize\":\"STANDARD\"}"
    });
    fs::write(chat_dir.join("model-config.json"), serde_json::to_vec(&model_config)?)?;
    Ok(())
}

/// Copy attached files into `~/codexis-1.0-import`, skipping ones already present.
fn import_files(
    manifest: &Value,
    extract_dir: &Path,
    files_dest: &Path,
    dry_run: bool,
) -> Result<u64, CdxctlError> {
    let Some(files) = manifest.get("files").and_then(Value::as_array) else {
        return Ok(0);
    };
    let mut imported = 0u64;
    for entry in files {
        let Some(file) = entry.get("file").and_then(Value::as_str) else { continue };
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| {
                Path::new(file)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file".to_string())
            });
        let src = extract_dir.join(file);
        if !src.exists() {
            continue;
        }
        let dest = files_dest.join(sanitize(&name));
        if dest.exists() {
            continue;
        }
        if dry_run {
            imported += 1;
            continue;
        }
        fs::create_dir_all(files_dest)?;
        if fs::copy(&src, &dest).is_ok() {
            imported += 1;
        }
    }
    Ok(imported)
}

/// Remove the daemon's chat history index so it is rebuilt from the chats on disk.
fn invalidate_history_index(home: &Path) {
    let cdx = home.join(".cdx");
    if let Ok(entries) = fs::read_dir(cdx.join("indices")) {
        for entry in entries.flatten() {
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with("chat-global-history-index")
            {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    let _ = fs::remove_dir_all(cdx.join("chat-index"));
}

fn map_role(author_type: Option<&str>) -> Option<&'static str> {
    match author_type {
        Some("USER") => Some("USER"),
        Some("AI") => Some("AI"),
        Some("SYSTEM") => Some("SYSTEM"),
        _ => None,
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c == '/' || c == '\\' { '_' } else { c })
        .collect()
}

fn home_dir() -> Result<PathBuf, CdxctlError> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| CdxctlError::Parse("HOME environment variable is not set".into()))
}

fn read_json(path: &Path) -> Result<Value, CdxctlError> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn extract_zip(zip_path: &Path, dest: &Path) -> Result<(), CdxctlError> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| CdxctlError::Network(format!("opening snapshot archive failed: {e}")))?;
    archive
        .extract(dest)
        .map_err(|e| CdxctlError::Network(format!("extracting snapshot archive failed: {e}")))?;
    Ok(())
}

fn download(url: &str, dest: &Path) -> Result<(), CdxctlError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()?;
    let mut response = client.get(url).send()?;
    if !response.status().is_success() {
        return Err(CdxctlError::Network(format!(
            "snapshot download failed: HTTP {}",
            response.status()
        )));
    }
    let mut out = fs::File::create(dest)?;
    response.copy_to(&mut out)?;
    Ok(())
}

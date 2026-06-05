use crate::client::GraphQLClient;
use crate::error::CdxctlError;
use crate::graphql;
use crate::output::{print_output, OutputFormat};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

/// Import chats and files from Codexis 1.0.
///
/// Asks the daemon for a one-hour presigned snapshot (`startCodexisExport`),
/// downloads + unzips it, then writes each chat into the daemon's on-disk
/// layout under `~/.cdx/chats/<slug>/<chatId>/` so it shows up in chat history.
/// Already-present chats are skipped, so re-running is idempotent.
pub fn sync(client: &GraphQLClient, dry_run: bool, format: OutputFormat) -> Result<(), CdxctlError> {
    let data = client.execute(graphql::START_CODEXIS_EXPORT, json!({}))?;
    let export = data
        .get("startCodexisExport")
        .ok_or_else(|| CdxctlError::Parse("response missing 'startCodexisExport'".into()))?;
    let snapshot_id = export
        .get("snapshotId")
        .and_then(Value::as_str)
        .unwrap_or("snapshot");
    let archive_url = export
        .get("archiveUrl")
        .and_then(Value::as_str)
        .ok_or_else(|| CdxctlError::Parse("response missing 'archiveUrl'".into()))?;

    let work_dir = std::env::temp_dir().join(format!("codexis-sync-{snapshot_id}"));
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir)?;
    let zip_path = work_dir.join("snapshot.zip");
    let extract_dir = work_dir.join("extracted");
    fs::create_dir_all(&extract_dir)?;

    eprintln!("Downloading Codexis 1.0 snapshot {snapshot_id} ...");
    download(archive_url, &zip_path)?;

    let status = Command::new("unzip")
        .arg("-o")
        .arg("-q")
        .arg(&zip_path)
        .arg("-d")
        .arg(&extract_dir)
        .status()?;
    if !status.success() {
        return Err(CdxctlError::Network("unzip failed on the downloaded snapshot".into()));
    }

    let manifest = read_json(&extract_dir.join("manifest.json"))?;

    let mut messages_by_chat: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(items) = manifest.get("chatMessages").and_then(Value::as_array) {
        for m in items {
            if let (Some(chat_id), Some(file)) = (
                m.get("chatId").and_then(Value::as_str),
                m.get("file").and_then(Value::as_str),
            ) {
                messages_by_chat
                    .entry(chat_id.to_string())
                    .or_default()
                    .push(file.to_string());
            }
        }
    }

    let home = home_dir()?;
    let working_directory = home.to_string_lossy().to_string();
    let slug = working_directory.replace('/', "-");
    let chats_root = home.join(".cdx").join("chats").join(&slug);

    let mut imported = 0u64;
    let mut skipped = 0u64;
    let mut messages_imported = 0u64;

    if let Some(chats) = manifest.get("chats").and_then(Value::as_array) {
        for c in chats {
            let Some(file) = c.get("file").and_then(Value::as_str) else { continue };
            let Ok(chat) = read_json(&extract_dir.join(file)) else { continue };
            let Some(id) = chat.get("id").and_then(Value::as_str) else { continue };

            let chat_dir = chats_root.join(id);
            if chat_dir.exists() {
                skipped += 1;
                continue;
            }

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

            let mut msgs: Vec<(String, String, String)> = Vec::new();
            if let Some(files) = messages_by_chat.get(id) {
                for mf in files {
                    let Ok(mv) = read_json(&extract_dir.join(mf)) else { continue };
                    let Some(role) = map_role(mv.get("authorType").and_then(Value::as_str)) else {
                        continue;
                    };
                    let text = mv
                        .get("payload")
                        .and_then(|p| p.get("text"))
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if text.is_empty() {
                        continue;
                    }
                    let ts = mv
                        .get("createdAt")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    msgs.push((role.to_string(), text.to_string(), ts));
                }
            }
            msgs.sort_by(|a, b| a.2.cmp(&b.2));

            if dry_run {
                imported += 1;
                messages_imported += msgs.len() as u64;
                continue;
            }

            fs::create_dir_all(&chat_dir)?;

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
            for (role, content, ts) in &msgs {
                let line = json!({
                    "role": role,
                    "messageId": Uuid::new_v4().to_string(),
                    "status": "DONE",
                    "parts": [{ "type": "text", "partId": Uuid::new_v4().to_string(), "content": content }],
                    "createdAt": ts
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

            imported += 1;
            messages_imported += msgs.len() as u64;
        }
    }

    let files_dest = home.join("codexis-1.0-import");
    let mut files_imported = 0u64;
    if let Some(files) = manifest.get("files").and_then(Value::as_array) {
        for f in files {
            let Some(file) = f.get("file").and_then(Value::as_str) else { continue };
            let name = f
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
                files_imported += 1;
                continue;
            }
            fs::create_dir_all(&files_dest)?;
            if fs::copy(&src, &dest).is_ok() {
                files_imported += 1;
            }
        }
    }

    if !dry_run {
        let _ = fs::remove_dir_all(&work_dir);
    }

    let summary = json!({
        "snapshotId": snapshot_id,
        "dryRun": dry_run,
        "chatsImported": imported,
        "chatsSkipped": skipped,
        "messagesImported": messages_imported,
        "filesImported": files_imported,
        "filesLocation": files_dest.to_string_lossy(),
    });
    print_output(&summary, format);
    Ok(())
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

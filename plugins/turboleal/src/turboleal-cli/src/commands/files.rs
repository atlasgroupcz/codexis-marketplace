use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::api::{ApiClient, Upload};
use crate::cli::FileCommand;
use crate::commands::{collect, print_next_cursor};
use crate::model::File;
use crate::output::{Output, table, text};

const FILES_PER_REQUEST: usize = 20;

pub fn run(client: &ApiClient, output: &Output, command: FileCommand) -> Result<()> {
    match command {
        FileCommand::List { watch_set, page } => {
            let files = collect(&page, |after, limit| client.files(watch_set, after, limit))?;
            output.print(&files, |page| {
                print(&page.items);
                print_next_cursor(page.next_cursor);
            })
        }
        FileCommand::Upload {
            watch_set,
            paths,
            folder,
        } => upload(client, output, watch_set, &paths, folder.as_deref()),
        FileCommand::Delete { watch_set, file } => {
            client.delete_file(watch_set, file)?;
            output.message(&format!("Removed file {file}"));
            Ok(())
        }
    }
}

fn upload(
    client: &ApiClient,
    output: &Output,
    watch_set: Uuid,
    paths: &[PathBuf],
    folder: Option<&str>,
) -> Result<()> {
    let files = expand(paths, folder.unwrap_or(""))?;
    let mut uploaded = Vec::with_capacity(files.len());
    for batch in files.chunks(FILES_PER_REQUEST) {
        let uploads = batch.iter().map(read_upload).collect::<Result<Vec<_>>>()?;
        uploaded.extend(client.upload_files(watch_set, uploads)?);
    }
    output.print(&uploaded, |files| print(files))
}

/// Turns files and directories into (file, folder inside the watch set) pairs.
fn expand(paths: &[PathBuf], prefix: &str) -> Result<Vec<(PathBuf, String)>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let root = path.parent().unwrap_or(Path::new(""));
            for entry in WalkDir::new(path).sort_by_file_name() {
                let entry = entry.with_context(|| format!("cannot read {}", path.display()))?;
                if entry.file_type().is_file() {
                    let relative = entry
                        .path()
                        .parent()
                        .unwrap_or(root)
                        .strip_prefix(root)
                        .unwrap_or(Path::new(""));
                    files.push((entry.path().to_path_buf(), join(prefix, relative)));
                }
            }
        } else {
            files.push((path.clone(), join(prefix, Path::new(""))));
        }
    }
    Ok(files)
}

fn join(prefix: &str, relative: &Path) -> String {
    let relative = relative.to_string_lossy().replace('\\', "/");
    [prefix.trim_matches('/'), relative.trim_matches('/')]
        .into_iter()
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

fn read_upload((path, folder): &(PathBuf, String)) -> Result<Upload> {
    Ok(Upload {
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        folder: folder.clone(),
        content_type: content_type(path),
        bytes: fs::read(path).with_context(|| format!("cannot read {}", path.display()))?,
    })
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("pdf") => "application/pdf",
        Some("html" | "htm") => "text/html",
        Some("txt") => "text/plain",
        _ => "application/octet-stream",
    }
}

fn print(files: &[File]) {
    println!(
        "{}",
        table(
            ["id", "folder", "name", "status", "error"],
            files.iter().map(|file| {
                [
                    file.id.to_string(),
                    file.path.clone(),
                    file.name.clone(),
                    file.status.clone(),
                    text(&file.error),
                ]
            }),
        )
    );
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::join;

    #[test]
    fn join_skips_empty_segments() {
        assert_eq!(join("", Path::new("")), "");
        assert_eq!(
            join("/contracts/", Path::new("bakery/2026")),
            "contracts/bakery/2026"
        );
        assert_eq!(join("", Path::new("bakery")), "bakery");
    }
}

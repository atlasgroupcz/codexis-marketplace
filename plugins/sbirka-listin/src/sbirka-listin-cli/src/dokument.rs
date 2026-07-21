//! Fetching one document and keeping it where both the agent and the app can
//! reach it.
//!
//! Files live under
//! `$HOME/.cdx/apps/sbirka-listin/dokumenty/<ico>/<cislo>.<ext>`, where `<cislo>`
//! is the číslo listiny made filesystem-safe (`B-8573-SL95-MSPH`) — the app's own
//! directory, not the caller's cwd, because chat mode hides the working directory
//! from everything but apps. One subfolder per subject mirrors the watcher's
//! `subjekty/<ico>/` layout and lets "what has this company got downloaded" be a
//! single `read_dir` rather than a scan-and-filter of every subject's files.
//!
//! Re-fetching is avoided by looking for the file, not by keeping a registry: a
//! registry would go stale the moment the file is touched, whereas a missing
//! file simply downloads again. The extension is unknown until the response
//! arrives (PDF vs XHTML/ESEF), so the lookup globs `<cislo>.*`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::api;
use crate::http::PortalClient;
use crate::parse;

pub const COLLECTION: &str = "dokumenty";

/// `$HOME/.cdx/apps/sbirka-listin/dokumenty`.
pub fn dir() -> PathBuf {
    cdx_common::paths::app_dir(crate::watch::APP_ID).join(COLLECTION)
}

/// The per-subject folder `dokumenty/<ico>`.
pub fn subject_dir(ico8: &str) -> PathBuf {
    dir().join(ico8)
}

/// The stored file for a číslo listiny, when one is already there — matched by
/// its filesystem-safe stem, so an already downloaded document is found before
/// any network call.
fn existing(ico8: &str, stem: &str) -> Option<PathBuf> {
    let prefix = format!("{stem}.");
    fs::read_dir(subject_dir(ico8))
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&prefix))
        })
}

/// The stored file for a číslo listiny, if it is already downloaded. Public so
/// the dashboard can tell which listiny are on disk (→ "Otevřít" vs "Stáhnout");
/// re-checked on every detail load, so a removed file simply reverts to
/// "Stáhnout" with no stale state to maintain.
pub fn stored_path(ico8: &str, cislo: &str) -> Option<PathBuf> {
    existing(ico8, &parse::file_stem(cislo))
}

/// The extension for a Content-Type. Only the response header is authoritative:
/// listiny carry no reliable format hint before the fetch (PDF and XHTML/ESEF
/// both occur, older statements of listed companies being XHTML).
pub fn extension_for(content_type: Option<&str>) -> &'static str {
    match content_type.unwrap_or_default() {
        "application/pdf" => "pdf",
        "application/xhtml+xml" => "xhtml",
        "text/html" => "html",
        "application/zip" => "zip",
        "text/plain" => "txt",
        _ => "bin",
    }
}

/// Result of a fetch.
pub struct Fetched {
    pub path: PathBuf,
    pub cislo: String,
    /// The official download URL — known only when we actually resolved (i.e. not
    /// for a file served straight from disk).
    pub url: Option<String>,
    pub reused: bool,
}

/// Fetch a document by its číslo listiny unless it is already stored. Returns its
/// path either way.
pub fn fetch(client: &PortalClient, ico8: &str, cislo: &str) -> Result<Fetched> {
    let stem = parse::file_stem(cislo);
    if let Some(path) = existing(ico8, &stem) {
        return Ok(Fetched {
            path,
            cislo: parse::normalize_key(cislo),
            url: None,
            reused: true,
        });
    }

    let fetched = api::fetch_document(client, ico8, cislo)?;
    let path = subject_dir(ico8).join(format!(
        "{}.{}",
        parse::file_stem(&fetched.listina.cislo),
        extension_for(fetched.document.content_type.as_deref())
    ));
    write_atomically(&path, &fetched.document.bytes)
        .with_context(|| format!("nepodařilo se uložit dokument do {}", path.display()))?;
    Ok(Fetched {
        path,
        cislo: fetched.listina.cislo,
        url: Some(fetched.url),
        reused: false,
    })
}

/// Temp file in the destination dir, then rename — a crash leaves either nothing
/// or the whole document, never a truncated one that `existing()` would then
/// hand out as complete.
fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{}.tmp.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("dokument"),
        std::process::id()
    ));
    match fs::write(&tmp, bytes).and_then(|()| fs::rename(&tmp, path)) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&tmp);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_follows_the_content_type() {
        assert_eq!(extension_for(Some("application/pdf")), "pdf");
        // ČEZ's post-2020 účetní závěrky are ESEF/inline-XBRL, not PDF.
        assert_eq!(extension_for(Some("application/xhtml+xml")), "xhtml");
        assert_eq!(extension_for(None), "bin");
        assert_eq!(extension_for(Some("application/octet-stream")), "bin");
    }

    #[test]
    fn write_atomically_leaves_no_temp_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("27082440").join("B-8573-SL95-MSPH.pdf");
        write_atomically(&path, b"%PDF-1.4").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"%PDF-1.4");
        let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty());
    }
}

//! Fetching one document and keeping it where both the agent and the app can
//! reach it.
//!
//! Files live under
//! `$HOME/.cdx/apps/sbirka-listin/dokumenty/<ico>/<typ-slug>__<cislo>.<ext>`, where
//! `<cislo>` is the číslo listiny made filesystem-safe (`B-8573-SL95-MSPH`) and
//! `<typ-slug>` is the document type slugified (`ucetni-zaverka-2024`); an empty
//! typ falls back to the bare `<cislo>.<ext>`. This is the app's own directory,
//! not the caller's cwd, because chat mode hides the working directory from
//! everything but apps. One subfolder per subject mirrors the watcher's
//! `subjekty/<ico>/` layout and lets "what has this company got downloaded" be a
//! single `read_dir` rather than a scan-and-filter of every subject's files.
//!
//! Re-fetching is avoided by looking for the file, not by keeping a registry: a
//! registry would go stale the moment the file is touched, whereas a missing
//! file simply downloads again. The extension is unknown until the response
//! arrives (PDF vs XHTML/ESEF), so the lookup keys on the číslo segment of the
//! name rather than on the extension.

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

/// Whether a stored file name belongs to this číslo. The match is on the číslo
/// *segment* of the stem - the full stem equals `<cislo>` (empty-typ fallback)
/// or ends with `__<cislo>` (descriptive `<typ-slug>__<cislo>`). Keying on the
/// number, not the typ text, means a reworded typ never hides a download.
fn name_matches_cislo(file_name: &str, cislo_safe: &str) -> bool {
    if file_name.starts_with('.') {
        // hidden files and the `.<name>.tmp.<pid>` write leftovers
        return false;
    }
    let stem = file_name.rsplit_once('.').map_or(file_name, |(s, _)| s);
    stem == cislo_safe || stem.ends_with(&format!("__{cislo_safe}"))
}

/// The stored file for a číslo listiny, when one is already there - matched by
/// its číslo segment, so an already downloaded document is found before any
/// network call whatever human label its name carries.
fn existing(ico8: &str, cislo_safe: &str) -> Option<PathBuf> {
    fs::read_dir(subject_dir(ico8))
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| name_matches_cislo(n, cislo_safe))
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
    /// The official download URL - known only when we actually resolved (i.e. not
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
    let stem = parse::doc_file_stem(&fetched.listina.typ, &fetched.listina.cislo);
    let path = subject_dir(ico8).join(format!(
        "{stem}.{}",
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

/// Temp file in the destination dir, then rename - a crash leaves either nothing
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
    fn name_matches_cislo_keys_on_the_cislo_segment() {
        let cislo = "B-8573-SL95-MSPH";
        // new descriptive name, empty-typ fallback, and a different extension all match
        assert!(name_matches_cislo(
            "ucetni-zaverka-2024__B-8573-SL95-MSPH.pdf",
            cislo
        ));
        assert!(name_matches_cislo("B-8573-SL95-MSPH.pdf", cislo));
        assert!(name_matches_cislo("stanovy__B-8573-SL95-MSPH.xhtml", cislo));
        // a different číslo must not match
        assert!(!name_matches_cislo(
            "ucetni-zaverka-2024__B-8573-SL12-MSPH.pdf",
            cislo
        ));
        // a longer number that merely shares a tail must not match on a partial segment
        assert!(!name_matches_cislo("B-8573-SL95-MSPH-EXTRA.pdf", cislo));
        // a hidden / .tmp. leftover is ignored
        assert!(!name_matches_cislo(
            ".ucetni-zaverka-2024__B-8573-SL95-MSPH.pdf.tmp.42",
            cislo
        ));
    }

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

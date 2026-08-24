//! Putting a downloaded attachment on disk under the name Datify gave it.
//!
//! The caller names a directory, never a file: the agent driving this CLI has
//! not seen the bytes, so a name it invents carries a guessed extension, and
//! everything downstream (`ocr` above all) picks its reader from that extension.
//! Datify sends the real name in `Content-Disposition`, so that is what lands on
//! disk. Renaming afterwards is `mv`, which the shell does better than a flag.
//!
//! Kept apart from `client.rs`, which owns HTTP and nothing else - the name
//! parsing and collision rules here are pure and testable without a socket.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::client::AttachmentContent;
use crate::error::{DatifyError, Result};

/// Save `content` into `dir`, reporting where it landed.
///
/// `attachment_id` is the fallback stem for the rare reply that carries no
/// usable name; an id is always available and never collides.
pub fn save(content: &AttachmentContent, dir: &Path, attachment_id: &str) -> Result<Value> {
    if dir.exists() && !dir.is_dir() {
        return Err(DatifyError::Config(format!(
            "--output-dir musí být adresář, {} je soubor.",
            dir.display()
        )));
    }
    std::fs::create_dir_all(dir)?;

    let filename = content
        .disposition
        .as_deref()
        .and_then(filename_from_disposition)
        .unwrap_or_else(|| fallback_name(attachment_id, content.content_type.as_deref()));

    let path = unique_path(dir, &filename);
    std::fs::write(&path, &content.bytes)?;

    Ok(json!({
        "path": path.display().to_string(),
        "filename": path.file_name().and_then(|n| n.to_str()).unwrap_or(&filename),
        "contentType": content.content_type,
        "size": content.bytes.len(),
    }))
}

/// The file name out of a `Content-Disposition` value.
///
/// Datify sends both forms - `filename="AAA 1234 foto1.png"` and the RFC 5987
/// `filename*=UTF-8''AAA%201234%20foto1.png`. RFC 6266 gives `filename*`
/// precedence, and it is the one that survives non-ASCII, so it is tried first.
fn filename_from_disposition(value: &str) -> Option<String> {
    for part in value.split(';').map(str::trim) {
        if let Some(raw) = part.strip_prefix("filename*=") {
            if let Some(name) = decode_ext_value(raw).as_deref().and_then(sanitize) {
                return Some(name);
            }
        }
    }
    for part in value.split(';').map(str::trim) {
        if let Some(raw) = part.strip_prefix("filename=") {
            if let Some(name) = sanitize(raw.trim_matches('"')) {
                return Some(name);
            }
        }
    }
    None
}

/// `UTF-8''na%C4%8Dten%C3%AD.pdf` → `načtení.pdf`. Only the charset-prefixed
/// form is accepted; anything else is left for the plain `filename=` to answer.
fn decode_ext_value(raw: &str) -> Option<String> {
    let encoded = raw.rsplit_once('\'').map(|(_, rest)| rest)?;
    let bytes = encoded.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// A name safe to join onto the target directory.
///
/// Only the last path component survives, so a reply naming
/// `../../.ssh/authorized_keys` writes into the directory the caller asked for
/// and nowhere else. `.` and `..` resolve to nothing and are refused.
fn sanitize(name: &str) -> Option<String> {
    let last = name
        .rsplit(['/', '\\'])
        .next()
        .map(str::trim)
        .unwrap_or_default();
    if last.is_empty() || last == "." || last == ".." {
        return None;
    }
    Some(last.to_string())
}

/// `<id>.<ext>` for a reply that named nothing, the extension guessed from the
/// media type - the one place a name is invented rather than taken.
fn fallback_name(attachment_id: &str, content_type: Option<&str>) -> String {
    let extension = match content_type.unwrap_or_default().split(';').next() {
        Some("application/pdf") => "pdf",
        Some("image/png") => "png",
        Some("image/jpeg") => "jpg",
        Some("text/plain") => "txt",
        Some("application/json") => "json",
        _ => "bin",
    };
    format!("{attachment_id}.{extension}")
}

/// `dir/name`, or `dir/name (2).ext` when that is taken.
///
/// Two different attachments do share a name in practice, so overwriting would
/// silently drop one of them - and the name came from Datify, not from the
/// caller, who therefore never chose to replace anything.
fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let (stem, extension) = match filename.rsplit_once('.') {
        // A leading dot is the whole name of a hidden file, not an extension.
        Some((stem, extension)) if !stem.is_empty() => (stem, format!(".{extension}")),
        _ => (filename, String::new()),
    };
    // Bounded so a directory that cannot be written to fails on the write
    // rather than spinning here forever.
    for n in 2..10_000 {
        let candidate = dir.join(format!("{stem} ({n}){extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content(bytes: &[u8], disposition: Option<&str>, ct: Option<&str>) -> AttachmentContent {
        AttachmentContent {
            bytes: bytes.to_vec(),
            disposition: disposition.map(str::to_string),
            content_type: ct.map(str::to_string),
        }
    }

    // ── the name out of Content-Disposition ──────────────────────────────────

    #[test]
    fn the_plain_filename_parameter_is_read() {
        assert_eq!(
            filename_from_disposition(r#"attachment; filename="smlouva.pdf""#).as_deref(),
            Some("smlouva.pdf")
        );
    }

    /// The header Datify actually sends carries both forms at once.
    #[test]
    fn the_rfc_5987_form_wins_over_the_plain_one() {
        // Only `filename*` survives non-ASCII, so it has to be preferred even
        // though the plain parameter comes first in the header.
        let header = "attachment; filename=\"nacteni.pdf\"; filename*=UTF-8''na%C4%8Dten%C3%AD.pdf";
        assert_eq!(
            filename_from_disposition(header).as_deref(),
            Some("načtení.pdf")
        );
    }

    #[test]
    fn a_percent_escaped_space_survives_the_round_trip() {
        let header = "attachment; filename*=UTF-8''AAA%201234%20foto1.png";
        assert_eq!(
            filename_from_disposition(header).as_deref(),
            Some("AAA 1234 foto1.png")
        );
    }

    #[test]
    fn a_header_naming_nothing_yields_no_name() {
        assert_eq!(filename_from_disposition("attachment"), None);
        assert_eq!(
            filename_from_disposition(r#"attachment; filename="""#),
            None
        );
    }

    /// A name is joined onto a directory the caller chose, so a path in it must
    /// never reach outside that directory.
    #[test]
    fn a_traversing_name_is_reduced_to_its_last_component() {
        for header in [
            r#"attachment; filename="../../.ssh/authorized_keys""#,
            "attachment; filename*=UTF-8''..%2F..%2F.ssh%2Fauthorized_keys",
        ] {
            assert_eq!(
                filename_from_disposition(header).as_deref(),
                Some("authorized_keys"),
                "{header}"
            );
        }
    }

    #[test]
    fn a_name_that_is_only_dots_is_refused() {
        assert_eq!(
            filename_from_disposition(r#"attachment; filename="..""#),
            None
        );
        assert_eq!(
            filename_from_disposition(r#"attachment; filename=".""#),
            None
        );
    }

    // ── collisions ───────────────────────────────────────────────────────────

    #[test]
    fn a_free_name_is_used_as_it_is() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(unique_path(dir.path(), "a.pdf"), dir.path().join("a.pdf"));
    }

    #[test]
    fn a_taken_name_gets_a_counter_before_its_extension() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.pdf"), b"x").unwrap();
        assert_eq!(
            unique_path(dir.path(), "a.pdf"),
            dir.path().join("a (2).pdf")
        );
        std::fs::write(dir.path().join("a (2).pdf"), b"x").unwrap();
        assert_eq!(
            unique_path(dir.path(), "a.pdf"),
            dir.path().join("a (3).pdf")
        );
    }

    #[test]
    fn a_taken_name_without_an_extension_still_gets_a_counter() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("README"), b"x").unwrap();
        assert_eq!(
            unique_path(dir.path(), "README"),
            dir.path().join("README (2)")
        );
    }

    /// `.gitignore` is a name, not an extension - splitting on the leading dot
    /// would produce the nonsense `" (2).gitignore"`.
    #[test]
    fn a_dotfile_keeps_its_whole_name() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), b"x").unwrap();
        assert_eq!(
            unique_path(dir.path(), ".gitignore"),
            dir.path().join(".gitignore (2)")
        );
    }

    // ── saving ───────────────────────────────────────────────────────────────

    #[test]
    fn the_bytes_land_under_the_name_the_server_gave() {
        let dir = tempfile::tempdir().unwrap();
        let content = content(
            b"\x00PDF-bytes\xff",
            Some(r#"attachment; filename="smlouva.pdf""#),
            Some("application/pdf"),
        );

        let report = save(&content, dir.path(), "a1").unwrap();

        assert_eq!(report["filename"], "smlouva.pdf");
        assert_eq!(report["contentType"], "application/pdf");
        assert_eq!(report["size"], 11);
        let path = report["path"].as_str().unwrap();
        // Byte-for-byte: a text round trip would corrupt a real attachment.
        assert_eq!(std::fs::read(path).unwrap(), b"\x00PDF-bytes\xff");
    }

    #[test]
    fn a_second_attachment_of_the_same_name_does_not_replace_the_first() {
        let dir = tempfile::tempdir().unwrap();
        let disposition = Some(r#"attachment; filename="foto.png""#);
        save(&content(b"first", disposition, None), dir.path(), "a1").unwrap();
        let report = save(&content(b"second", disposition, None), dir.path(), "a2").unwrap();

        assert_eq!(report["filename"], "foto (2).png");
        assert_eq!(
            std::fs::read(dir.path().join("foto.png")).unwrap(),
            b"first"
        );
    }

    #[test]
    fn a_reply_naming_nothing_falls_back_to_the_id_and_the_media_type() {
        let dir = tempfile::tempdir().unwrap();
        let report = save(&content(b"x", None, Some("image/png")), dir.path(), "a1").unwrap();
        assert_eq!(report["filename"], "a1.png");
    }

    #[test]
    fn an_unknown_media_type_falls_back_to_bin() {
        let dir = tempfile::tempdir().unwrap();
        let report = save(
            &content(b"x", None, Some("application/x-weird")),
            dir.path(),
            "a1",
        )
        .unwrap();
        assert_eq!(report["filename"], "a1.bin");
    }

    #[test]
    fn a_missing_directory_is_created() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b/c");
        let report = save(
            &content(b"x", Some(r#"attachment; filename="f.txt""#), None),
            &nested,
            "a1",
        )
        .unwrap();
        assert_eq!(report["path"], nested.join("f.txt").display().to_string());
    }

    /// `--output-dir` naming an existing *file* is a caller mistake worth
    /// reporting as one, rather than an io error about a directory.
    #[test]
    fn a_target_that_is_a_file_is_refused_by_name() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("not-a-dir");
        std::fs::write(&file, b"x").unwrap();

        let err = save(&content(b"x", None, None), &file, "a1").unwrap_err();
        assert!(err.to_string().contains("musí být adresář"), "{err}");
    }
}

//! Atomic filesystem writes usable by any plugin (CLI or hosted component).
//!
//! Write to a temp file next to the target, fsync, then rename into place.
//! POSIX `rename(2)` is atomic within a filesystem, so a reader never observes a
//! torn file. Returns plain `io::Result` - decoupled from any plugin error enum;
//! a JSON serialization failure surfaces as `io::ErrorKind::InvalidData`.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

/// Monotonic counter for temp-file names - avoids a rand dependency.
static NEXT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A process-unique token (pid + monotonic counter) for temp / trash file names,
/// avoiding a rand dependency. Unique within a process run; not globally unique
/// across restarts, so callers probe with an `exists()` loop (as `atomic_replace`
/// and the item store's trash-rename do).
pub fn unique_token() -> String {
    format!(
        "{}.{}",
        std::process::id(),
        NEXT_COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

/// Atomically replace the file at `path` with whatever `writer` emits.
/// `mode` (if set) applies before the rename - use `0o600` for secrets.
pub fn atomic_replace<F>(path: &Path, mode: Option<u32>, writer: F) -> io::Result<()>
where
    F: FnOnce(&mut File) -> io::Result<()>,
{
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    if let Some(parent) = parent {
        fs::create_dir_all(parent)?;
    }
    let parent_dir = parent.unwrap_or_else(|| Path::new("."));

    let base_name = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "target path has no filename")
    })?;
    let tmp_path = loop {
        let candidate = parent_dir.join(format!(".{base_name}.{}.tmp", unique_token()));
        if !candidate.exists() {
            break candidate;
        }
    };

    let mut open_opts = OpenOptions::new();
    open_opts.write(true).create_new(true);
    if let Some(m) = mode {
        open_opts.mode(m);
    }

    let result = (|| -> io::Result<()> {
        let mut file = open_opts.open(&tmp_path)?;
        writer(&mut file)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);

        if let Some(m) = mode {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp_path, fs::Permissions::from_mode(m))?;
        }

        fs::rename(&tmp_path, path)?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

/// Atomically write `data` as pretty JSON with a trailing newline - byte-for-byte
/// what the Python `atomic_write_json` produced, so state files don't churn.
pub fn atomic_write_json<T: Serialize>(path: &Path, data: &T) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    atomic_replace(path, None, |file| {
        file.write_all(&bytes)?;
        file.write_all(b"\n")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn atomic_replace_writes_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.txt");
        atomic_replace(&path, None, |f| f.write_all(b"hello")).unwrap();
        let mut buf = String::new();
        File::open(&path).unwrap().read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "hello");
    }

    #[test]
    fn atomic_replace_applies_mode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret");
        atomic_replace(&path, Some(0o600), |f| f.write_all(b"sensitive")).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn atomic_replace_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a/b/c/file.txt");
        atomic_replace(&path, None, |f| f.write_all(b"x")).unwrap();
        assert!(path.is_file());
    }

    #[test]
    fn atomic_replace_cleans_up_on_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.txt");
        let err = atomic_replace(&path, None, |_| Err(io::Error::other("boom")));
        assert!(err.is_err());
        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert!(
            leftovers.is_empty(),
            "expected empty dir, got: {leftovers:?}"
        );
    }

    #[test]
    fn atomic_write_json_writes_pretty_with_newline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.json");
        #[derive(Serialize)]
        struct Payload {
            name: String,
            count: u32,
        }
        atomic_write_json(
            &path,
            &Payload {
                name: "test".into(),
                count: 3,
            },
        )
        .unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"name\": \"test\""));
        assert!(body.contains("\"count\": 3"));
        assert!(body.ends_with('\n'));
    }
}

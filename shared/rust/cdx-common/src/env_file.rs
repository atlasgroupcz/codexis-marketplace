//! Tiny `KEY=VALUE` credential-file helpers shared by the watcher plugins.
//!
//! Each plugin that needs a user-provided API credential stores it in
//! `<user_home>/.cdx/apps/<app>/.env` (mode 0600). This module ports the
//! byte-identical mechanics of the Python cores' `settings.py` — reading the
//! pairs and writing the file atomically. What the keys mean, how they are
//! validated and masked stays in each plugin.

use std::io::Write;
use std::path::Path;

/// Parses an env file into `(key, value)` pairs, in file order. Blank lines,
/// `#` comments and lines without `=` are skipped; keys and values are trimmed.
/// A missing or unreadable file yields an empty list.
pub fn read_env_file(path: &Path) -> Vec<(String, String)> {
    let Ok(contents) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

/// Reads one key from an env file. `None` when the file or key is missing, or
/// the value is empty.
pub fn read_env_var(path: &Path, key: &str) -> Option<String> {
    read_env_file(path)
        .into_iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .filter(|v| !v.is_empty())
}

/// Atomically writes the whole env file (mode 0600 — these are credentials).
pub fn write_env_file(path: &Path, pairs: &[(&str, &str)]) -> std::io::Result<()> {
    crate::fs::atomic_replace(path, Some(0o600), |file| {
        for (key, value) in pairs {
            writeln!(file, "{key}={value}")?;
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn write_then_read_roundtrips_with_0600() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        write_env_file(&path, &[("API_KEY", "abc123"), ("OTHER", "x y")]).unwrap();

        assert_eq!(read_env_var(&path, "API_KEY").as_deref(), Some("abc123"));
        assert_eq!(read_env_var(&path, "OTHER").as_deref(), Some("x y"));
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn read_skips_comments_blanks_and_trims() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(&path, "# comment\n\n KEY = value \nnoequals\nB=2\n").unwrap();

        assert_eq!(
            read_env_file(&path),
            vec![
                ("KEY".to_string(), "value".to_string()),
                ("B".to_string(), "2".to_string()),
            ]
        );
    }

    #[test]
    fn missing_file_and_empty_value_yield_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nope.env");
        assert!(read_env_file(&missing).is_empty());
        assert_eq!(read_env_var(&missing, "K"), None);

        let path = dir.path().join(".env");
        std::fs::write(&path, "K=\n").unwrap();
        assert_eq!(read_env_var(&path, "K"), None);
    }
}

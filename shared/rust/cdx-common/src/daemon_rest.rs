//! Auth helper for plugins that call daemon REST endpoints directly (OCR, media
//! upload) rather than publishing sources. std-only.
//!
//! This is a *different* concern from [`crate::config::DaemonConfig`]: there is
//! no chat/tool-call context, and the token is returned as a bare
//! `Bearer <token>` value (the caller adds the `Authorization:` header itself).

use std::env;
use std::fs;
use std::path::Path;

const SECRET_FILE_RELATIVE: &str = ".cdx/env/secret";

/// Reads the VM daemon token from `~/.cdx/env/secret` and returns it as a
/// `Bearer <token>` value (no `Authorization:` prefix). Returns `None` when the
/// secret file is missing or empty.
///
/// This is the VM token (the same one `cdxctl` reads), NOT
/// `CODEXIS_USER_API_TOKEN` — the gateway rejects the latter for these
/// endpoints.
pub fn daemon_bearer() -> Option<String> {
    let secret = read_secret_file()?;
    Some(ensure_bearer_prefix(&secret))
}

fn read_secret_file() -> Option<String> {
    let home = env::var("HOME").ok()?;
    let contents = fs::read_to_string(Path::new(&home).join(SECRET_FILE_RELATIVE)).ok()?;
    let trimmed = contents.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn ensure_bearer_prefix(value: &str) -> String {
    let mut s = value.trim();
    if let Some(stripped) = s.strip_prefix("Authorization:") {
        s = stripped.trim();
    }
    if s.len() >= 7 && s.as_bytes()[..7].eq_ignore_ascii_case(b"Bearer ") {
        s.to_string()
    } else {
        format!("Bearer {}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_prefix_added_to_raw_token() {
        assert_eq!(ensure_bearer_prefix("abc.def.ghi"), "Bearer abc.def.ghi");
    }

    #[test]
    fn bearer_prefix_preserved_and_authorization_stripped() {
        assert_eq!(ensure_bearer_prefix("Bearer xyz"), "Bearer xyz");
        assert_eq!(
            ensure_bearer_prefix("Authorization: Bearer xyz"),
            "Bearer xyz"
        );
    }
}

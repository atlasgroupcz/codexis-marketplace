//! Auth helper for plugins that call daemon REST endpoints directly (media
//! upload) rather than publishing sources. std-only.
//!
//! This is a *different* concern from [`crate::config::DaemonConfig`]: there is
//! no chat/tool-call context, and the token is returned as a bare
//! `Bearer <token>` value (the caller adds the `Authorization:` header itself).

/// Reads the VM daemon token from `~/.cdx/env/secret` (via the shared
/// [`crate::config::read_daemon_secret`] home-chain lookup) and returns it as a
/// `Bearer <token>` value (no `Authorization:` prefix). Returns `None` when the
/// secret file is missing or empty.
///
/// This is the VM token (the same one `cdxctl` reads), NOT
/// `CODEXIS_USER_API_TOKEN` - the gateway rejects the latter for these
/// endpoints.
pub fn daemon_bearer() -> Option<String> {
    let secret = crate::config::read_daemon_secret()?;
    Some(ensure_bearer_prefix(&secret))
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
    fn daemon_bearer_reads_secret_from_real_home_even_when_public_home_differs() {
        let public_home = tempfile::tempdir().unwrap();
        let real_home = tempfile::tempdir().unwrap();
        let _g = crate::test_support::home_guard(public_home.path());
        // The host writes the secret to the real VM home; the daemon-injected
        // CODEXIS_PUBLIC_USER_HOME points at per-user app state elsewhere.
        std::env::set_var("HOME", real_home.path());
        let secret_dir = real_home.path().join(".cdx/env");
        std::fs::create_dir_all(&secret_dir).unwrap();
        std::fs::write(secret_dir.join("secret"), "tok.en.x\n").unwrap();

        assert_eq!(daemon_bearer().as_deref(), Some("Bearer tok.en.x"));
    }

    #[test]
    fn daemon_bearer_ignores_secret_under_public_user_home() {
        let public_home = tempfile::tempdir().unwrap();
        let real_home = tempfile::tempdir().unwrap();
        let _g = crate::test_support::home_guard(public_home.path());
        std::env::set_var("HOME", real_home.path());
        // A secret that exists only under CODEXIS_PUBLIC_USER_HOME is not the
        // host-provisioned VM token and must not be picked up.
        let secret_dir = public_home.path().join(".cdx/env");
        std::fs::create_dir_all(&secret_dir).unwrap();
        std::fs::write(secret_dir.join("secret"), "wrong.token\n").unwrap();

        assert_eq!(daemon_bearer(), None);
    }

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

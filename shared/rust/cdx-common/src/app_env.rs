//! Store a plugin's secret through `cdxctl appenv`. A plugin runs inside the
//! isolation, where `~/.cdx/apps/<app>/.env` is read-only; `cdxctl` asks the
//! daemon to rewrite the file from outside. The plugin keeps reading the file
//! through [`crate::env_file`] as before.

use std::fmt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// `cdxctl` exited nonzero or could not be started.
#[derive(Debug)]
pub struct AppEnvError {
    /// The exit code of `cdxctl`, `None` when it could not be spawned.
    pub exit_code: Option<i32>,
    pub message: String,
}

impl fmt::Display for AppEnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.exit_code {
            Some(code) => write!(f, "cdxctl appenv exited with {code}: {}", self.message),
            None => write!(f, "cdxctl could not be started: {}", self.message),
        }
    }
}

impl std::error::Error for AppEnvError {}

/// `cdxctl appenv <app> set <name> <value>`.
pub fn set(app: &str, name: &str, value: &str) -> Result<(), AppEnvError> {
    run(&["appenv", app, "set", name, value])
}

/// `cdxctl appenv <app> delete <name>`.
pub fn delete(app: &str, name: &str) -> Result<(), AppEnvError> {
    run(&["appenv", app, "delete", name])
}

fn run(args: &[&str]) -> Result<(), AppEnvError> {
    let output = Command::new(cdxctl_program())
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| AppEnvError {
            exit_code: None,
            message: e.to_string(),
        })?;
    if output.status.success() {
        return Ok(());
    }
    Err(AppEnvError {
        exit_code: output.status.code(),
        message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

/// `cdxctl` from `PATH`, else the install location the codexis plugin uses.
fn cdxctl_program() -> PathBuf {
    let on_path = std::env::var_os("PATH")
        .map(|path| {
            std::env::split_paths(&path)
                .map(|dir| dir.join("cdxctl"))
                .any(|candidate| crate::paths::is_regular_file(&candidate))
        })
        .unwrap_or(false);
    if on_path {
        return PathBuf::from("cdxctl");
    }
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".local/bin/cdxctl"))
        .unwrap_or_else(|_| PathBuf::from("cdxctl"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{fake_cdxctl, home_guard};

    #[test]
    fn set_runs_cdxctl_appenv_set_with_all_arguments() {
        let tmp = tempfile::tempdir().unwrap();
        let home = home_guard(tmp.path());
        let fake = fake_cdxctl(&home, tmp.path());

        set("katastr", "API_KEY", "s3cr3t").unwrap();

        assert_eq!(fake.calls(), vec!["appenv katastr set API_KEY s3cr3t"]);
    }

    #[test]
    fn delete_runs_cdxctl_appenv_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let home = home_guard(tmp.path());
        let fake = fake_cdxctl(&home, tmp.path());

        delete("katastr", "API_KEY").unwrap();

        assert_eq!(fake.calls(), vec!["appenv katastr delete API_KEY"]);
    }

    #[test]
    fn nonzero_exit_becomes_an_error_with_the_code_and_stderr() {
        let tmp = tempfile::tempdir().unwrap();
        let home = home_guard(tmp.path());
        let fake = fake_cdxctl(&home, tmp.path());
        fake.set_exit(7);

        let error = set("katastr", "API_KEY", "x").unwrap_err();

        assert_eq!(error.exit_code, Some(7));
        assert!(error.message.contains("fake failure"), "{}", error.message);
        assert!(error.to_string().starts_with("cdxctl appenv exited with 7"));
    }

    #[test]
    fn missing_cdxctl_is_reported_without_an_exit_code() {
        let tmp = tempfile::tempdir().unwrap();
        let _home = home_guard(tmp.path());
        let previous_path = std::env::var_os("PATH");
        std::env::set_var("PATH", tmp.path());

        let error = delete("katastr", "API_KEY").unwrap_err();

        match previous_path {
            Some(path) => std::env::set_var("PATH", path),
            None => std::env::remove_var("PATH"),
        }
        assert_eq!(error.exit_code, None);
        assert!(error.to_string().starts_with("cdxctl could not be started"));
    }
}

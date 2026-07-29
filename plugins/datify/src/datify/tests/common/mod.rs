//! Shared rig for the binary tests: a stub Datify, and a sandboxed environment
//! to run the binaries in.
//!
//! Both binaries take their endpoint and token from the environment, so they can
//! be exercised as processes without mocking anything - the request really goes
//! over a socket, and the settings file really lands on disk.
//!
//! Cargo compiles this module separately into each test binary, so whatever one
//! of them does not use reads as dead code there - hence the crate-wide allow,
//! which is the usual price of sharing a test helper.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub use datify_core::test_support::stub;

/// A scratch home, so the settings file a test writes cannot reach the real one.
pub struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    pub fn home(&self) -> &Path {
        self.dir.path()
    }

    /// Where `settings set-key` stores the token.
    pub fn env_file(&self) -> PathBuf {
        self.dir.path().join(".cdx/apps/datify/.env")
    }

    /// Run a binary with this sandbox as its home. `api_url` and `token`, when
    /// given, are what the binary will talk to and with.
    pub fn run(
        &self,
        binary: &str,
        args: &[&str],
        api_url: Option<&str>,
        token: Option<&str>,
    ) -> Output {
        let mut command = sandboxed_command(binary, self.home());
        command.args(args);
        if let Some(api_url) = api_url {
            command.env("DATIFY_API_URL", api_url);
        }
        if let Some(token) = token {
            command.env("DATIFY_API_KEY", token);
        }
        command.output().unwrap()
    }

    /// Same, with `stdin` piped in - how the daemon hands a CGI its body, and
    /// how `--data -` reads one.
    pub fn run_with_stdin(
        &self,
        binary: &str,
        args: &[&str],
        api_url: Option<&str>,
        token: Option<&str>,
        env: &[(&str, String)],
        stdin: &str,
    ) -> Output {
        use std::io::Write as _;
        let mut command = sandboxed_command(binary, self.home());
        command
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());
        for (name, value) in env {
            command.env(name, value);
        }
        if let Some(api_url) = api_url {
            command.env("DATIFY_API_URL", api_url);
        }
        if let Some(token) = token {
            command.env("DATIFY_API_KEY", token);
        }
        let mut child = command.spawn().unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

/// Start from a bare environment, keeping only what the child genuinely needs.
///
/// `env_clear` is the point - a stray `DATIFY_API_KEY` on the developer's shell
/// would otherwise decide the outcome of a test. But it also drops the variable
/// `cargo llvm-cov` uses to collect a child process's counters, and without that
/// the binaries read as 0% covered however hard these tests drive them.
pub fn sandboxed_command(binary: &str, home: &Path) -> Command {
    let mut command = Command::new(binary);
    command
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", home)
        .env("CODEXIS_PUBLIC_USER_HOME", home);
    for var in ["LLVM_PROFILE_FILE", "CARGO_LLVM_COV", "LD_LIBRARY_PATH"] {
        if let Ok(value) = std::env::var(var) {
            command.env(var, value);
        }
    }
    command
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

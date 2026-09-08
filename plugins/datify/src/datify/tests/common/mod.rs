//! Shared rig for the binary tests: a stub Datify, and a sandboxed environment
//! to run the binaries in.
//!
//! Both binaries take their endpoint and token from the environment, so they can
//! be exercised as processes without mocking anything - the request really goes
//! over a socket, and the settings file really lands on disk.
//!
//! Cargo compiles this module separately into each test binary, so whatever one
//! of them does not use reads as dead code there - hence the crate-wide allow,
//! which is the usual price of sharing a test helper. Re-exports need the same
//! treatment: only the CLI tests download a file, so only they need `stub_raw`.
#![allow(dead_code, unused_imports)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub use datify_core::test_support::{stub, stub_raw};

/// A scratch home, so the settings file a test writes cannot reach the real one.
///
/// It also carries a fake `cdxctl` on the child's `PATH`: the binaries no longer
/// write the credential file themselves, they ask the platform to, so what a
/// test can observe is the command line they asked with.
pub struct Sandbox {
    dir: tempfile::TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        let sandbox = Self {
            dir: tempfile::tempdir().unwrap(),
        };
        sandbox.install_fake_cdxctl();
        sandbox
    }

    pub fn home(&self) -> &Path {
        self.dir.path()
    }

    /// Where the daemon stores the token, and where the binaries read it from.
    pub fn env_file(&self) -> PathBuf {
        self.dir.path().join(".cdx/apps/datify/.env")
    }

    /// Seed the credential file the way the daemon would, so the read side has
    /// something to report.
    pub fn store_key(&self, key: &str) {
        let path = self.env_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, format!("DATIFY_API_KEY={key}\n")).unwrap();
    }

    /// One line per `cdxctl` invocation: the argv joined by spaces.
    pub fn cdxctl_calls(&self) -> Vec<String> {
        std::fs::read_to_string(self.dir.path().join("bin/cdxctl-calls.log"))
            .map(|s| s.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    /// Make the next `cdxctl` run fail with `code`, the way a daemon that
    /// refused the write would.
    pub fn fail_cdxctl(&self, code: i32) {
        std::fs::write(self.dir.path().join("bin/cdxctl-exit"), code.to_string()).unwrap();
    }

    /// The child's `PATH`, with the fake `cdxctl` ahead of anything real.
    fn path_with_fake_cdxctl(&self) -> std::ffi::OsString {
        let mut dirs = vec![self.bin_dir()];
        if let Ok(path) = std::env::var("PATH") {
            dirs.extend(std::env::split_paths(&path));
        }
        std::env::join_paths(dirs).unwrap()
    }

    fn bin_dir(&self) -> PathBuf {
        self.dir.path().join("bin")
    }

    fn install_fake_cdxctl(&self) {
        use std::os::unix::fs::PermissionsExt;
        let bin = self.bin_dir();
        std::fs::create_dir_all(&bin).unwrap();
        let script = bin.join("cdxctl");
        std::fs::write(
            &script,
            "#!/bin/sh\nhere=\"$(dirname \"$0\")\"\nprintf '%s\\n' \"$*\" >> \"$here/cdxctl-calls.log\"\ncode=$(cat \"$here/cdxctl-exit\" 2>/dev/null || echo 0)\n[ \"$code\" -eq 0 ] || echo \"fake failure\" >&2\nexit \"$code\"\n",
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
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
        command.env("PATH", self.path_with_fake_cdxctl()).args(args);
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
            .env("PATH", self.path_with_fake_cdxctl())
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

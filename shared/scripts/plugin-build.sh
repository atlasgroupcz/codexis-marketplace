#!/usr/bin/env bash
# Shared build logic for the marketplace Rust plugins. Sourced by each plugin's
# `src/build.sh` (and by the repo-root build.sh). Build-time only - never shipped
# (`shared/` is dist-excluded).
#
# All Rust crates live in ONE Cargo workspace (root Cargo.toml). The root
# build.sh compiles every crate ONCE with a single `cargo build --workspace`
# (all shared deps compiled once, not per plugin) - see build_rust_workspace in
# build.sh. Each plugin's src/build.sh then just installs its already-built
# binary from the shared target/ via `build_plugin_crate` /
# `install_workspace_binary` - no per-plugin compile. The produced binaries are
# unchanged: static, self-contained.
#
# This file holds the shared build CONFIG (pinned image, arch, target dir) and
# the per-plugin INSTALL helpers. The one-time whole-workspace compile lives in
# the root build.sh orchestrator (which sources this file for the config).
#
# Usage from a plugin's src/build.sh (unchanged for the simple single-bin plugins):
#   set -euo pipefail
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"
#   build_plugin_crate "$SCRIPT_DIR/<crate-dir>" "<bin-name>"
#
# Runs on any host with docker - the toolchain lives in the image, not on the
# machine.
#
# Override before `source` when needed:
#   RUST_IMAGE           Docker image (default: pinned rust:1.86 slim, by digest)
#   CARGO_BUILD_ARGS     cargo flags (default: --release --locked)
#   CDX_DOCKER_PLATFORM  build platform (default: linux/amd64, what the guest runs)

set -euo pipefail

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker is required to build the plugin binaries." >&2
  exit 1
fi

# Repo root, derived from this file's own location (shared/scripts/ -> ../..).
_CDX_BUILD_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$_CDX_BUILD_DIR/../.." && pwd)"

RUST_IMAGE="${RUST_IMAGE:-rust:1.86.0-slim-bookworm@sha256:57d415bbd61ce11e2d5f73de068103c7bd9f3188dc132c97cef4a8f62989e944}"
CARGO_BUILD_ARGS="${CARGO_BUILD_ARGS:---release --locked}"

# The target platform is a property of where the binaries RUN (the guest VM,
# x86-64), not of the machine that happens to compile them. It used to be
# derived from `uname -m`, which meant an arm64 host silently produced arm64
# binaries: the build succeeded, the artifacts were committed like any other,
# and the failure only surfaced when the daemon tried to run them. Pin it.
# On an arm64 host this emulates, which is slow but correct - and the reason
# the release build belongs in CI rather than on a laptop.
_CDX_DOCKER_PLATFORM="${CDX_DOCKER_PLATFORM:-linux/amd64}"

# Install one already-built binary from the shared workspace target/ into a
# destination dir (Linux).
#   $1  binary name (as it appears in target/release/)
#   $2  destination directory
install_workspace_binary() {
  local bin_name="$1"
  local dst_dir="$2"
  mkdir -p "$dst_dir"

  local lin="$REPO_ROOT/target/release/$bin_name"
  if [[ ! -f "$lin" ]]; then
    echo "ERROR: $bin_name not found in workspace target ($lin)." >&2
    echo "       Did build_rust_workspace run first (root build.sh)?" >&2
    exit 1
  fi
  install -m 0755 "$lin" "$dst_dir/$bin_name"
  echo "   installed -> $dst_dir/$bin_name"
}

_cdx_bin_dir() {
  # crate_dir = <plugin>/src/<crate>  ->  plugin bin dir = <plugin>/bin
  local crate_dir="$1"
  local plugin_dir
  plugin_dir="$(cd "$crate_dir/../.." && pwd)"
  echo "$plugin_dir/bin"
}

# --- standalone fallback ------------------------------------------------------
# In the release pipeline the root build.sh runs the one-pass workspace build
# first, so every binary is already in target/ and the plugins just install them
# (the fallback below never fires). But a plugin's src/build.sh can also be run
# on its own (no prior workspace build) - then we build just that crate here, the
# same release artifact the workspace build would produce, before installing.

# Build one crate's release binaries (all its [[bin]]) for linux (Docker).
# The whole repo is mounted so the shared Cargo.lock + path deps resolve; `-p`
# (rather than --manifest-path) keeps cargo resolving the whole workspace, so
# feature unification - and therefore the reused artifacts - match the one-pass
# `cargo build --workspace`.
_build_one_crate() {
  local crate_dir="$1"
  local crate_rel="${crate_dir#$REPO_ROOT/}"
  local crate_name
  crate_name="$(basename "$crate_dir")"

  echo "==> Building $crate_rel for $_CDX_DOCKER_PLATFORM (Docker)"
  docker run --rm \
    --platform "$_CDX_DOCKER_PLATFORM" \
    -u "$(id -u):$(id -g)" \
    -e CARGO_HOME=/tmp/cargo \
    -v "$REPO_ROOT:/workspace" \
    -w /workspace \
    "$RUST_IMAGE" \
    cargo build -p "$crate_name" $CARGO_BUILD_ARGS
}

# Compile the crate unless the one-pass workspace build already ran in THIS
# invocation - the root build.sh exports CDX_WORKSPACE_BUILT before calling the
# plugin scripts.
#
# Two things this must NOT do:
#   - skip because the binary exists in target/: a leftover binary is not
#     necessarily current, so a per-plugin build.sh run after editing a crate
#     silently installed a stale artifact.
#   - rebuild per crate after the workspace pass: measured at ~1 min per crate,
#     because a single-crate build resolves features differently than
#     `--workspace` and so cannot reuse its artifacts. That is the cost the
#     one-pass build exists to avoid.
_ensure_crate_built() {
  local crate_dir="$1"
  [[ -n "${CDX_WORKSPACE_BUILT:-}" ]] || _build_one_crate "$crate_dir"
}

# Entry point for the simple single-binary plugins. Installs the binary from the
# shared target/ into the plugin's bin/, compiling the crate first unless the
# workspace pass already did. Signature unchanged, so those plugins' src/build.sh
# need no edits.
#   $1  crate dir (…/plugins/<p>/src/<crate>)   $2  binary name
build_plugin_crate() {
  local crate_dir="$1"
  local bin_name="$2"
  _ensure_crate_built "$crate_dir"
  install_workspace_binary "$bin_name" "$(_cdx_bin_dir "$crate_dir")"
}

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
# Override before `source` when needed:
#   RUST_IMAGE        Docker image (default: pinned rust:1.86 slim, by digest)
#   CARGO_BUILD_ARGS  cargo flags (default: --release --locked)

set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "ERROR: Linux-only build script." >&2
  exit 1
fi

case "$(uname -m)" in
  x86_64|amd64) _CDX_DOCKER_ARCH="amd64" ;;
  aarch64|arm64) _CDX_DOCKER_ARCH="arm64" ;;
  *) echo "ERROR: Unsupported CPU architecture: $(uname -m)" >&2; exit 1 ;;
esac

# Repo root, derived from this file's own location (shared/scripts/ -> ../..).
_CDX_BUILD_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$_CDX_BUILD_DIR/../.." && pwd)"

RUST_IMAGE="${RUST_IMAGE:-rust:1.86.0-slim-bookworm@sha256:57d415bbd61ce11e2d5f73de068103c7bd9f3188dc132c97cef4a8f62989e944}"
CARGO_BUILD_ARGS="${CARGO_BUILD_ARGS:---release --locked}"
_CDX_DOCKER_PLATFORM="linux/${_CDX_DOCKER_ARCH}"

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
# The whole repo is mounted so the shared
# Cargo.lock + path deps resolve; --manifest-path builds only this crate.
_build_one_crate() {
  local crate_dir="$1"
  local crate_rel="${crate_dir#$REPO_ROOT/}"

  echo "==> Workspace not pre-built - building $crate_rel standalone for $_CDX_DOCKER_PLATFORM (Docker)"
  docker run --rm \
    --platform "$_CDX_DOCKER_PLATFORM" \
    -u "$(id -u):$(id -g)" \
    -e CARGO_HOME=/tmp/cargo \
    -v "$REPO_ROOT:/workspace" \
    -w /workspace \
    "$RUST_IMAGE" \
    cargo build --manifest-path "$crate_rel/Cargo.toml" $CARGO_BUILD_ARGS
}

# Build the crate only if its binary isn't already in the shared target/ (i.e. a
# standalone run without the one-pass workspace build). Checks one representative
# binary - a crate's [[bin]] targets are all produced by the same build.
_ensure_crate_built() {
  local crate_dir="$1"
  local a_bin="$2"
  [[ -f "$REPO_ROOT/target/release/$a_bin" ]] || _build_one_crate "$crate_dir"
}

# Entry point for the simple single-binary plugins. Installs the binary from the
# shared target/ into the plugin's bin/ - building the crate first only if the
# workspace build hasn't already produced it (standalone run). Signature
# unchanged, so those plugins' src/build.sh need no edits.
#   $1  crate dir (…/plugins/<p>/src/<crate>)   $2  binary name
build_plugin_crate() {
  local crate_dir="$1"
  local bin_name="$2"
  _ensure_crate_built "$crate_dir" "$bin_name"
  install_workspace_binary "$bin_name" "$(_cdx_bin_dir "$crate_dir")"
}

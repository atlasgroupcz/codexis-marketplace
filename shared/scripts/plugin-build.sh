#!/usr/bin/env bash
# Shared build logic for the marketplace Rust plugins. Sourced by each plugin's
# `src/build.sh`. Build-time only — never shipped (`shared/` is dist-excluded).
#
# Usage from a plugin's src/build.sh:
#   set -euo pipefail
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"
#   build_plugin_crate "$SCRIPT_DIR/<crate-dir>" "<bin-name>"
#
# Override before `source` when a plugin needs it:
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
_CDX_MACOS_ARM64_TARGET="aarch64-apple-darwin"

# Builds one crate for linux (in Docker) and, when osxcross is present, macOS
# arm64 (natively), installing both into the plugin's bin/ dir.
#   $1  absolute crate dir (…/plugins/<p>/src/<crate>)
#   $2  binary name
build_plugin_crate() {
  local crate_dir="$1"
  local bin_name="$2"
  _cdx_build_linux "$crate_dir" "$bin_name"
  _cdx_build_macos_arm64 "$crate_dir" "$bin_name"
}

_cdx_bin_dir() {
  # crate_dir = <plugin>/src/<crate>  ->  plugin bin dir = <plugin>/bin
  local crate_dir="$1"
  local plugin_dir
  plugin_dir="$(cd "$crate_dir/../.." && pwd)"
  echo "$plugin_dir/bin"
}

_cdx_build_linux() {
  local crate_dir="$1"
  local bin_name="$2"
  local bin_dir
  bin_dir="$(_cdx_bin_dir "$crate_dir")"
  mkdir -p "$bin_dir"

  if [[ ! -f "$crate_dir/Cargo.toml" ]]; then
    echo "ERROR: Missing Cargo.toml in $crate_dir" >&2
    exit 1
  fi

  echo "==> Building $bin_name for $_CDX_DOCKER_PLATFORM"
  # Mount only the crate and shared/ (not the whole repo), rebuilding their real
  # relative layout under /workspace so the crate's `../../../../shared/...`
  # path dependency resolves. The crate still builds in isolation: workdir is
  # the crate dir and, with no workspace manifest above it, cargo writes
  # `target/` into the crate dir.
  docker run --rm \
    --platform "$_CDX_DOCKER_PLATFORM" \
    -u "$(id -u):$(id -g)" \
    -e CARGO_HOME=/tmp/cargo \
    -v "$crate_dir:/workspace/${crate_dir#$REPO_ROOT/}" \
    -v "$REPO_ROOT/shared:/workspace/shared:ro" \
    -w "/workspace/${crate_dir#$REPO_ROOT/}" \
    "$RUST_IMAGE" \
    cargo build $CARGO_BUILD_ARGS

  install -m 0755 "$crate_dir/target/release/$bin_name" "$bin_dir/$bin_name"
  echo "   installed -> $bin_dir/$bin_name"
}

_cdx_build_macos_arm64() {
  local crate_dir="$1"
  local bin_name="$2"
  local bin_dir
  bin_dir="$(_cdx_bin_dir "$crate_dir")"
  local osxcross_bin="$HOME/opt/macos/target/bin"
  local sdk_root="$HOME/opt/macos/target/SDK/MacOSX26.1.sdk"
  local linker="aarch64-apple-darwin25.1-clang"
  local target_bin="$crate_dir/target/$_CDX_MACOS_ARM64_TARGET/release/$bin_name"
  local dst_bin="$bin_dir/${bin_name}-${_CDX_MACOS_ARM64_TARGET}"

  if [[ ! -d "$osxcross_bin" ]] || [[ ! -d "$sdk_root" ]]; then
    echo "==> Skipping $bin_name macOS ARM64 build: osxcross not found at $osxcross_bin"
    return 0
  fi

  echo "==> Building $bin_name for darwin/arm64"
  rustup target add "$_CDX_MACOS_ARM64_TARGET" >/dev/null

  # macOS builds natively (no Docker): the crate's relative path dependency on
  # shared/rust/cdx-common resolves directly on the host filesystem.
  (
    cd "$crate_dir"
    SDKROOT="$sdk_root" \
      PATH="$PATH:$osxcross_bin" \
      CC_aarch64_apple_darwin="$linker" \
      CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="$linker" \
      cargo build $CARGO_BUILD_ARGS --target "$_CDX_MACOS_ARM64_TARGET"
  )

  install -m 0755 "$target_bin" "$dst_bin"
  echo "   installed -> $dst_bin"
}

#!/usr/bin/env bash
# Builds the katastr-cli (system-wide CLI) and katastr-cgi (plugin component)
# Rust binaries for Linux (default, no suffix) and optionally macOS arm64 (when
# osxcross is available).
#
# Outputs:
#   plugins/katastr/bin/katastr-cli{,-aarch64-apple-darwin}
#     → installed to ~/.local/bin by hooks/install-binaries.sh
#   plugins/katastr/components/katastr/katastr-cgi{,-aarch64-apple-darwin}
#     → executed in place by the daemon via the sibling `katastr` bash dispatcher
#
# Unlike the single-crate sibling plugins (ares, zrsr, …), katastr is a Cargo
# workspace with three members (core / cli / cgi) plus a path dependency on
# shared/rust/cdx-common. We therefore mount the workspace and shared/ at their
# real repo-relative layout so `../../../shared/...` resolves inside Docker,
# then build the whole workspace in one pass.

set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "ERROR: Linux-only build script." >&2
  exit 1
fi

case "$(uname -m)" in
  x86_64|amd64)   DOCKER_ARCH="amd64" ;;
  aarch64|arm64)  DOCKER_ARCH="arm64" ;;
  *) echo "ERROR: Unsupported CPU architecture: $(uname -m)" >&2; exit 1 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # plugins/katastr/src
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"                    # plugins/katastr
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"               # marketplace root
SRC_REL="${SCRIPT_DIR#$REPO_ROOT/}"                          # plugins/katastr/src
CGI_TARGET_DIR="$PLUGIN_DIR/components/katastr"
CLI_TARGET_DIR="$PLUGIN_DIR/bin"
MACOS_ARM64_TARGET="aarch64-apple-darwin"

# Pinned image digest matches what other Rust plugins use - keeps all plugin
# binaries compiled against the same toolchain for reproducibility.
RUST_IMAGE="${RUST_IMAGE:-rust:1.86.0-slim-bookworm@sha256:57d415bbd61ce11e2d5f73de068103c7bd9f3188dc132c97cef4a8f62989e944}"

install_built_binary() {
  local bin="$1"
  local target_triple_dir="${2:-}"  # empty for the default Linux build
  local dst_dir
  case "$bin" in
    katastr-cli)  dst_dir="$CLI_TARGET_DIR" ;;
    katastr-cgi)  dst_dir="$CGI_TARGET_DIR" ;;
    *) echo "ERROR: unknown binary $bin" >&2; return 1 ;;
  esac

  mkdir -p "$dst_dir"
  local src="$SCRIPT_DIR/target"
  local dst_name="$bin"
  if [[ -n "$target_triple_dir" ]]; then
    src="$src/$target_triple_dir"
    dst_name="${bin}-${target_triple_dir}"
  fi
  src="$src/release/$bin"

  install -m 0755 "$src" "$dst_dir/$dst_name"
  echo "   installed -> $dst_dir/$dst_name"
}

build_linux() {
  echo "==> Building workspace for linux/${DOCKER_ARCH} (via Docker)"
  docker run --rm \
    --platform "linux/${DOCKER_ARCH}" \
    -u "$(id -u):$(id -g)" \
    -e CARGO_HOME=/tmp/cargo \
    -v "$SCRIPT_DIR:/workspace/${SRC_REL}" \
    -v "$REPO_ROOT/shared:/workspace/shared:ro" \
    -w "/workspace/${SRC_REL}" \
    "$RUST_IMAGE" \
    cargo build --release --locked

  install_built_binary katastr-cli
  install_built_binary katastr-cgi
}

build_macos_arm64() {
  local osxcross_bin="$HOME/opt/macos/target/bin"
  local sdk_root="$HOME/opt/macos/target/SDK/MacOSX26.1.sdk"
  local linker="aarch64-apple-darwin25.1-clang"

  if [[ ! -d "$osxcross_bin" ]] || [[ ! -d "$sdk_root" ]]; then
    echo "==> Skipping macOS ARM64 build: osxcross not found at $osxcross_bin"
    return 0
  fi

  echo "==> Building workspace for darwin/arm64 (via osxcross)"
  rustup target add "$MACOS_ARM64_TARGET" >/dev/null

  (
    cd "$SCRIPT_DIR"
    SDKROOT="$sdk_root" \
      PATH="$PATH:$osxcross_bin" \
      CC_aarch64_apple_darwin="$linker" \
      CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="$linker" \
      cargo build --release --locked --target "$MACOS_ARM64_TARGET"
  )

  install_built_binary katastr-cli "$MACOS_ARM64_TARGET"
  install_built_binary katastr-cgi "$MACOS_ARM64_TARGET"
}

build_linux
build_macos_arm64

echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

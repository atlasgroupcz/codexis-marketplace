#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "ERROR: Linux-only build script." >&2
  exit 1
fi

case "$(uname -m)" in
  x86_64|amd64)
    DOCKER_ARCH="amd64"
    ;;
  aarch64|arm64)
    DOCKER_ARCH="arm64"
    ;;
  *)
    echo "ERROR: Unsupported CPU architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$PLUGIN_DIR/bin"
MACOS_ARM64_TARGET="aarch64-apple-darwin"

RUST_IMAGE="rust:1.86.0-slim-bookworm@sha256:57d415bbd61ce11e2d5f73de068103c7bd9f3188dc132c97cef4a8f62989e944"
DOCKER_PLATFORM="linux/${DOCKER_ARCH}"

mkdir -p "$BIN_DIR"

build_crate() {
  local crate_dir="$1"
  local bin_name="$2"
  local cargo_args="${3:---release --locked}"

  if [[ ! -f "$crate_dir/Cargo.toml" ]]; then
    echo "ERROR: Missing Cargo.toml in $crate_dir" >&2
    exit 1
  fi

  echo "==> Building $bin_name for $DOCKER_PLATFORM"
  docker run --rm \
    --platform "$DOCKER_PLATFORM" \
    -u "$(id -u):$(id -g)" \
    -e CARGO_HOME=/tmp/cargo \
    -v "$crate_dir:/workspace" \
    -w /workspace \
    "$RUST_IMAGE" \
    cargo build $cargo_args

  install -m 0755 "$crate_dir/target/release/$bin_name" "$BIN_DIR/$bin_name"
  echo "   installed -> $BIN_DIR/$bin_name"
}

build_crate_macos_arm64() {
  local crate_dir="$1"
  local bin_name="$2"
  local cargo_args="${3:---release --locked}"
  local osxcross_bin="$HOME/opt/macos/target/bin"
  local sdk_root="$HOME/opt/macos/target/SDK/MacOSX26.1.sdk"
  local linker="aarch64-apple-darwin25.1-clang"
  local target_bin="$crate_dir/target/$MACOS_ARM64_TARGET/release/$bin_name"
  local dst_bin="$BIN_DIR/${bin_name}-${MACOS_ARM64_TARGET}"

  if [[ ! -d "$osxcross_bin" ]] || [[ ! -d "$sdk_root" ]]; then
    echo "==> Skipping $bin_name macOS ARM64 build: osxcross not found at $osxcross_bin"
    return 0
  fi

  echo "==> Building $bin_name for darwin/arm64"
  rustup target add "$MACOS_ARM64_TARGET" >/dev/null

  (
    cd "$crate_dir"
    SDKROOT="$sdk_root" \
      PATH="$PATH:$osxcross_bin" \
      CC_aarch64_apple_darwin="$linker" \
      CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="$linker" \
      cargo build $cargo_args --target "$MACOS_ARM64_TARGET"
  )

  install -m 0755 "$target_bin" "$dst_bin"
  echo "   installed -> $dst_bin"
}

build_crate "$SCRIPT_DIR/cdx-cli" "cdx-cli"
build_crate_macos_arm64 "$SCRIPT_DIR/cdx-cli" "cdx-cli"
build_crate "$SCRIPT_DIR/cdx-link-rewriter" "cdx-link-rewriter"
build_crate_macos_arm64 "$SCRIPT_DIR/cdx-link-rewriter" "cdx-link-rewriter"
build_crate "$SCRIPT_DIR/cdxctl" "cdxctl"
build_crate_macos_arm64 "$SCRIPT_DIR/cdxctl" "cdxctl"

echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

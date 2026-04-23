#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

BIN_NAME="cdx-sk"

build_linux_docker() {
  echo "Building $BIN_NAME for Linux ARM64 via Docker..."
  docker run --rm --platform linux/arm64 \
    -v "$SCRIPT_DIR:/src" \
    -v cdx-cargo-registry:/usr/local/cargo/registry \
    -v cdx-cargo-git:/usr/local/cargo/git \
    -w /src rust:1.86-slim \
    cargo build --release
  local bin_dst="$SCRIPT_DIR/${BIN_NAME}-arm64"
  cp "target/release/$BIN_NAME" "$bin_dst"
  chmod 0755 "$bin_dst"
  echo "Built: $bin_dst"
}

build_linux_native() {
  echo "Building $BIN_NAME for Linux..."
  cargo build --release
  local bin_dst="$SCRIPT_DIR/$BIN_NAME"
  cp "target/release/$BIN_NAME" "$bin_dst"
  chmod 0755 "$bin_dst"
  echo "Built: $bin_dst"
}

build_macos_osxcross() {
  local osxcross_bin="$HOME/opt/macos/target/bin"
  local sdk_root="$HOME/opt/macos/target/SDK/MacOSX26.1.sdk"
  local linker="aarch64-apple-darwin25.1-clang"

  if [[ ! -d "$osxcross_bin" ]] || [[ ! -d "$sdk_root" ]]; then
    echo "Skipping macOS ARM64 build: osxcross not found at $osxcross_bin" >&2
    return 0
  fi

  echo "Building $BIN_NAME for macOS ARM64 via osxcross..."
  rustup target add aarch64-apple-darwin >/dev/null

  SDKROOT="$sdk_root" \
    PATH="$PATH:$osxcross_bin" \
    CC_aarch64_apple_darwin="$linker" \
    CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="$linker" \
    cargo build --release --target aarch64-apple-darwin

  local bin_dst="$SCRIPT_DIR/${BIN_NAME}-aarch64-apple-darwin"
  cp "target/aarch64-apple-darwin/release/$BIN_NAME" "$bin_dst"
  chmod 0755 "$bin_dst"
  echo "Built: $bin_dst"
}

case "$(uname -s)" in
  Darwin)
    if command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
      build_linux_docker
    else
      echo "ERROR: Docker required to build $BIN_NAME on macOS" >&2
      exit 1
    fi
    ;;
  Linux)
    build_linux_native
    build_macos_osxcross
    ;;
  *)
    echo "Unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

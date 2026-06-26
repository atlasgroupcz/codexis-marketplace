#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "ERROR: Linux-only build script." >&2
  exit 1
fi

case "$(uname -m)" in
  x86_64|amd64) DOCKER_ARCH="amd64" ;;
  aarch64|arm64) DOCKER_ARCH="arm64" ;;
  *) echo "ERROR: Unsupported CPU architecture: $(uname -m)" >&2; exit 1 ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$PLUGIN_DIR/bin"

# Non-slim image: ships perl + make + cc, required to build the vendored OpenSSL
# that rzpro-cli statically links (the KSRZIS server only accepts native-tls).
RUST_IMAGE="rust:1.90.0-bookworm"
DOCKER_PLATFORM="linux/${DOCKER_ARCH}"

mkdir -p "$BIN_DIR"

crate_dir="$SCRIPT_DIR/rzpro-cli"
bin_name="rzpro-cli"

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
  cargo build --release --locked

install -m 0755 "$crate_dir/target/release/$bin_name" "$BIN_DIR/$bin_name"
echo "   installed -> $BIN_DIR/$bin_name"

# macOS arm64 cross-build via osxcross (skipped when osxcross isn't installed).
# On macOS native-tls uses Secure Transport, so the Linux-only vendored OpenSSL
# (see Cargo.toml) is not built here.
MACOS_ARM64_TARGET="aarch64-apple-darwin"
OSXCROSS_BIN="$HOME/opt/macos/target/bin"
SDK_ROOT="$HOME/opt/macos/target/SDK/MacOSX26.1.sdk"
MAC_LINKER="aarch64-apple-darwin25.1-clang"
if [[ -d "$OSXCROSS_BIN" && -d "$SDK_ROOT" ]]; then
  echo "==> Building $bin_name for darwin/arm64"
  rustup target add "$MACOS_ARM64_TARGET" >/dev/null
  (
    cd "$crate_dir"
    SDKROOT="$SDK_ROOT" \
      PATH="$PATH:$OSXCROSS_BIN" \
      CC_aarch64_apple_darwin="$MAC_LINKER" \
      CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="$MAC_LINKER" \
      cargo build --release --locked --target "$MACOS_ARM64_TARGET"
  )
  install -m 0755 "$crate_dir/target/$MACOS_ARM64_TARGET/release/$bin_name" \
    "$BIN_DIR/${bin_name}-${MACOS_ARM64_TARGET}"
  echo "   installed -> $BIN_DIR/${bin_name}-${MACOS_ARM64_TARGET}"
else
  echo "==> Skipping $bin_name macOS ARM64 build: osxcross not found at $OSXCROSS_BIN"
fi

echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

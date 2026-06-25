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
echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

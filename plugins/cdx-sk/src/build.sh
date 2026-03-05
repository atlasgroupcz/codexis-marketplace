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

RUST_IMAGE="rust:1.86.0-slim-bookworm@sha256:57d415bbd61ce11e2d5f73de068103c7bd9f3188dc132c97cef4a8f62989e944"
DOCKER_PLATFORM="linux/${DOCKER_ARCH}"

mkdir -p "$BIN_DIR"

build_crate() {
  local crate_dir="$1"
  local bin_name="$2"

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
}

build_crate "$SCRIPT_DIR/cdx-sk-link-rewriter" "cdx-sk-link-rewriter"

echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Non-slim image: ships perl + make + cc for the vendored OpenSSL that
# rzpro-cli statically links (the KSRZIS server only accepts native-tls).
RUST_IMAGE="rust:1.90.0-bookworm"

source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

build_plugin_crate "$SCRIPT_DIR/rzpro-cli" "rzpro-cli"

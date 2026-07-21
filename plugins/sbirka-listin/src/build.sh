#!/usr/bin/env bash
# Builds both binaries of the sbirka-listin crate:
#   sbirka-listin-cli → plugins/sbirka-listin/bin/            (system-wide CLI)
#   sbirka-listin-cgi → plugins/sbirka-listin/components/…/   (dashboard handler,
#                       run in place by the daemon via the `sbirka-listin` dispatcher)
#
# A single Cargo build produces both bins; `build_plugin_crate` compiles the
# crate (linux via Docker + macOS arm64 via osxcross when present) and installs
# the CLI into bin/. This script then copies the CGI binary that same build left
# in target/ into the component dir, for each arch that was built.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

CRATE_DIR="$SCRIPT_DIR/sbirka-listin-cli"
CGI_TARGET_DIR="$SCRIPT_DIR/../components/sbirka-listin"
MACOS_ARM64_TARGET="aarch64-apple-darwin"

# Build the crate (both bins) and install the CLI into bin/.
build_plugin_crate "$CRATE_DIR" "sbirka-listin-cli"

# Install the CGI binary the same build produced, per arch, into the component.
install_cgi() {
  local target_triple_dir="${1:-}"  # empty for the default linux build
  local src dst_name
  if [[ -n "$target_triple_dir" ]]; then
    src="$CRATE_DIR/target/$target_triple_dir/release/sbirka-listin-cgi"
    dst_name="sbirka-listin-cgi-${target_triple_dir}"
  else
    src="$CRATE_DIR/target/release/sbirka-listin-cgi"
    dst_name="sbirka-listin-cgi"
  fi
  if [[ ! -f "$src" ]]; then
    echo "==> Skipping CGI ${target_triple_dir:-linux}: not built ($src)"
    return 0
  fi
  mkdir -p "$CGI_TARGET_DIR"
  install -m 0755 "$src" "$CGI_TARGET_DIR/$dst_name"
  echo "   installed -> $CGI_TARGET_DIR/$dst_name"
}

install_cgi
install_cgi "$MACOS_ARM64_TARGET"

echo "Completed plugin build: sbirka-listin"

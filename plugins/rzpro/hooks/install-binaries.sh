#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

install_binary() {
  local name="$1"
  local source_path="${SOURCE_BIN_DIR}/${name}"
  local target_path="${TARGET_BIN_DIR}/${name}"
  if [[ ! -x "${source_path}" ]]; then
    echo "ERROR: Missing executable ${source_path}" >&2
    exit 1
  fi
  if [[ -d "${TARGET_BIN_DIR}" && -w "${TARGET_BIN_DIR}" ]]; then
    install -m 0755 "${source_path}" "${target_path}"
  else
    install -d "${TARGET_BIN_DIR}"
    install -m 0755 "${source_path}" "${target_path}"
  fi
  echo "Installed ${name} -> ${target_path}"
}

install_binary "rzpro-cli"

# Pre-download the datasets, but don't fail the install on a network hiccup.
"${TARGET_BIN_DIR}/rzpro-cli" refresh >/dev/null 2>&1 \
  || echo "WARN: rzpro-cli refresh failed (will retry lazily on first use)" >&2

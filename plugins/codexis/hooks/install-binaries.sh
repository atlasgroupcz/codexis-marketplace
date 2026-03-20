#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-/usr/local/bin}"

install_binary() {
  local binary_name="$1"
  local source_path="${SOURCE_BIN_DIR}/${binary_name}"
  local target_path="${TARGET_BIN_DIR}/${binary_name}"

  if [[ ! -x "${source_path}" ]]; then
    echo "ERROR: Missing executable ${source_path}" >&2
    exit 1
  fi

  if [[ -d "${TARGET_BIN_DIR}" && -w "${TARGET_BIN_DIR}" ]]; then
    install -m 0755 "${source_path}" "${target_path}"
  else
    sudo install -d "${TARGET_BIN_DIR}"
    sudo install -m 0755 "${source_path}" "${target_path}"
  fi

  echo "Installed ${binary_name} -> ${target_path}"
}

install_binary "cdx-cli"
install_binary "cdx-link-rewriter"
install_binary "cdx-sledovane-dokumenty"
install_binary "cdxctl"

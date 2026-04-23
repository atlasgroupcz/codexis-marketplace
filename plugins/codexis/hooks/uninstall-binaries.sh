#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-/usr/local/bin}"

remove_binary() {
  local binary_name="$1"
  local target_path="${TARGET_BIN_DIR}/${binary_name}"

  if [[ -d "${TARGET_BIN_DIR}" && -w "${TARGET_BIN_DIR}" ]]; then
    rm -f "${target_path}"
  else
    sudo rm -f "${target_path}"
  fi
}

remove_binary "cdx-cli"
remove_binary "cdx-link-rewriter"
remove_binary "cdx-sledovane-dokumenty"
remove_binary "cdx-sledovana-judikatura"
remove_binary "cdxctl"

# Remove the plugin-owned share directory (matches katastr's full-nuke pattern).
SHARE_DIR="/usr/local/share/codexis"
if [[ -d "${SHARE_DIR}" ]]; then
  if [[ -w "${SHARE_DIR}" ]]; then
    rm -rf "${SHARE_DIR}"
  else
    sudo rm -rf "${SHARE_DIR}"
  fi
fi

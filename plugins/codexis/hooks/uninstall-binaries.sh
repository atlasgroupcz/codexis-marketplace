#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

remove_path() {
  local path="$1"
  [[ -e "${path}" ]] || return 0
  rm -rf "${path}" 2>/dev/null || sudo rm -rf "${path}"
  echo "Removed ${path}"
}

remove_path "${TARGET_BIN_DIR}/cdx-cli"
remove_path "${TARGET_BIN_DIR}/cdx-sledovane-dokumenty"
remove_path "${TARGET_BIN_DIR}/cdx-sledovana-judikatura"
remove_path "${TARGET_BIN_DIR}/cdxctl"
remove_path "${HOME}/.local/share/codexis"

# Sweep leftover temp files from an interrupted install (e.g. power loss).
rm -f "${TARGET_BIN_DIR}"/.cdx-cli.tmp.* 2>/dev/null || true
rm -f "${TARGET_BIN_DIR}"/.cdx-sledovane-dokumenty.tmp.* 2>/dev/null || true
rm -f "${TARGET_BIN_DIR}"/.cdx-sledovana-judikatura.tmp.* 2>/dev/null || true
rm -f "${TARGET_BIN_DIR}"/.cdxctl.tmp.* 2>/dev/null || true

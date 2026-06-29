#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

remove_path() {
  local path="$1"
  [[ -e "${path}" ]] || return 0
  rm -rf "${path}" 2>/dev/null || sudo rm -rf "${path}"
  echo "Removed ${path}"
}

USER_HOME="${CODEXIS_PUBLIC_USER_HOME:-$HOME}"
remove_path "${TARGET_BIN_DIR}/leciva-cli"
remove_path "${USER_HOME}/.cdx/apps/leciva"

# Sweep leftover temp files from an interrupted install (e.g. power loss).
rm -f "${TARGET_BIN_DIR}"/.leciva-cli.tmp.* 2>/dev/null || true

#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
SHARE_DIR="${TARGET_SHARE_DIR:-${HOME}/.local/share}"

remove_path() {
  local path="$1"
  [[ -e "${path}" ]] || return 0
  rm -rf "${path}" 2>/dev/null || sudo rm -rf "${path}"
  echo "Removed ${path}"
}

remove_path "${TARGET_BIN_DIR}/sid-cli"
remove_path "${SHARE_DIR}/sid"

# The user's API key (~/.cdx/apps/sid/) and setting overrides
# (~/.cdx/app-state/sid/) are deliberately
# left alone - a reinstall would otherwise force issuing a new key in SID.

# Sweep leftover temp files from an interrupted install (e.g. power loss).
rm -f "${TARGET_BIN_DIR}"/.sid-cli.tmp.* 2>/dev/null || true

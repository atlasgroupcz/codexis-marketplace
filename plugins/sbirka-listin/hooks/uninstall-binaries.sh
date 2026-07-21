#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

remove_path() {
  local path="$1"
  [[ -e "${path}" ]] || return 0
  rm -rf "${path}" 2>/dev/null || sudo rm -rf "${path}"
  echo "Removed ${path}"
}

remove_path "${TARGET_BIN_DIR}/sbirka-listin-cli"

# Installed notify defaults. The user's own tracked subjects, downloads and
# settings override live under ~/.cdx/apps/sbirka-listin and are left untouched.
remove_path "${HOME}/.local/share/sbirka-listin"

# Sweep leftover temp files from an interrupted install (e.g. power loss).
rm -f "${TARGET_BIN_DIR}"/.sbirka-listin-cli.tmp.* 2>/dev/null || true

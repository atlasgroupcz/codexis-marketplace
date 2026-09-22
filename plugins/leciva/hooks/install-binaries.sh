#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
source "${PLUGIN_DIR}/../../plugin-runtime/install-executable.sh"

cleanup_own_temps() {
  rm -f "${TARGET_BIN_DIR}"/.*.tmp.$$ 2>/dev/null || true
}
trap cleanup_own_temps EXIT INT TERM

install_binary() {
  local source_name="$1"
  local target_name="${2:-$1}"
  local source_path="${SOURCE_BIN_DIR}/${source_name}"
  local target_path="${TARGET_BIN_DIR}/${target_name}"
  if [[ ! -x "${source_path}" ]]; then
    echo "ERROR: Missing executable ${source_path}" >&2
    exit 1
  fi
  install_executable "${source_path}" "${target_path}" "${target_name}"
}


install_binary "leciva-cli" "leciva-cli"

# Migrate off the pre-1.4 local index. Old versions downloaded ~23 MB of SÚKL
# data (TSV index + codelists) to ~/.cdx/apps/leciva; the API client needs none
# of it. Best-effort - MUST NOT fail install/update, hence || true (no sudo: the
# dir is user-owned under $HOME).
USER_HOME="${CODEXIS_PUBLIC_USER_HOME:-$HOME}"
if [[ -e "${USER_HOME}/.cdx/apps/leciva" ]]; then
  rm -rf "${USER_HOME}/.cdx/apps/leciva" 2>/dev/null || true
  echo "Removed legacy local index: ${USER_HOME}/.cdx/apps/leciva" >&2
fi
if [[ -e "${USER_HOME}/.cdx/app-state/leciva" ]]; then
  rm -rf "${USER_HOME}/.cdx/app-state/leciva" 2>/dev/null || true
  echo "Removed legacy local index: ${USER_HOME}/.cdx/app-state/leciva" >&2
fi

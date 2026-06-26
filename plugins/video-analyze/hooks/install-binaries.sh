#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

# Remove only THIS run's temp artifacts, even on crash or interrupt. Scoped to
# the current PID so a concurrent install can't delete an in-flight temp file.
cleanup_own_temps() {
  rm -f "${TARGET_BIN_DIR}"/.*.tmp.$$ 2>/dev/null || true
}
trap cleanup_own_temps EXIT INT TERM

select_binary_source() {
  local binary_name="$1"

  case "$(uname -s):$(uname -m)" in
    Darwin:arm64|Darwin:aarch64)
      echo "${binary_name}-aarch64-apple-darwin"
      ;;
    Darwin:*)
      echo "ERROR: ${binary_name} ships a macOS build for Apple Silicon only" >&2
      exit 1
      ;;
    *)
      echo "${binary_name}"
      ;;
  esac
}

install_binary() {
  local source_name="$1"
  local target_name="${2:-$1}"
  local source_path="${SOURCE_BIN_DIR}/${source_name}"
  local target_path="${TARGET_BIN_DIR}/${target_name}"
  if [[ ! -x "${source_path}" ]]; then
    echo "ERROR: Missing executable ${source_path}" >&2
    exit 1
  fi
  # Atomic install: write to a temp file in the destination dir, flush it to
  # disk, then rename into place. A crash leaves either the old binary or the
  # complete new one — never a truncated/empty file at the real name.
  local tmp_path="${TARGET_BIN_DIR}/.${target_name}.tmp.$$"
  if mkdir -p "${TARGET_BIN_DIR}" 2>/dev/null && [[ -w "${TARGET_BIN_DIR}" ]]; then
    install -m 0755 "${source_path}" "${tmp_path}"
    sync "${tmp_path}" 2>/dev/null || sync
    mv -f "${tmp_path}" "${target_path}"
  else
    sudo install -d "${TARGET_BIN_DIR}"
    sudo install -m 0755 "${source_path}" "${tmp_path}"
    sudo sync "${tmp_path}" 2>/dev/null || sudo sync
    sudo mv -f "${tmp_path}" "${target_path}"
  fi
  echo "Installed ${target_name} -> ${target_path}"
}

install_binary "$(select_binary_source "video-analyze")" "video-analyze"

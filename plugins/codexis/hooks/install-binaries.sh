#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
SHARED_LIB_DIR="${HOME}/.local/share/codexis/lib"

# Remove only THIS run's temp artifacts, even on crash or interrupt. Scoped to
# the current PID so a concurrent install can't delete an in-flight temp file.
cleanup_own_temps() {
  rm -f "${TARGET_BIN_DIR}"/.*.tmp.$$ 2>/dev/null || true
  rm -rf "${SHARED_LIB_DIR}"/.*.tmp.$$ "${SHARED_LIB_DIR}"/.*.old.$$ 2>/dev/null || true
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

install_binary "$(select_binary_source "cdx-cli")" "cdx-cli"
install_binary "cdx-sledovane-dokumenty"
install_binary "cdx-sledovana-judikatura"
install_binary "$(select_binary_source "cdxctl")" "cdxctl"

install_lib() {
  local module_name="$1"
  local source_path="${PLUGIN_DIR}/lib/${module_name}"
  if [[ ! -d "${source_path}" ]]; then
    return
  fi
  # Atomic install: copy the module into a temp dir, flush it to disk, then
  # swap it into place via rename. A crash never leaves a half-copied module
  # at the real path; worst case it is briefly absent and the next run restores it.
  local dest="${SHARED_LIB_DIR}/${module_name}"
  local tmp_dest="${SHARED_LIB_DIR}/.${module_name}.tmp.$$"
  local old_dest="${SHARED_LIB_DIR}/.${module_name}.old.$$"
  if mkdir -p "${SHARED_LIB_DIR}" 2>/dev/null && [[ -w "${SHARED_LIB_DIR}" ]]; then
    cp -r "${source_path}" "${tmp_dest}"
    sync "${tmp_dest}" 2>/dev/null || sync
    [[ -e "${dest}" ]] && mv -f "${dest}" "${old_dest}"
    mv -f "${tmp_dest}" "${dest}"
    rm -rf "${old_dest}"
  else
    sudo install -d "${SHARED_LIB_DIR}"
    sudo cp -r "${source_path}" "${tmp_dest}"
    sudo sync "${tmp_dest}" 2>/dev/null || sudo sync
    [[ -e "${dest}" ]] && sudo mv -f "${dest}" "${old_dest}"
    sudo mv -f "${tmp_dest}" "${dest}"
    sudo rm -rf "${old_dest}"
  fi
  echo "Installed ${module_name} -> ${dest}"
}

install_lib "sledovane_dokumenty_core"
install_lib "sledovana_judikatura_core"

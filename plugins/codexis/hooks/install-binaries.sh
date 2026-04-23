#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-/usr/local/bin}"

install_binary() {
  local source_name="$1"
  local target_name="${2:-$1}"
  local source_path="${SOURCE_BIN_DIR}/${source_name}"
  local target_path="${TARGET_BIN_DIR}/${target_name}"

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

  echo "Installed ${source_name} -> ${target_path}"
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
install_binary "$(select_binary_source "cdx-link-rewriter")" "cdx-link-rewriter"
install_binary "cdx-sledovane-dokumenty"
install_binary "cdx-sledovana-judikatura"
install_binary "$(select_binary_source "cdxctl")" "cdxctl"

# Install shared core library used by CLI and CGI handler.
SHARED_LIB_DIR="/usr/local/share/codexis/lib"
install_lib() {
  local module_name="$1"
  local source_path="${PLUGIN_DIR}/lib/${module_name}"
  if [[ ! -d "${source_path}" ]]; then
    return
  fi
  if [[ -d "${SHARED_LIB_DIR}" && -w "${SHARED_LIB_DIR}" ]]; then
    install -d "${SHARED_LIB_DIR}"
    rm -rf "${SHARED_LIB_DIR}/${module_name}"
    cp -r "${source_path}" "${SHARED_LIB_DIR}/${module_name}"
  else
    sudo install -d "${SHARED_LIB_DIR}"
    sudo rm -rf "${SHARED_LIB_DIR}/${module_name}"
    sudo cp -r "${source_path}" "${SHARED_LIB_DIR}/${module_name}"
  fi
  echo "Installed ${module_name} -> ${SHARED_LIB_DIR}/${module_name}"
}

install_lib "sledovane_dokumenty_core"
install_lib "sledovana_judikatura_core"

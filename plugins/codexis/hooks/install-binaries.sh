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

select_cdx_cli_binary() {
  case "$(uname -s):$(uname -m)" in
    Darwin:arm64|Darwin:aarch64)
      echo "cdx-cli-aarch64-apple-darwin"
      ;;
    Darwin:*)
      echo "ERROR: cdx-cli ships a macOS build for Apple Silicon only" >&2
      exit 1
      ;;
    *)
      echo "cdx-cli"
      ;;
  esac
}

install_binary "$(select_cdx_cli_binary)" "cdx-cli"
install_binary "cdx-link-rewriter"
install_binary "cdx-sledovane-dokumenty"
install_binary "cdx-sledovana-judikatura"
install_binary "cdxctl"

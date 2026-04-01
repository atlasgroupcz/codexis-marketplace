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

remove_binary "cdx-cz-psp"
remove_binary "cdx-cz-psp-link-rewriter"

#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

remove_binary() {
  local target_name="$1"
  local target_path="${TARGET_BIN_DIR}/${target_name}"

  if [[ ! -e "${target_path}" ]]; then
    return 0
  fi

  if [[ -w "${TARGET_BIN_DIR}" ]]; then
    rm -f "${target_path}"
  else
    sudo rm -f "${target_path}"
  fi

  echo "Removed ${target_path}"
}

remove_binary "zrsr-cli"

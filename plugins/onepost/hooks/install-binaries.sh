#!/usr/bin/env bash

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
SHARED_LIB_DIR="${HOME}/.local/share/onepost/lib"
source "${PLUGIN_DIR}/../../plugin-runtime/install-executable.sh"

function main() {
  install_lib || return 1
  install_executable \
    "${PLUGIN_DIR}/bin/onepost-cli" \
    "${TARGET_BIN_DIR}/onepost-cli" \
    "onepost-cli"
}

function install_lib() {
  local source_path="${PLUGIN_DIR}/lib/onepost_core"
  local target_path="${SHARED_LIB_DIR}/onepost_core"

  if [[ ! -d "${source_path}" ]]; then
    echo "ERROR: Missing library ${source_path}" >&2
    return 1
  fi
  if ! mkdir -p "${SHARED_LIB_DIR}"; then
    echo "ERROR: Cannot create library directory ${SHARED_LIB_DIR}" >&2
    return 1
  fi
  if ! rm -rf "${target_path}"; then
    echo "ERROR: Cannot replace library ${target_path}" >&2
    return 1
  fi
  if ! cp -r "${source_path}" "${target_path}"; then
    echo "ERROR: Cannot install library ${target_path}" >&2
    return 1
  fi
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
  exit $?
fi

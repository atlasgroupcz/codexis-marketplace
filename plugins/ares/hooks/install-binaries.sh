#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
TARGET_LIB_DIR="${TARGET_LIB_DIR:-${HOME}/.local/share/ares/lib}"

install -d "${TARGET_BIN_DIR}" "${TARGET_LIB_DIR}"
rm -rf "${TARGET_LIB_DIR}/ares_cli"
cp -R "${PLUGIN_DIR}/lib/ares_cli" "${TARGET_LIB_DIR}/ares_cli"
install -m 0755 "${PLUGIN_DIR}/bin/ares" "${TARGET_BIN_DIR}/ares"

echo "Installed ares -> ${TARGET_BIN_DIR}/ares"
echo "Installed ares_cli library -> ${TARGET_LIB_DIR}/ares_cli"

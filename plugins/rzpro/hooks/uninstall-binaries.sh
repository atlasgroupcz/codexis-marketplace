#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

rm -f "${TARGET_BIN_DIR}/rzpro-cli"
echo "Removed rzpro-cli from ${TARGET_BIN_DIR}"

#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
TARGET_SHARE_DIR="${TARGET_SHARE_DIR:-${HOME}/.local/share/ares}"

rm -f "${TARGET_BIN_DIR}/ares"
rm -rf "${TARGET_SHARE_DIR}"

echo "Removed ${TARGET_BIN_DIR}/ares"
echo "Removed ${TARGET_SHARE_DIR}"

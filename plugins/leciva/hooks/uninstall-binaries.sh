#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
USER_HOME="${CODEXIS_PUBLIC_USER_HOME:-$HOME}"

rm -f "${TARGET_BIN_DIR}/leciva-cli" "${TARGET_BIN_DIR}/leciva-cli-aarch64-apple-darwin"
rm -rf "${USER_HOME}/.cdx/apps/leciva"
echo "Removed leciva-cli and ~/.cdx/apps/leciva"

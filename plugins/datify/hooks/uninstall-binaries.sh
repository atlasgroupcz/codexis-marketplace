#!/usr/bin/env bash
set -euo pipefail

TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

# The stored API token in ~/.cdx/apps/datify/ is deliberately left in place:
# reinstalling the plugin should not make the user generate a new one.
for name in datify-cli; do
  target="${TARGET_BIN_DIR}/${name}"
  if [[ -e "${target}" ]]; then
    rm -f "${target}" 2>/dev/null || sudo rm -f "${target}"
    echo "Removed ${target}"
  fi
done

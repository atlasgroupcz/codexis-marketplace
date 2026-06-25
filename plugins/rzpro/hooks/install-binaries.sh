#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

install_binary() {
  local name="$1"
  local source_path="${SOURCE_BIN_DIR}/${name}"
  local target_path="${TARGET_BIN_DIR}/${name}"
  if [[ ! -x "${source_path}" ]]; then
    echo "ERROR: Missing executable ${source_path}" >&2
    exit 1
  fi
  # Atomic install: write to a temp file in the destination dir, flush it to
  # disk, then rename into place. A crash leaves either the old binary or the
  # complete new one — never a truncated/empty file at the real name.
  local tmp_path="${target_path}.tmp.$$"
  rm -f "${target_path}".tmp.* 2>/dev/null || true
  if mkdir -p "${TARGET_BIN_DIR}" 2>/dev/null && [[ -w "${TARGET_BIN_DIR}" ]]; then
    install -m 0755 "${source_path}" "${tmp_path}"
    sync "${tmp_path}" 2>/dev/null || sync
    mv -f "${tmp_path}" "${target_path}"
  else
    sudo install -d "${TARGET_BIN_DIR}"
    sudo install -m 0755 "${source_path}" "${tmp_path}"
    sudo sync "${tmp_path}" 2>/dev/null || sudo sync
    sudo mv -f "${tmp_path}" "${target_path}"
  fi
  echo "Installed ${name} -> ${target_path}"
}

install_binary "rzpro-cli"

# Pre-download the datasets, but don't fail the install on a network hiccup.
"${TARGET_BIN_DIR}/rzpro-cli" refresh >/dev/null 2>&1 \
  || echo "WARN: rzpro-cli refresh failed (will retry lazily on first use)" >&2

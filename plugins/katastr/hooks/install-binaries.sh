#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
SHARE_DIR="${HOME}/.local/share"
source "${PLUGIN_DIR}/../../plugin-runtime/install-executable.sh"

# Remove only THIS run's temp artifacts, even on crash or interrupt. Scoped to
# the current PID so a concurrent install can't delete an in-flight temp file.
cleanup_own_temps() {
  rm -f "${TARGET_BIN_DIR}"/.*.tmp.$$ 2>/dev/null || true
}
trap cleanup_own_temps EXIT INT TERM

install_binary() {
  local name="$1"
  local source_path="${SOURCE_BIN_DIR}/${name}"
  local target_path="${TARGET_BIN_DIR}/${name}"
  if [[ ! -x "${source_path}" ]]; then
    echo "ERROR: Missing executable ${source_path}" >&2
    exit 1
  fi
  install_executable "${source_path}" "${target_path}" "${name}"
}

# The katastr-cli binary is a self-contained Rust executable - no Python lib to
# install anymore (the former `install_lib katastr_core` step is gone).
install_binary "katastr-cli"

install_settings() {
  local source_root="${PLUGIN_DIR}/settings"
  [[ -d "$source_root" ]] || return 0
  for app_dir in "$source_root"/*/; do
    [[ -d "$app_dir" ]] || continue
    local app_name dest tmp_dest old_dest
    app_name="$(basename "$app_dir")"
    dest="${SHARE_DIR}/${app_name}/settings"
    tmp_dest="${SHARE_DIR}/${app_name}/.settings.tmp.$$"
    old_dest="${SHARE_DIR}/${app_name}/.settings.old.$$"
    mkdir -p "${SHARE_DIR}/${app_name}"
    rm -rf "$tmp_dest"
    cp -a "$app_dir" "$tmp_dest"
    if [[ -d "$dest" ]]; then mv "$dest" "$old_dest"; fi
    mv "$tmp_dest" "$dest"
    rm -rf "$old_dest"
    echo "Installed settings/${app_name} -> ${dest}"
  done
}

install_settings

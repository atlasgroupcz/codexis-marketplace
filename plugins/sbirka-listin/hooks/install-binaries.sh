#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
SHARE_DIR="${HOME}/.local/share"

# Remove only THIS run's temp artifacts, even on crash or interrupt. Scoped to
# the current PID so a concurrent install can't delete an in-flight temp file.
cleanup_own_temps() {
  rm -f "${TARGET_BIN_DIR}"/.*.tmp.$$ 2>/dev/null || true
  rm -rf "${SHARE_DIR}"/*/.settings.tmp.$$ "${SHARE_DIR}"/*/.settings.old.$$ 2>/dev/null || true
}
trap cleanup_own_temps EXIT INT TERM

install_binary() {
  local source_name="$1"
  local target_name="${2:-$1}"
  local source_path="${SOURCE_BIN_DIR}/${source_name}"
  local target_path="${TARGET_BIN_DIR}/${target_name}"
  if [[ ! -x "${source_path}" ]]; then
    echo "ERROR: Missing executable ${source_path}" >&2
    exit 1
  fi
  # Atomic install: write to a temp file in the destination dir, flush it to
  # disk, then rename into place. A crash leaves either the old binary or the
  # complete new one - never a truncated/empty file at the real name.
  local tmp_path="${TARGET_BIN_DIR}/.${target_name}.tmp.$$"
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
  echo "Installed ${target_name} -> ${target_path}"
}


install_binary "sbirka-listin-cli" "sbirka-listin-cli"

# Ship the notify defaults to where the CLI reads them
# (`~/.local/share/<app>/settings/defaults.json`, the layered SettingsStore's
# defaults path). Atomic per app: copy into a temp dir, swap the old one out,
# rename into place - a crash never leaves a half-copied settings dir.
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

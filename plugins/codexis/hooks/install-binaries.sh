#!/usr/bin/env bash
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SOURCE_BIN_DIR="${PLUGIN_DIR}/bin"
TARGET_BIN_DIR="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
SHARED_LIB_DIR="${HOME}/.local/share/codexis/lib"
SETTINGS_DIR="${HOME}/.local/share/codexis/settings"
source "${PLUGIN_DIR}/../../plugin-runtime/install-executable.sh"

# Remove only THIS run's temp artifacts, even on crash or interrupt. Scoped to
# the current PID so a concurrent install can't delete an in-flight temp file.
cleanup_own_temps() {
  rm -f "${TARGET_BIN_DIR}"/.*.tmp.$$ 2>/dev/null || true
  rm -rf "${SHARED_LIB_DIR}"/.*.tmp.$$ "${SHARED_LIB_DIR}"/.*.old.$$ 2>/dev/null || true
  rm -rf "${SETTINGS_DIR}"/.*.tmp.$$ "${SETTINGS_DIR}"/.*.old.$$ 2>/dev/null || true
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
  install_executable "${source_path}" "${target_path}" "${target_name}"
}

install_binary "cdx-cli" "cdx-cli"
install_binary "cdx-sledovane-dokumenty"
install_binary "cdx-sledovana-judikatura"
install_binary "cdxctl" "cdxctl"

install_lib() {
  local module_name="$1"
  local source_path="${PLUGIN_DIR}/lib/${module_name}"
  if [[ ! -d "${source_path}" ]]; then
    return
  fi
  # Atomic install: copy the module into a temp dir, flush it to disk, then
  # swap it into place via rename. A crash never leaves a half-copied module
  # at the real path; worst case it is briefly absent and the next run restores it.
  local dest="${SHARED_LIB_DIR}/${module_name}"
  local tmp_dest="${SHARED_LIB_DIR}/.${module_name}.tmp.$$"
  local old_dest="${SHARED_LIB_DIR}/.${module_name}.old.$$"
  if mkdir -p "${SHARED_LIB_DIR}" 2>/dev/null && [[ -w "${SHARED_LIB_DIR}" ]]; then
    cp -r "${source_path}" "${tmp_dest}"
    sync "${tmp_dest}" 2>/dev/null || sync
    [[ -e "${dest}" ]] && mv -f "${dest}" "${old_dest}"
    mv -f "${tmp_dest}" "${dest}"
    rm -rf "${old_dest}"
  else
    sudo install -d "${SHARED_LIB_DIR}"
    sudo cp -r "${source_path}" "${tmp_dest}"
    sudo sync "${tmp_dest}" 2>/dev/null || sudo sync
    [[ -e "${dest}" ]] && sudo mv -f "${dest}" "${old_dest}"
    sudo mv -f "${tmp_dest}" "${dest}"
    sudo rm -rf "${old_dest}"
  fi
  echo "Installed ${module_name} -> ${dest}"
}

install_lib "sledovane_dokumenty_core"
install_lib "sledovana_judikatura_core"

install_settings() {
  local source_root="${PLUGIN_DIR}/settings"
  if [[ ! -d "${source_root}" ]]; then
    return
  fi
  # Ship one subdir per app (e.g. settings/sledovane-dokumenty/), each
  # installed atomically like install_lib above: copy into a temp dir, flush
  # it to disk, then swap it into place via rename. A crash never leaves a
  # half-copied tree at the real path. These are the read-only shipped
  # defaults consumed by settings.defaults_path(); overwriting them on every
  # install/update is intended (refresh-on-update), and user overrides live
  # elsewhere under ~/.cdx/apps/, which this never touches.
  local source_path app_name dest tmp_dest old_dest
  for source_path in "${source_root}"/*; do
    [[ -d "${source_path}" ]] || continue
    app_name="$(basename "${source_path}")"
    dest="${SETTINGS_DIR}/${app_name}"
    tmp_dest="${SETTINGS_DIR}/.${app_name}.tmp.$$"
    old_dest="${SETTINGS_DIR}/.${app_name}.old.$$"
    if mkdir -p "${SETTINGS_DIR}" 2>/dev/null && [[ -w "${SETTINGS_DIR}" ]]; then
      cp -r "${source_path}" "${tmp_dest}"
      sync "${tmp_dest}" 2>/dev/null || sync
      [[ -e "${dest}" ]] && mv -f "${dest}" "${old_dest}"
      mv -f "${tmp_dest}" "${dest}"
      rm -rf "${old_dest}"
    else
      sudo install -d "${SETTINGS_DIR}"
      sudo cp -r "${source_path}" "${tmp_dest}"
      sudo sync "${tmp_dest}" 2>/dev/null || sudo sync
      [[ -e "${dest}" ]] && sudo mv -f "${dest}" "${old_dest}"
      sudo mv -f "${tmp_dest}" "${dest}"
      sudo rm -rf "${old_dest}"
    fi
    echo "Installed settings/${app_name} -> ${dest}"
  done
}

install_settings

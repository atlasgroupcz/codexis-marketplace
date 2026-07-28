#!/usr/bin/env bash

function install_executable() {
  local source_path="$1"
  local target_path="$2"
  local display_name="${3:-$(basename "${target_path}")}"
  local target_dir
  local tmp_path

  if [[ ! -f "${source_path}" || ! -s "${source_path}" ]]; then
    echo "ERROR: Missing or empty executable ${source_path}" >&2
    return 1
  fi
  if executable_matches "${source_path}" "${target_path}"; then
    echo "Verified ${display_name} -> ${target_path}"
    return 0
  fi

  target_dir="$(dirname "${target_path}")"
  tmp_path="${target_dir}/.$(basename "${target_path}").tmp.$$"
  if mkdir -p "${target_dir}" 2>/dev/null && [[ -w "${target_dir}" ]]; then
    install_executable_without_sudo "${source_path}" "${target_path}" "${tmp_path}" || return 1
  else
    install_executable_with_sudo "${source_path}" "${target_path}" "${tmp_path}" || return 1
  fi

  if ! executable_matches "${source_path}" "${target_path}"; then
    echo "ERROR: Installed executable failed integrity verification ${target_path}" >&2
    return 1
  fi
  echo "Installed ${display_name} -> ${target_path}"
}

function install_executable_without_sudo() {
  local source_path="$1"
  local target_path="$2"
  local tmp_path="$3"

  rm -f "${tmp_path}"
  if ! install -m 0755 "${source_path}" "${tmp_path}"; then
    echo "ERROR: Cannot stage executable ${target_path}" >&2
    return 1
  fi
  if ! executable_matches "${source_path}" "${tmp_path}"; then
    rm -f "${tmp_path}"
    echo "ERROR: Staged executable failed integrity verification ${target_path}" >&2
    return 1
  fi
  sync "${tmp_path}" 2>/dev/null || sync
  if ! mv -f "${tmp_path}" "${target_path}"; then
    rm -f "${tmp_path}"
    echo "ERROR: Cannot publish executable ${target_path}" >&2
    return 1
  fi
}

function install_executable_with_sudo() {
  local source_path="$1"
  local target_path="$2"
  local tmp_path="$3"

  if ! sudo install -d "$(dirname "${target_path}")"; then
    echo "ERROR: Cannot create executable directory for ${target_path}" >&2
    return 1
  fi
  sudo rm -f "${tmp_path}"
  if ! sudo install -m 0755 "${source_path}" "${tmp_path}"; then
    echo "ERROR: Cannot stage executable ${target_path}" >&2
    return 1
  fi
  if ! sudo_executable_matches "${source_path}" "${tmp_path}"; then
    sudo rm -f "${tmp_path}"
    echo "ERROR: Staged executable failed integrity verification ${target_path}" >&2
    return 1
  fi
  sudo sync "${tmp_path}" 2>/dev/null || sudo sync
  if ! sudo mv -f "${tmp_path}" "${target_path}"; then
    sudo rm -f "${tmp_path}"
    echo "ERROR: Cannot publish executable ${target_path}" >&2
    return 1
  fi
}

function executable_matches() {
  local source_path="$1"
  local target_path="$2"
  local source_checksum
  local target_checksum

  [[ -f "${target_path}" && -s "${target_path}" && -x "${target_path}" ]] || return 1
  source_checksum="$(sha256_file "${source_path}")" || return 1
  target_checksum="$(sha256_file "${target_path}")" || return 1
  [[ "${source_checksum}" == "${target_checksum}" ]]
}

function sudo_executable_matches() {
  local source_path="$1"
  local target_path="$2"
  local source_checksum
  local target_checksum

  sudo test -f "${target_path}" && sudo test -s "${target_path}" && sudo test -x "${target_path}" || return 1
  source_checksum="$(sha256_file "${source_path}")" || return 1
  target_checksum="$(sudo_sha256_file "${target_path}")" || return 1
  [[ "${source_checksum}" == "${target_checksum}" ]]
}

function sha256_file() {
  local path="$1"
  local output

  if command -v sha256sum >/dev/null 2>&1; then
    output="$(sha256sum "${path}")" || return 1
  elif command -v shasum >/dev/null 2>&1; then
    output="$(shasum -a 256 "${path}")" || return 1
  else
    echo "ERROR: sha256sum or shasum is required" >&2
    return 1
  fi
  printf '%s\n' "${output%% *}"
}

function sudo_sha256_file() {
  local path="$1"
  local output

  if command -v sha256sum >/dev/null 2>&1; then
    output="$(sudo sha256sum "${path}")" || return 1
  elif command -v shasum >/dev/null 2>&1; then
    output="$(sudo shasum -a 256 "${path}")" || return 1
  else
    echo "ERROR: sha256sum or shasum is required" >&2
    return 1
  fi
  printf '%s\n' "${output%% *}"
}

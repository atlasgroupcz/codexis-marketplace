#!/usr/bin/env bash

function install_executable() {
  local source_path="$1"
  local target_path="$2"
  local display_name="${3:-$(basename "${target_path}")}"
  local target_dir
  local tmp_path
  local -a elevate=()

  target_dir="$(dirname "${target_path}")"
  tmp_path="${target_dir}/.$(basename "${target_path}").tmp.$$"
  if [[ ! -f "${source_path}" || ! -s "${source_path}" ]]; then
    echo "ERROR: Missing or empty executable ${source_path}" >&2
    return 1
  fi
  if executable_matches "${source_path}" "${target_path}"; then
    echo "Verified ${display_name} -> ${target_path}"
    return 0
  fi

  if ! mkdir -p "${target_dir}" 2>/dev/null || [[ ! -w "${target_dir}" ]]; then
    elevate=(sudo)
    if ! sudo install -d "${target_dir}"; then
      echo "ERROR: Cannot create executable directory ${target_dir}" >&2
      return 1
    fi
  fi

  "${elevate[@]}" rm -f "${tmp_path}"
  if ! "${elevate[@]}" install -m 0755 "${source_path}" "${tmp_path}"; then
    echo "ERROR: Cannot stage executable ${target_path}" >&2
    return 1
  fi
  if ! executable_matches "${source_path}" "${tmp_path}" "${elevate[@]}"; then
    "${elevate[@]}" rm -f "${tmp_path}"
    echo "ERROR: Staged executable failed integrity verification ${target_path}" >&2
    return 1
  fi
  "${elevate[@]}" sync "${tmp_path}" 2>/dev/null || "${elevate[@]}" sync
  if ! "${elevate[@]}" mv -f "${tmp_path}" "${target_path}"; then
    "${elevate[@]}" rm -f "${tmp_path}"
    echo "ERROR: Cannot publish executable ${target_path}" >&2
    return 1
  fi

  if ! executable_matches "${source_path}" "${target_path}" "${elevate[@]}"; then
    echo "ERROR: Installed executable failed integrity verification ${target_path}" >&2
    return 1
  fi
  echo "Installed ${display_name} -> ${target_path}"
}

function executable_matches() {
  local source_path="$1"
  local target_path="$2"
  local source_checksum
  local target_checksum
  shift 2

  "${@}" test -f "${target_path}" &&
    "${@}" test -s "${target_path}" &&
    "${@}" test -x "${target_path}" || return 1
  source_checksum="$(sha256_file "${source_path}")" || return 1
  target_checksum="$(sha256_file "${target_path}" "${@}")" || return 1
  [[ "${source_checksum}" == "${target_checksum}" ]]
}

function sha256_file() {
  local path="$1"
  local output
  shift

  if command -v sha256sum >/dev/null 2>&1; then
    output="$("${@}" sha256sum "${path}")" || return 1
  elif command -v shasum >/dev/null 2>&1; then
    output="$("${@}" shasum -a 256 "${path}")" || return 1
  else
    echo "ERROR: sha256sum or shasum is required" >&2
    return 1
  fi
  printf '%s\n' "${output%% *}"
}

#!/usr/bin/env bash

# Remove the documents plugin's shell entrypoints from the user-local bin dir.

SCRIPT_NAME=$(basename "$0")

function usage() {
    cat <<EOM

Remove document rendering, recalculation and restart-check commands from the user-local bin directory.

usage: ${SCRIPT_NAME} [options]

options:
    --target  <dir>   Install target (default: \$TARGET_BIN_DIR, else \$HOME/.local/bin)
    -h|--help         Show this help

EOM
    exit 1
}

function main() {
    local target="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

    while [ "$1" != "" ]; do
        case $1 in
        --target)    shift; target="$1" ;;
        -h | --help) usage ;;
        *)           echo "Error: unknown option '$1'" >&2; usage ;;
        esac
        shift
    done

    rm -f "${target}/cdx-render" "${target}/cdx-recalc" "${target}/cdx-check-docx-restarts"
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
    exit 0
fi

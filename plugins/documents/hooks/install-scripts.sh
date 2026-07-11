#!/usr/bin/env bash

# Install the documents plugin's shell entrypoints into the user-local bin dir.
# The guest VM has ${HOME}/.local/bin on PATH, so no sudo and no system writes.

DEPENDENCIES=(install)
SCRIPT_NAME=$(basename "$0")

function usage() {
    cat <<EOM

Install cdx-render and cdx-recalc into the user-local bin directory.

usage: ${SCRIPT_NAME} [options]

options:
    --plugin-dir  <dir>    Plugin root (default: \$PLUGIN_DIR, else two levels up)
    --target      <dir>    Install target (default: \$TARGET_BIN_DIR, else \$HOME/.local/bin)
    -h|--help              Show this help

examples:
    ${SCRIPT_NAME}
    TARGET_BIN_DIR=/usr/local/bin ${SCRIPT_NAME}

EOM
    exit 1
}

function main() {
    local plugin_dir="${PLUGIN_DIR:-}"
    local target="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

    while [ "$1" != "" ]; do
        case $1 in
        --plugin-dir) shift; plugin_dir="$1" ;;
        --target)     shift; target="$1" ;;
        -h | --help)  usage ;;
        *)            echo "Error: unknown option '$1'" >&2; usage ;;
        esac
        shift
    done

    if [ -z "$plugin_dir" ]; then
        plugin_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd) || {
            echo "Error: cannot resolve the plugin directory" >&2
            exit 1
        }
    fi

    exit_on_missing_tools "${DEPENDENCIES[@]}"

    if ! install -d "$target"; then
        echo "Error: cannot create ${target}" >&2
        exit 1
    fi

    install_script "${plugin_dir}/shared/scripts/render.sh" "${target}/cdx-render" || exit 1
    install_script "${plugin_dir}/shared/scripts/recalc.sh" "${target}/cdx-recalc" || exit 1

    warn_on_missing_runtime
}

function install_script() {
    local source="$1" dest="$2"

    if [ ! -f "$source" ]; then
        echo "Error: missing ${source}" >&2
        return 1
    fi
    if ! install -m 0755 "$source" "$dest"; then
        echo "Error: cannot install ${dest}" >&2
        return 1
    fi
    echo "installed ${dest}"
}

# The plugin is useless without a renderer; say so at install time rather than
# letting the model discover it mid-task.
function warn_on_missing_runtime() {
    for cmd in soffice pdftoppm; do
        command -v "$cmd" &>/dev/null && continue
        printf "Warning: '%s' is not on PATH — document rendering will not work.\n" "$cmd" >&2
        printf "         The guest image must provide LibreOffice and poppler-utils.\n" >&2
    done
}

function exit_on_missing_tools() {
    for cmd in "$@"; do
        if command -v "$cmd" &>/dev/null; then
            continue
        fi
        printf "Error: required tool '%s' is not installed or not in PATH\n" "$cmd" >&2
        exit 1
    done
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
    exit 0
fi

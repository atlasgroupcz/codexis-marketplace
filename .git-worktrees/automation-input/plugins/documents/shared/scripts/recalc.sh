#!/usr/bin/env bash

# Recalculate an .xlsx written by a library (ExcelJS, openpyxl, POI) and report
# formula errors. Libraries store formulas as strings without cached results, so
# a workbook can look fine and still be full of #REF! when the user opens it.
# LibreOffice recalculates on load and caches the values back into the file.

DEPENDENCIES=(soffice unzip)
SCRIPT_NAME=$(basename "$0")
VERSION="1.0.0"

function usage() {
    cat <<EOM

Recalculate formulas in an .xlsx and report error cells as JSON.

usage: ${SCRIPT_NAME} -i <file.xlsx> [--in-place]

options:
    -i|--input     <file>    Workbook to recalculate (required)
    --in-place               Overwrite the input with the recalculated workbook
                             (default: leaves the recalculated copy in a temp dir)
    -h|--help                Show this help

dependencies: ${DEPENDENCIES[*]}

output: JSON on stdout, e.g.
    {"status":"ok","errors":0}
    {"status":"errors_found","errors":3,"types":["#REF!","#DIV/0!"]}

exit codes: 0 = no formula errors, 1 = failure, 2 = formula errors found

examples:
    ${SCRIPT_NAME} -i model.xlsx --in-place

EOM
    exit 1
}

function main() {
    local input="" in_place=false

    while [ "$1" != "" ]; do
        case $1 in
        -i | --input) shift; input="$1" ;;
        --in-place)   in_place=true ;;
        --version)    echo "${SCRIPT_NAME} ${VERSION}"; exit 0 ;;
        -h | --help)  usage ;;
        *)            echo "Error: unknown option '$1'" >&2; usage ;;
        esac
        shift
    done

    if [ -z "$input" ]; then
        echo "Error: --input is required" >&2
        usage
    fi
    if [ ! -f "$input" ]; then
        echo "Error: no such file: ${input}" >&2
        exit 1
    fi

    exit_on_missing_tools "${DEPENDENCIES[@]}"

    local workdir
    workdir=$(mktemp -d) || exit 1
    trap 'rm -rf "$workdir"' EXIT

    local recalculated
    recalculated=$(recalculate "$input" "$workdir") || exit 1

    if [ "$in_place" = true ]; then
        if ! cp "$recalculated" "$input"; then
            echo "Error: cannot overwrite ${input}" >&2
            exit 1
        fi
        recalculated="$input"
    fi

    report_errors "$recalculated"
}

function recalculate() {
    local input="$1" workdir="$2"
    local log basename

    if ! log=$(soffice -env:UserInstallation="file:///tmp/lo_profile_$$" \
        --headless --convert-to xlsx --outdir "$workdir" "$input" 2>&1); then
        echo "Error: LibreOffice recalculation failed: ${log}" >&2
        return 1
    fi

    basename=$(basename "$input")
    basename="${basename%.*}"

    local out="${workdir}/${basename}.xlsx"
    if [ ! -f "$out" ]; then
        echo "Error: expected ${out}, LibreOffice said: ${log}" >&2
        return 1
    fi
    echo "$out"
}

# Cached error values land in the sheet XML as <c t="e"><v>#REF!</v></c>.
function report_errors() {
    local workbook="$1"
    local errors types

    errors=$(unzip -p "$workbook" 'xl/worksheets/*.xml' 2>/dev/null |
        grep -o '#[A-Z0-9/]*[!?]' | sort | uniq -c)

    if [ -z "$errors" ]; then
        echo '{"status":"ok","errors":0}'
        return 0
    fi

    local count
    count=$(echo "$errors" | awk '{sum += $1} END {print sum}')
    types=$(echo "$errors" | awk '{printf "\"%s\",", $2}' | sed 's/,$//')

    echo "{\"status\":\"errors_found\",\"errors\":${count},\"types\":[${types}]}"
    return 2
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
    exit $?
fi

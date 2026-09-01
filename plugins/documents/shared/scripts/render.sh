#!/usr/bin/env bash

# Render any document (pdf/docx/xlsx/pptx/odt/html) to per-page PNG images so the
# AI can visually inspect layout, charts and tables that text extraction destroys.

DEPENDENCIES=(soffice pdftoppm)
SCRIPT_NAME=$(basename "$0")
VERSION="1.1.0"
WORKDIR=""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [[ -n "${NO_COLOR:-}" ]] || [[ "${TERM:-}" == "dumb" ]]; then
    RED="" GREEN="" YELLOW="" NC=""
fi

function usage() {
    cat <<EOM

Render a document to one PNG per page for visual inspection.

usage: ${SCRIPT_NAME} -i <file> [-o <dir>] [-r <dpi>] [-f <n>] [-l <n>]

options:
    -i|--input     <file>    Document to render (required)
    -o|--output    <dir>     Output directory (default: <input-dir>/.tmp/<basename>-pages)
    -r|--dpi       <n>       Resolution (default: 150 - readable, cheap on tokens)
    -f|--first     <n>       First page to render
    -l|--last      <n>       Last page to render
    -h|--help                Show this help

dependencies: ${DEPENDENCIES[*]}
optional: pngcheck (pages are verified when it is installed)

output: prints the output directory, then one image path per line.

Pages are rendered into a staging directory and moved into place only after all
of them render, so an interrupted run leaves no partial page behind.

examples:
    ${SCRIPT_NAME} -i /home/codexis/smlouva.docx
    ${SCRIPT_NAME} -i report.pdf -f 3 -l 3 -o /tmp/recheck

EOM
    exit 1
}

function main() {
    local input="" outdir="" dpi=150 first="" last=""

    while [ "$1" != "" ]; do
        case $1 in
        -i | --input)  shift; input="$1" ;;
        -o | --output) shift; outdir="$1" ;;
        -r | --dpi)    shift; dpi="$1" ;;
        -f | --first)  shift; first="$1" ;;
        -l | --last)   shift; last="$1" ;;
        --version)     echo "${SCRIPT_NAME} ${VERSION}"; exit 0 ;;
        -h | --help)   usage ;;
        *)             echo "Error: unknown option '$1'" >&2; usage ;;
        esac
        shift
    done

    if [ -z "$input" ]; then
        echo -e "${RED}Error: --input is required${NC}" >&2
        usage
    fi
    if [ ! -f "$input" ]; then
        echo -e "${RED}Error: no such file: ${input}${NC}" >&2
        exit 1
    fi

    exit_on_missing_tools "${DEPENDENCIES[@]}"

    local basename ext
    basename=$(basename "$input")
    basename="${basename%.*}"
    ext="${input##*.}"

    if [ -z "$outdir" ]; then
        outdir="$(cd "$(dirname "$input")" && pwd)/.tmp/${basename}-pages"
    fi
    mkdir -p "$outdir" || {
        echo -e "${RED}Error: cannot create ${outdir}${NC}" >&2
        exit 1
    }

    # Staged inside the output directory so publishing is a same-filesystem
    # rename. A staging area under /tmp would make it a copy, which is exactly
    # the half-written file this is meant to prevent.
    # Global, not local: the EXIT trap runs after main has returned, so a local
    # would already be out of scope and the staging directory would survive.
    WORKDIR=$(mktemp -d "${outdir}/.render-XXXXXX") || {
        echo -e "${RED}Error: cannot create a staging directory under ${outdir}${NC}" >&2
        exit 1
    }
    trap discard_staging EXIT

    local pdf
    if [ "$(to_lower "$ext")" = "pdf" ]; then
        pdf="$input"
    else
        pdf=$(convert_to_pdf "$input" "$WORKDIR") || exit 1
    fi

    rasterize "$pdf" "$WORKDIR" "$dpi" "$first" "$last" || exit 1
    verify_pages "$WORKDIR" || exit 1
    publish_pages "$WORKDIR" "$outdir" || exit 1

    echo -e "${GREEN}${outdir}${NC}"
    find "$outdir" -maxdepth 1 -name 'page-*.png' | sort
}

function convert_to_pdf() {
    local input="$1" outdir="$2"
    local log

    # LibreOffice hangs on a shared user profile in sandboxes; give every run its own.
    if ! log=$(soffice -env:UserInstallation="file:///tmp/lo_profile_$$" \
        --headless --convert-to pdf --outdir "$outdir" "$input" 2>&1); then
        echo -e "${RED}Error: LibreOffice conversion failed: ${log}${NC}" >&2
        return 1
    fi

    local basename
    basename=$(basename "$input")
    basename="${basename%.*}"

    local pdf="${outdir}/${basename}.pdf"
    if [ ! -f "$pdf" ]; then
        echo -e "${RED}Error: expected ${pdf}, LibreOffice said: ${log}${NC}" >&2
        return 1
    fi
    echo "$pdf"
}

function rasterize() {
    local pdf="$1" outdir="$2" dpi="$3" first="$4" last="$5"
    local args=(-png -r "$dpi")

    [ -n "$first" ] && args+=(-f "$first")
    [ -n "$last" ] && args+=(-l "$last")

    if ! pdftoppm "${args[@]}" "$pdf" "${outdir}/page"; then
        echo -e "${RED}Error: pdftoppm failed on ${pdf}${NC}" >&2
        return 1
    fi
}

function verify_pages() {
    local workdir="$1"
    local page

    if ! command -v pngcheck &>/dev/null; then
        echo -e "${YELLOW}Warning: pngcheck not installed, pages were not verified${NC}" >&2
        return 0
    fi

    for page in "$workdir"/page-*.png; do
        [ -e "$page" ] || continue
        if ! pngcheck -q "$page"; then
            echo -e "${RED}Error: ${page##*/} is incomplete or corrupt, discarding the render${NC}" >&2
            return 1
        fi
    done
}

function publish_pages() {
    local workdir="$1" outdir="$2"
    local page

    # A shorter re-render must not leave the previous run's extra pages behind.
    rm -f "$outdir"/page-*.png

    for page in "$workdir"/page-*.png; do
        [ -e "$page" ] || continue
        if ! mv "$page" "$outdir/"; then
            echo -e "${RED}Error: cannot move ${page##*/} into ${outdir}${NC}" >&2
            return 1
        fi
    done
}

function discard_staging() {
    [ -n "$WORKDIR" ] && rm -rf "$WORKDIR"
}

function to_lower() {
    echo "$1" | tr '[:upper:]' '[:lower:]'
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

#!/usr/bin/env bash

DEPENDENCIES=(docker git mktemp tar)
SCRIPT_NAME=$(basename "$0")
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
BUILD_CONTEXT=""

function usage() {
    cat <<EOM

Build a VM-mounted OCI volume from a committed marketplace distribution checkout.

usage: ${SCRIPT_NAME} --source <dist-checkout> --tag <oci-image>

options:
    --source <dir>    Clean Git checkout of the public marketplace distribution
    --tag <image>     OCI image name and tag to build
    -h|--help         Show this help message

dependencies: ${DEPENDENCIES[*]}

example:
    ${SCRIPT_NAME} --source ./dist --tag registry.agrp.dev/legal-machine/codexis-marketplace-volume:latest

EOM
    exit "${1:-1}"
}

function main() {
    local source_dir=""
    local image_tag=""
    local revision=""

    while [ "$#" -gt 0 ]; do
        case "$1" in
        --source)
            shift
            [ "$#" -gt 0 ] || usage
            source_dir="$1"
            ;;
        --tag)
            shift
            [ "$#" -gt 0 ] || usage
            image_tag="$1"
            ;;
        -h | --help)
            usage 0
            ;;
        *)
            printf "Error: Unknown option '%s'\n" "$1" >&2
            usage
            ;;
        esac
        shift
    done

    [ -n "$source_dir" ] || usage
    [ -n "$image_tag" ] || usage
    exit_on_missing_tools "${DEPENDENCIES[@]}"

    source_dir=$(cd "$source_dir" 2>/dev/null && pwd) || {
        printf "Error: Marketplace distribution directory is unavailable: %s\n" "$source_dir" >&2
        exit 1
    }

    validate_source "$source_dir"
    revision=$(git -C "$source_dir" rev-parse HEAD) || {
        printf "Error: Cannot resolve marketplace distribution revision\n" >&2
        exit 1
    }

    create_build_context "$source_dir" "$revision"
    build_image "$image_tag" "$revision"
    validate_image "$image_tag" "$revision"

    printf 'CODEXIS_MARKETPLACE_VOLUME_IMAGE=%s\n' "$image_tag"
    printf 'CODEXIS_MARKETPLACE_VOLUME_REVISION=%s\n' "$revision"
}

function validate_source() {
    local source_dir="$1"
    local status=""

    git -C "$source_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1 || {
        printf "Error: Marketplace distribution is not a Git checkout: %s\n" "$source_dir" >&2
        exit 1
    }

    status=$(git -C "$source_dir" status --porcelain) || {
        printf "Error: Cannot inspect marketplace distribution checkout\n" >&2
        exit 1
    }
    if [ -n "$status" ]; then
        printf "Error: Marketplace distribution checkout must be clean\n%s\n" "$status" >&2
        exit 1
    fi

    git -C "$source_dir" cat-file -e HEAD:.claude-plugin/marketplace.json 2>/dev/null || {
        printf "Error: Marketplace distribution revision has no marketplace manifest\n" >&2
        exit 1
    }
}

function create_build_context() {
    local source_dir="$1"
    local revision="$2"
    local archive_path=""

    BUILD_CONTEXT=$(mktemp -d "${TMPDIR:-/tmp}/codexis-marketplace-volume.XXXXXX") || {
        printf "Error: Cannot create temporary build context\n" >&2
        exit 1
    }
    trap cleanup EXIT

    mkdir -p "$BUILD_CONTEXT/marketplace" || {
        printf "Error: Cannot create marketplace build context\n" >&2
        exit 1
    }
    archive_path="$BUILD_CONTEXT/marketplace.tar"
    git -C "$source_dir" archive --format=tar --output="$archive_path" HEAD || {
        printf "Error: Cannot archive marketplace distribution\n" >&2
        exit 1
    }
    tar -xf "$archive_path" -C "$BUILD_CONTEXT/marketplace" || {
        printf "Error: Cannot extract marketplace distribution archive\n" >&2
        exit 1
    }
    rm -f "$archive_path" || {
        printf "Error: Cannot remove temporary marketplace archive\n" >&2
        exit 1
    }
    printf '%s\n' "$revision" >"$BUILD_CONTEXT/marketplace/.codexis-marketplace-revision" || {
        printf "Error: Cannot write marketplace revision marker\n" >&2
        exit 1
    }
}

function build_image() {
    local image_tag="$1"
    local revision="$2"

    docker build \
        --build-arg "MARKETPLACE_REVISION=$revision" \
        --file "$ROOT_DIR/Dockerfile.volume" \
        --tag "$image_tag" \
        "$BUILD_CONTEXT" || {
        printf "Error: Cannot build marketplace volume image\n" >&2
        exit 1
    }
}

function validate_image() {
    local image_tag="$1"
    local revision="$2"
    local actual_revision=""

    actual_revision=$(docker image inspect \
        --format '{{ index .Config.Labels "org.opencontainers.image.revision" }}' \
        "$image_tag") || {
        printf "Error: Cannot inspect marketplace volume image\n" >&2
        exit 1
    }
    if [ "$actual_revision" != "$revision" ]; then
        printf "Error: Marketplace volume image revision mismatch: %s\n" "$actual_revision" >&2
        exit 1
    fi
}

function exit_on_missing_tools() {
    local command_name=""

    for command_name in "$@"; do
        if command -v "$command_name" >/dev/null 2>&1; then
            continue
        fi
        printf "Error: Required tool '%s' is not installed or not in PATH\n" "$command_name" >&2
        exit 1
    done
}

function cleanup() {
    if [ -n "$BUILD_CONTEXT" ] && [ -d "$BUILD_CONTEXT" ]; then
        rm -rf -- "$BUILD_CONTEXT"
    fi
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
    exit 0
fi

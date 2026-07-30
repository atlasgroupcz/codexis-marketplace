#!/usr/bin/env bash
# Shared build logic for the marketplace frontend components. Sourced by each
# plugin's `components/build.sh`. Build-time only - never shipped (`shared/` is
# dist-excluded).
#
# `frontend/apps/<app>/` is the source; what the daemon serves is the built copy
# committed under `plugins/<plugin>/components/<app>/`. These helpers build one
# app in a pinned bun image and sync its `dist/` into that directory.
#
# Unlike the Rust side there is nothing to share between apps at build time -
# each `bun run build` produces its own bundle - so this file holds no one-pass
# equivalent, only the config and the two steps every plugin repeated verbatim.
#
# Usage from a plugin's components/build.sh:
#   set -euo pipefail
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "$SCRIPT_DIR/../../../shared/scripts/component-build.sh"
#   build_component <app>              # builds, then syncs into ./<app>
#   build_component <app> <component-dir>   # when the two names differ
#
# Override before `source` when needed:
#   BUN_IMAGE   Docker image (default: pinned oven/bun)

set -euo pipefail

case "$(uname -m)" in
  x86_64|amd64) _CDX_DOCKER_ARCH="amd64" ;;
  aarch64|arm64) _CDX_DOCKER_ARCH="arm64" ;;
  *) echo "ERROR: Unsupported CPU architecture: $(uname -m)" >&2; exit 1 ;;
esac

# Repo root, derived from this file's own location (shared/scripts/ -> ../..).
_CDX_COMPONENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$_CDX_COMPONENT_DIR/../.." && pwd)"

# The sourcing script - always a plugin's components/build.sh, and always the
# parent of the component directory it writes. Captured here rather than passed
# in: every caller would otherwise repeat `"$SCRIPT_DIR/<app>"`, which restates
# that invariant nine times instead of holding it, and leaves room to sync one
# plugin's build into another's directory.
_CDX_CALLER_DIR="$(cd "$(dirname "${BASH_SOURCE[1]}")" && pwd)"
FRONTEND_DIR="$REPO_ROOT/frontend"

BUN_IMAGE="${BUN_IMAGE:-oven/bun:1.3.9-slim}"
_CDX_DOCKER_PLATFORM="linux/${_CDX_DOCKER_ARCH}"

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker is required to build plugin frontend components." >&2
  exit 1
fi

if [[ ! -d "$FRONTEND_DIR" ]]; then
  echo "ERROR: Missing frontend workspace at $FRONTEND_DIR" >&2
  exit 1
fi

# Build one app and sync it into this plugin's component directory.
#
# The build runs as the invoking user, or dist/ and the synced output end up
# root-owned and the host user cannot even delete them.
#
# The target holds authored files too - component.json, the icon, the CGI binary
# - so it is never wiped wholesale. Only what the build owns is cleared first:
# index.html, locales/, and everything under assets/, whose filenames carry a
# content hash and would otherwise pile up one set per build.
#
# Authored files live at the component root, outside that blast radius. They used
# to sit in assets/ and needed an explicit preserve list; the list stayed on for
# months after they moved out, protecting nothing.
build_component() {
  local app="$1"
  local target_dir="$_CDX_CALLER_DIR/${2:-$app}"
  local dist_dir="$FRONTEND_DIR/apps/$app/dist"

  if [[ ! -d "$FRONTEND_DIR/apps/$app" ]]; then
    echo "ERROR: No such frontend app: $FRONTEND_DIR/apps/$app" >&2
    exit 1
  fi

  echo "==> Building frontend app '$app' in Docker ($_CDX_DOCKER_PLATFORM)"
  docker run --rm \
    --platform "$_CDX_DOCKER_PLATFORM" \
    -u "$(id -u):$(id -g)" \
    -v "$REPO_ROOT:/workspace" \
    -w /workspace/frontend \
    "$BUN_IMAGE" \
    bash -lc "
      set -euo pipefail
      bun install --frozen-lockfile
      cd apps/$app
      bun run build
    "

  if [[ ! -d "$dist_dir" ]]; then
    echo "ERROR: Missing built frontend output: $dist_dir" >&2
    exit 1
  fi

  mkdir -p "$target_dir/assets"
  rm -f "$target_dir/index.html"
  rm -rf "$target_dir/locales"
  find "$target_dir/assets" -mindepth 1 -delete

  cp -a "$dist_dir"/. "$target_dir"/
  echo "   synced -> ${target_dir#"$REPO_ROOT"/}"
}

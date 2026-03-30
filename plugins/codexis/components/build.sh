#!/usr/bin/env bash
set -euo pipefail

case "$(uname -m)" in
  x86_64|amd64)
    DOCKER_ARCH="amd64"
    ;;
  aarch64|arm64)
    DOCKER_ARCH="arm64"
    ;;
  *)
    echo "ERROR: Unsupported CPU architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$PLUGIN_DIR/../.." && pwd)"
FRONTEND_DIR="$REPO_ROOT/frontend"

DOCKER_PLATFORM="linux/${DOCKER_ARCH}"
BUN_IMAGE="oven/bun:1.3.9-slim"

if ! command -v docker >/dev/null 2>&1; then
  echo "ERROR: docker is required to build plugin frontend components." >&2
  exit 1
fi

if [[ ! -d "$FRONTEND_DIR" ]]; then
  echo "ERROR: Missing frontend workspace at $FRONTEND_DIR" >&2
  exit 1
fi

build_frontend_apps() {
  echo "==> Building frontend apps in Docker ($DOCKER_PLATFORM)"
  docker run --rm \
    --platform "$DOCKER_PLATFORM" \
    -u "$(id -u):$(id -g)" \
    -v "$REPO_ROOT:/workspace" \
    -w /workspace/frontend \
    "$BUN_IMAGE" \
    bash -lc '
      set -euo pipefail
      bun install --frozen-lockfile
      cd apps/codexis:sledovane-dokumenty
      bun run build
      cd /workspace/frontend/apps/codexis:sledovana-judikatura
      bun run build
    '
}

sync_component_output() {
  local dist_dir="$1"
  local target_dir="$2"
  shift 2
  local -a preserved_assets=("$@")

  if [[ ! -d "$dist_dir" ]]; then
    echo "ERROR: Missing built frontend output: $dist_dir" >&2
    exit 1
  fi

  mkdir -p "$target_dir/assets"
  rm -f "$target_dir/index.html"
  rm -rf "$target_dir/locales"

  while IFS= read -r -d '' path; do
    rm -rf "$path"
  done < <(find "$target_dir/assets" -mindepth 1 -maxdepth 1 -type d -print0)

  while IFS= read -r -d '' path; do
    local base preserve=false
    base="$(basename "$path")"

    for preserved in "${preserved_assets[@]}"; do
      if [[ "$base" == "$preserved" ]]; then
        preserve=true
        break
      fi
    done

    if [[ "$preserve" == false ]]; then
      rm -f "$path"
    fi
  done < <(find "$target_dir/assets" -mindepth 1 -maxdepth 1 -type f -print0)

  cp -a "$dist_dir"/. "$target_dir"/
  echo "   synced -> ${target_dir#$REPO_ROOT/}"
}

build_frontend_apps

sync_component_output \
  "$FRONTEND_DIR/apps/codexis:sledovane-dokumenty/dist" \
  "$SCRIPT_DIR/sledovane-dokumenty" \
  "banner.png" \
  "icon.png" \
  "logo.svg" \
  "styles.css"

sync_component_output \
  "$FRONTEND_DIR/apps/codexis:sledovana-judikatura/dist" \
  "$SCRIPT_DIR/sledovana-judikatura" \
  "icon.png"

install -m 0644 \
  "$SCRIPT_DIR/sledovane-dokumenty/assets/icon.png" \
  "$SCRIPT_DIR/sledovana-judikatura/assets/icon.png"

echo "Completed component build: $(basename "$PLUGIN_DIR")"

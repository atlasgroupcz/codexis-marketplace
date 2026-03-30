#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PARENT_DIR="$(dirname "$ROOT_DIR")"
REPO_NAME="$(basename "$ROOT_DIR")"
DIST_DIR="$PARENT_DIR/${REPO_NAME}-dist"
EXCLUDES_FILE="$ROOT_DIR/dist-exclusions.txt"
DIST_CONTENT_DIR="$ROOT_DIR/dist-content"

echo "Creating distribution copy at: $DIST_DIR"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

if [[ ! -f "$EXCLUDES_FILE" ]]; then
  echo "ERROR: Missing excludes file: $EXCLUDES_FILE" >&2
  exit 1
fi

rsync -a --delete --delete-excluded --exclude-from="$EXCLUDES_FILE" "$ROOT_DIR"/ "$DIST_DIR"/

if [[ -d "$DIST_CONTENT_DIR" ]]; then
  echo "Applying dist-content overlay from: $DIST_CONTENT_DIR"
  rsync -a --exclude='.gitkeep' "$DIST_CONTENT_DIR"/ "$DIST_DIR"/
fi

echo "Distribution repository created successfully."

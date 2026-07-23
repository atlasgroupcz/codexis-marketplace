#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_NAME="$(basename "$ROOT_DIR")"
DIST_DIR="$ROOT_DIR/dist"
EXCLUDES_FILE="$ROOT_DIR/dist-exclusions.txt"
DIST_CONTENT_DIR="$ROOT_DIR/dist-content"
DIST_REMOTE_URL="${DIST_REMOTE_URL:-git@github.com:atlasgroupcz/codexis-marketplace.git}"
TEMP_DIST_DIR="$(mktemp -d)"

cleanup() {
  rm -rf "$TEMP_DIST_DIR"
}

resolve_dist_branch() {
  local remote_head_ref

  if [[ -n "${DIST_BRANCH:-}" ]]; then
    printf '%s\n' "$DIST_BRANCH"
    return
  fi

  remote_head_ref="$(git ls-remote --symref "$DIST_REMOTE_URL" HEAD 2>/dev/null | awk '/^ref:/ {sub("refs/heads/", "", $2); print $2; exit}')"

  if [[ -n "$remote_head_ref" ]]; then
    printf '%s\n' "$remote_head_ref"
    return
  fi

  printf 'main\n'
}

prepare_dist_repo() {
  local dist_branch="$1"

  echo "Preparing distribution repository at: $DIST_DIR"
  rm -rf "$DIST_DIR"

  if git ls-remote --exit-code --heads "$DIST_REMOTE_URL" "$dist_branch" >/dev/null 2>&1; then
    git clone --branch "$dist_branch" --single-branch "$DIST_REMOTE_URL" "$DIST_DIR"
    return
  fi

  mkdir -p "$DIST_DIR"
  git -C "$DIST_DIR" init --quiet
  git -C "$DIST_DIR" checkout -b "$dist_branch" >/dev/null 2>&1
  git -C "$DIST_DIR" remote add origin "$DIST_REMOTE_URL"
}

commit_dist_changes() {
  local dist_branch="$1"
  local source_branch
  local source_rev
  local source_dirty=""
  local commit_message

  source_branch="$(git -C "$ROOT_DIR" branch --show-current 2>/dev/null || true)"
  source_rev="$(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || printf 'unknown')"

  if [[ -n "$(git -C "$ROOT_DIR" status --short 2>/dev/null)" ]]; then
    source_dirty='-dirty'
  fi

  commit_message="${DIST_COMMIT_MESSAGE:-Sync dist from ${REPO_NAME} (${source_branch:-detached}@${source_rev}${source_dirty})}"

  git -C "$DIST_DIR" add --all

  if git -C "$DIST_DIR" diff --cached --quiet --exit-code; then
    echo "No distribution changes to commit."
    return
  fi

  git -C "$DIST_DIR" commit -m "$commit_message"
  git -C "$DIST_DIR" push --set-upstream origin "$dist_branch"
}

trap cleanup EXIT

DIST_BRANCH="$(resolve_dist_branch)"

echo "Creating distribution staging copy at: $TEMP_DIST_DIR"

if [[ ! -f "$EXCLUDES_FILE" ]]; then
  echo "ERROR: Missing excludes file: $EXCLUDES_FILE" >&2
  exit 1
fi

rsync -a --delete --delete-excluded --exclude-from="$EXCLUDES_FILE" "$ROOT_DIR"/ "$TEMP_DIST_DIR"/

if [[ -d "$DIST_CONTENT_DIR" ]]; then
  echo "Applying dist-content overlay from: $DIST_CONTENT_DIR"
  rsync -a --exclude='.gitkeep' "$DIST_CONTENT_DIR"/ "$TEMP_DIST_DIR"/
fi

prepare_dist_repo "$DIST_BRANCH"
rsync -a --delete --exclude='.git' "$TEMP_DIST_DIR"/ "$DIST_DIR"/
commit_dist_changes "$DIST_BRANCH"

echo "Distribution repository created successfully at: $DIST_DIR"

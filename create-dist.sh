#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_NAME="$(basename "$ROOT_DIR")"
DIST_DIR="$ROOT_DIR/dist"
EXCLUDES_FILE="$ROOT_DIR/dist-exclusions.txt"
DIST_CONTENT_DIR="$ROOT_DIR/dist-content"
DIST_REMOTE_URL="${DIST_REMOTE_URL:-git@github.com:atlasgroupcz/codexis-marketplace.git}"
TEMP_DIST_DIR="$(mktemp -d)"
TEMP_TRACKED_DIR="$(mktemp -d)"
TEMP_FILE_LIST="$(mktemp)"

cleanup() {
  rm -rf "$TEMP_DIST_DIR" "$TEMP_TRACKED_DIR" "$TEMP_FILE_LIST"
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

  # `-dirty` says the dist cannot be reproduced from `source_rev` alone, but not
  # what was extra - and `git status --short` counts untracked files too, so a
  # single stray build directory earns it. Name them, or the next person reading
  # a dirty commit message has nothing to go on but guesses.
  local source_status
  source_status="$(git -C "$ROOT_DIR" status --short 2>/dev/null)"
  if [[ -n "$source_status" ]]; then
    source_dirty='-dirty'
    printf 'Source tree is dirty; marking the commit. Untracked or modified:\n%s\n' \
      "$source_status" >&2
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

# Seed the dist from git-tracked files only, in two passes:
#
#   Pass 1 materializes ONLY git-tracked files into a mirror. Anything git does
#   not track (target/, node_modules/, .venv, .turbo, worktree checkouts, ...)
#   is absent from the list, so no local build junk can leak, however dirty the
#   tree is. Files are read from the working tree, so freshly built binaries
#   under bin/ (tracked paths) ship as built. Note: rsync's --files-from
#   bypasses the exclude filters, so this pass copies every tracked file,
#   including the dev ones - pass 2 is what drops those.
#
#   Pass 2 copies that mirror into the staging dir applying the exclude file. A
#   plain recursive rsync DOES honor --exclude-from, so this drops the
#   tracked-but-dev files (src/, tests/, build.sh, Cargo.toml, uv.lock, ...).
git -C "$ROOT_DIR" ls-files -z > "$TEMP_FILE_LIST"
if [[ ! -s "$TEMP_FILE_LIST" ]]; then
  echo "ERROR: git ls-files listed no tracked files in $ROOT_DIR (not a git checkout?)." >&2
  echo "       Refusing to build an empty dist." >&2
  exit 1
fi

rsync -a --from0 --files-from="$TEMP_FILE_LIST" "$ROOT_DIR"/ "$TEMP_TRACKED_DIR"/
rsync -a --delete --delete-excluded --exclude-from="$EXCLUDES_FILE" "$TEMP_TRACKED_DIR"/ "$TEMP_DIST_DIR"/

if [[ -d "$DIST_CONTENT_DIR" ]]; then
  echo "Applying dist-content overlay from: $DIST_CONTENT_DIR"
  rsync -a --exclude='.gitkeep' "$DIST_CONTENT_DIR"/ "$TEMP_DIST_DIR"/
fi

prepare_dist_repo "$DIST_BRANCH"
rsync -a --delete --exclude='.git' "$TEMP_DIST_DIR"/ "$DIST_DIR"/
commit_dist_changes "$DIST_BRANCH"

echo "Distribution repository created successfully at: $DIST_DIR"

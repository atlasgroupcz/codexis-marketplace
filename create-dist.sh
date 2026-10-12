#!/usr/bin/env bash
# Build the marketplace distribution into dist/: what the marketplace ships, as
# a single commit of a fresh git repository. build-volume-image.sh packs that
# commit; shared/deb/build-debs.sh packages it.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_NAME="$(basename "$ROOT_DIR")"
DIST_DIR="$ROOT_DIR/dist"
EXCLUDES_FILE="$ROOT_DIR/dist-exclusions.txt"
DIST_CONTENT_DIR="$ROOT_DIR/dist-content"
TEMP_TRACKED_DIR="$(mktemp -d)"
TEMP_FILE_LIST="$(mktemp)"

cleanup() {
  rm -rf "$TEMP_TRACKED_DIR" "$TEMP_FILE_LIST"
}

commit_dist() {
  local source_branch
  local source_rev
  local source_dirty=""

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

  # The dist commit carries the source commit's time, so what is versioned from
  # it (the marketplace package) follows the source, not the hour it was built.
  local source_date
  source_date="$(git -C "$ROOT_DIR" log -1 --format=%cI)"

  git -C "$DIST_DIR" init --quiet --initial-branch=main
  git -C "$DIST_DIR" add --all
  GIT_AUTHOR_DATE="$source_date" GIT_COMMITTER_DATE="$source_date" \
    git -C "$DIST_DIR" commit --quiet \
    -m "Dist of ${REPO_NAME} (${source_branch:-detached}@${source_rev}${source_dirty})"
}

trap cleanup EXIT

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
#   Pass 2 copies that mirror into dist/ applying the exclude file. A plain
#   recursive rsync DOES honor --exclude-from, so this drops the tracked-but-dev
#   files (src/, tests/, build.sh, Cargo.toml, uv.lock, ...).
git -C "$ROOT_DIR" ls-files -z > "$TEMP_FILE_LIST"
if [[ ! -s "$TEMP_FILE_LIST" ]]; then
  echo "ERROR: git ls-files listed no tracked files in $ROOT_DIR (not a git checkout?)." >&2
  echo "       Refusing to build an empty dist." >&2
  exit 1
fi

echo "Creating distribution at: $DIST_DIR"
rm -rf "$DIST_DIR"
rsync -a --from0 --files-from="$TEMP_FILE_LIST" "$ROOT_DIR"/ "$TEMP_TRACKED_DIR"/
rsync -a --exclude-from="$EXCLUDES_FILE" "$TEMP_TRACKED_DIR"/ "$DIST_DIR"/

if [[ -d "$DIST_CONTENT_DIR" ]]; then
  echo "Applying dist-content overlay from: $DIST_CONTENT_DIR"
  rsync -a --exclude='.gitkeep' "$DIST_CONTENT_DIR"/ "$DIST_DIR"/
fi

commit_dist

echo "Distribution created at: $DIST_DIR ($(git -C "$DIST_DIR" rev-parse --short HEAD))"

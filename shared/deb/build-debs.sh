#!/usr/bin/env bash
# Build one .deb per marketplace plugin into target/deb/.
#
#   shared/deb/build-debs.sh              every plugin
#   shared/deb/build-debs.sh codexis sid  just these
#
# Packages the committed artifacts (bin/, components/) as they ship, so run
# ./build.sh first when the sources changed. Needs docker, git and rsync.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="$ROOT_DIR/target/deb"
# Ubuntu 26.04, the guest VM's release: dpkg-shlibdeps derives the library
# Depends from this image's packages. Pinned by digest, like the other builders.
DEB_IMAGE="buildpack-deps:resolute@sha256:c51212ca047bc2504e4e0fabfaee2fab439366ac8df87c71661b09a22a34d706"
STAGE_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGE_DIR"' EXIT

# The payload is exactly what create-dist.sh ships: git-tracked files only, then
# dist-exclusions.txt applied from the repo root, so anchored rules match there.
git -C "$ROOT_DIR" ls-files -z -- plugins > "$STAGE_DIR/files"
rsync -a --from0 --files-from="$STAGE_DIR/files" "$ROOT_DIR"/ "$STAGE_DIR/tracked"/
rsync -a --exclude-from="$ROOT_DIR/dist-exclusions.txt" \
  "$STAGE_DIR/tracked"/ "$STAGE_DIR/payload"/

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

# The plugin binaries are x86-64 (the guest runs nothing else), and
# dpkg-shlibdeps resolves their libraries only on a matching platform.
docker run --rm \
  --platform linux/amd64 \
  -u "$(id -u):$(id -g)" \
  -e SOURCE_DATE_EPOCH="$(git -C "$ROOT_DIR" log -1 --format=%ct)" \
  -v "$ROOT_DIR:/workspace:ro" \
  -v "$STAGE_DIR/payload/plugins:/payload:ro" \
  -v "$OUT_DIR:/out" \
  "$DEB_IMAGE" \
  python3 /workspace/shared/deb/debpkg.py \
    --payload /payload --meta /workspace/plugins --out /out "$@"

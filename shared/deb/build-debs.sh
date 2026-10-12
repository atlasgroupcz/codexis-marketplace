#!/usr/bin/env bash
# Build one .deb per plugin of a marketplace distribution into target/deb/, with
# a Packages index that makes the directory a flat apt repository.
#
#   shared/deb/build-debs.sh <dist> <namespace> [plugin...]
#
#   shared/deb/build-debs.sh dist codexis           every plugin and codexis-marketplace
#   shared/deb/build-debs.sh dist codexis sid ares  just these
#
# <dist> is a git checkout of a marketplace distribution, such as ./create-dist.sh
# writes; the time of its HEAD commit versions <namespace>-marketplace. The
# namespace prefixes the package names and names the directory under
# /usr/share/cdx-plugins the plugins install into. Needs docker and git.
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <dist> <namespace> [plugin...]" >&2
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="$(cd "$1" && pwd)"
NAMESPACE="$2"
shift 2
OUT_DIR="$ROOT_DIR/target/deb"
# Ubuntu 26.04, the guest VM's release: dpkg-shlibdeps derives the library
# Depends from this image's packages. Pinned by digest, like the other builders.
DEB_IMAGE="buildpack-deps:resolute@sha256:c51212ca047bc2504e4e0fabfaee2fab439366ac8df87c71661b09a22a34d706"
EPOCH="$(git -C "$DIST_DIR" log -1 --format=%ct)"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

# The plugin binaries are x86-64 (the guest runs nothing else), and
# dpkg-shlibdeps resolves their libraries only on a matching platform.
docker run --rm \
  --platform linux/amd64 \
  -u "$(id -u):$(id -g)" \
  -e SOURCE_DATE_EPOCH="$EPOCH" \
  -v "$ROOT_DIR/shared/deb:/builder:ro" \
  -v "$DIST_DIR:/marketplace:ro" \
  -v "$OUT_DIR:/out" \
  "$DEB_IMAGE" \
  python3 /builder/debpkg.py \
    --marketplace /marketplace --namespace "$NAMESPACE" --out /out "$@"

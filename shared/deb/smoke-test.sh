#!/usr/bin/env bash
# Install every package from target/deb/ into a clean Ubuntu 26.04 (the guest
# VM's release) and check that each stands on its own: apt resolves all of its
# Depends, every command it puts on PATH starts from /, none of its symlinks
# dangle, and purging leaves nothing behind. Run shared/deb/build-debs.sh first.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEB_DIR="$ROOT_DIR/target/deb"
IMAGE="ubuntu:26.04@sha256:f144425ff09be612d6d9ad965196e9cdc23dae1f42110a8a11a3e9a8198759f7"

if ! compgen -G "$DEB_DIR/*.deb" >/dev/null; then
  echo "ERROR: no packages in $DEB_DIR - run shared/deb/build-debs.sh first." >&2
  exit 1
fi

docker run --rm --platform linux/amd64 -v "$DEB_DIR:/debs:ro" "$IMAGE" bash -c '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq --no-install-recommends /debs/*.deb >/dev/null

failed=0
packages=$(dpkg-query -W -f "\${Package}\n" "codexis-*")
for package in $packages; do
  while IFS= read -r path; do
    if [ -L "$path" ] && [ ! -e "$path" ]; then
      echo "FAIL $package: dangling symlink $path"
      failed=1
    fi
    case "$path" in
      /usr/bin/?*)
        # Help only comes out once the interpreter, the shared libraries and
        # (for the Python commands) the plugin lib/ resolved. A few commands
        # print their usage with a non-zero exit, hence the second test.
        status=0
        output=$(cd / && "$path" --help 2>&1) || status=$?
        if [ "$status" -eq 0 ] || grep -qi usage <<<"$output"; then
          echo "ok   $path"
        else
          echo "FAIL $package: $path --help printed no usage:"
          echo "$output" | head -5
          failed=1
        fi
        ;;
    esac
  done < <(dpkg -L "$package")
done

apt-get purge -y -qq $packages >/dev/null
if [ -e /usr/share/cdx-plugins ]; then
  echo "FAIL: purge left files behind:"
  find /usr/share/cdx-plugins
  failed=1
fi
exit $failed
'

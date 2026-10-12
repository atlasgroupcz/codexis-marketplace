#!/usr/bin/env bash
# Install the whole marketplace from target/deb/ into a clean Ubuntu 26.04 (the
# guest VM's release) with one `apt-get install codexis-marketplace`, then check
# that every package stands on its own: apt resolves all of its Depends, every
# command it puts on PATH starts from /, every Python module it ships in lib/
# imports, none of its symlinks dangle, plugins.txt lists every plugin, and
# purging leaves nothing behind. Run shared/deb/build-debs.sh first.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEB_DIR="$ROOT_DIR/target/deb"
IMAGE="ubuntu:26.04@sha256:f144425ff09be612d6d9ad965196e9cdc23dae1f42110a8a11a3e9a8198759f7"

if ! compgen -G "$DEB_DIR/codexis-marketplace_*.deb" >/dev/null; then
  echo "ERROR: no codexis-marketplace in $DEB_DIR - run shared/deb/build-debs.sh first." >&2
  exit 1
fi

docker run --rm --platform linux/amd64 -v "$DEB_DIR:/debs:ro" "$IMAGE" bash -c '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
echo "deb [trusted=yes] file:/debs ./" > /etc/apt/sources.list.d/codexis.list
apt-get update -qq
apt-get install -y -qq --no-install-recommends codexis-marketplace >/dev/null

failed=0
fail() { echo "FAIL $*"; failed=1; }

built=$(ls /debs/*.deb | wc -l)
packages=$(dpkg-query -W -f "\${Package}\n" "codexis*")
installed=$(echo "$packages" | wc -l)
[ "$installed" -eq "$built" ] || fail "codexis-marketplace installed $installed of $built packages"

for package in $packages; do
  while IFS= read -r path; do
    if [ -L "$path" ] && [ ! -e "$path" ]; then
      fail "$package: dangling symlink $path"
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
          fail "$package: $path --help printed no usage:"
          echo "$output" | head -5
        fi
        ;;
    esac
  done < <(dpkg -L "$package")
done

root=/usr/share/cdx-plugins/codexis
for lib in "$root"/*/lib; do
  for module in "$lib"/*; do
    name=$(basename "$module" .py)
    [ -f "$module/__init__.py" ] || [ "${module%.py}" != "$module" ] || continue
    if (cd / && python3 -c "import sys; sys.path.insert(0, \"$lib\"); import $name") 2>/dev/null; then
      echo "ok   import $name"
    else
      fail "import $name from $lib"
    fi
  done
done

listed=$(wc -l < "$root/plugins.txt")
[ "$listed" -eq $((built - 1)) ] || fail "plugins.txt lists $listed plugins, expected $((built - 1))"
while IFS= read -r dir; do
  [ -f "$dir/.claude-plugin/plugin.json" ] || fail "plugins.txt names $dir, which is no plugin"
done < "$root/plugins.txt"

apt-get purge -y -qq $packages >/dev/null
if [ -e /usr/share/cdx-plugins ]; then
  fail "purge left files behind:"
  find /usr/share/cdx-plugins
fi
exit $failed
'

#!/usr/bin/env bash
# Install the whole marketplace from target/deb/ into a clean Ubuntu 26.04 (the
# guest VM's release) with one `apt-get install <namespace>-marketplace`, then
# check that every package stands on its own: apt resolves all of its Depends,
# every command it puts on PATH starts from /, every Python module it ships in
# lib/ imports, none of its symlinks dangle, its cdx-manifest.json names only
# files it ships, and purging leaves nothing behind. The installed directory
# must also read as the marketplace: its marketplace.json and revision, and each
# plugin where marketplace.json says it is.
#
#   shared/deb/smoke-test.sh <namespace>    after shared/deb/build-debs.sh
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <namespace>" >&2
  exit 2
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEB_DIR="$ROOT_DIR/target/deb"
NAMESPACE="$1"
IMAGE="ubuntu:26.04@sha256:f144425ff09be612d6d9ad965196e9cdc23dae1f42110a8a11a3e9a8198759f7"

if ! compgen -G "$DEB_DIR/$NAMESPACE-marketplace_*.deb" >/dev/null; then
  echo "ERROR: no $NAMESPACE-marketplace in $DEB_DIR - run shared/deb/build-debs.sh first." >&2
  exit 1
fi

docker run --rm --platform linux/amd64 -e NAMESPACE="$NAMESPACE" \
  -v "$DEB_DIR:/debs:ro" "$IMAGE" bash -c '
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
echo "deb [trusted=yes] file:/debs ./" > /etc/apt/sources.list.d/marketplace.list
apt-get update -qq
apt-get install -y -qq --no-install-recommends "$NAMESPACE-marketplace" >/dev/null

failed=0
fail() { echo "FAIL $*"; failed=1; }

built=$(ls /debs/*.deb | wc -l)
packages=$(dpkg-query -W -f "\${Package}\n" "$NAMESPACE*")
installed=$(echo "$packages" | wc -l)
[ "$installed" -eq "$built" ] || fail "$NAMESPACE-marketplace installed $installed of $built packages"

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

root="/usr/share/cdx-plugins/$NAMESPACE"
for lib in $(find "$root" -name cdx-manifest.json -printf "%h/lib\n"); do
  [ -d "$lib" ] || continue
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

python3 - "$root" $((built - 1)) <<"PY" || failed=1
import json
import re
import sys
from pathlib import Path

root, expected = Path(sys.argv[1]), int(sys.argv[2])
problems = []
revision = (root / ".codexis-marketplace-revision").read_text().strip()
if not re.fullmatch(r"[0-9a-f]{40,64}", revision):
    problems.append(f"revision {revision!r} is no commit")
manifests = list(root.rglob("cdx-manifest.json"))
if len(manifests) != expected:
    problems.append(f"{len(manifests)} plugins carry a cdx-manifest.json, expected {expected}")
marketplace = json.loads((root / ".claude-plugin/marketplace.json").read_text())
for entry in marketplace["plugins"]:
    name, source = entry["name"], entry["source"]
    path = root / source / "cdx-manifest.json"
    if not path.is_file():
        problems.append(f"{name}: no cdx-manifest.json at {source}")
        continue
    manifest = json.loads(path.read_text())
    listed = [manifest["plugin"], manifest.get("readme"), manifest.get("mcpServers")]
    for kind in ("skills", "agents", "apps", "previews", "automations"):
        listed += manifest.get(kind, [])
    missing = [rel for rel in listed if rel and not (path.parent / rel).is_file()]
    found = manifest["name"]
    if found != name or missing:
        problems.append(f"{path}: names {found}, missing {missing}")
    else:
        print(f"ok   manifest {source}")
for problem in problems:
    print(f"FAIL {problem}")
sys.exit(bool(problems))
PY

apt-get purge -y -qq $packages >/dev/null
if [ -e /usr/share/cdx-plugins ]; then
  fail "purge left files behind:"
  find /usr/share/cdx-plugins
fi
exit $failed
'

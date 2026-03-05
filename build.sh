#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

mapfile -t BUILD_SCRIPTS < <(
  find "$ROOT_DIR/plugins" -mindepth 3 -maxdepth 3 -type f -path '*/src/build.sh' | sort
)

if (( ${#BUILD_SCRIPTS[@]} == 0 )); then
  echo "ERROR: No plugin build scripts found under plugins/*/src/build.sh" >&2
  exit 1
fi

for script in "${BUILD_SCRIPTS[@]}"; do
  rel_script="${script#$ROOT_DIR/}"
  echo "==== Running $rel_script ===="
  "$script"
done

echo "All plugin builds completed successfully."

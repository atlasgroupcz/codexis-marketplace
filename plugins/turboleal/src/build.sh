#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

build_plugin_crate "$SCRIPT_DIR/turboleal-cli" "turboleal-cli"
install_workspace_binary turboleal-cgi "$PLUGIN_DIR/components/turboleal"

# turboleal.mjs is an ES module the chat imports from a `node -e` script, so
# unlike the extensionless CJS CLIs (plugins/sid) it is bundled as ESM.
BUN_IMAGE="oven/bun:1.3.9-slim@sha256:8ca06c7812d9050ccc4b80799685f395d6a0d051d3b7207dfd120e2b437b1ec9"
docker run --rm \
  --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -v "$SCRIPT_DIR/turboleal-js:/app" \
  -w /app \
  "$BUN_IMAGE" \
  sh -c "bun install --frozen-lockfile \
    && bun test \
    && bun build src/index.ts --target=node --format=esm --outfile=dist/turboleal.mjs"
install -d "$PLUGIN_DIR/lib"
install -m 0644 "$SCRIPT_DIR/turboleal-js/dist/turboleal.mjs" "$PLUGIN_DIR/lib/turboleal.mjs"
echo "   installed -> $PLUGIN_DIR/lib/turboleal.mjs"

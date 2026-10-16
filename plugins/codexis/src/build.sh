#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

build_plugin_crate "$SCRIPT_DIR/cdx-cli" "cdx-cli"
build_plugin_crate "$SCRIPT_DIR/cdxctl" "cdxctl"
build_plugin_crate "$SCRIPT_DIR/turboleal-cli" "turboleal-cli"
install_workspace_binary turboleal-cgi "$PLUGIN_DIR/components/turboleal"

# The alerts library for the chat `script` tool: one self-contained CJS bundle,
# tested in the same pinned image that builds it.
BUN_IMAGE="oven/bun:1.3.9-slim@sha256:8ca06c7812d9050ccc4b80799685f395d6a0d051d3b7207dfd120e2b437b1ec9"
docker run --rm \
  --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -v "$SCRIPT_DIR/turboleal-js:/app" \
  -w /app \
  "$BUN_IMAGE" \
  sh -c "bun install --frozen-lockfile \
    && bun test \
    && bun build src/index.ts --target=node --format=cjs --outfile=dist/turboleal.cjs"
install -d "$PLUGIN_DIR/lib/turboleal"
install -m 0644 "$SCRIPT_DIR/turboleal-js/dist/turboleal.cjs" "$PLUGIN_DIR/lib/turboleal/turboleal.cjs"
echo "   installed -> $PLUGIN_DIR/lib/turboleal/turboleal.cjs"

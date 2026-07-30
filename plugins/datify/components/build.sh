#!/usr/bin/env bash
# Builds the datify component frontend and syncs it into components/.
#
# The CGI binary, the icon and component.json are authored, not built - they live
# at the component root and the sync leaves them alone. See
# shared/scripts/component-build.sh.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/../../../shared/scripts/component-build.sh"

build_component datify

echo "Completed component build: $(basename "$PLUGIN_DIR")"

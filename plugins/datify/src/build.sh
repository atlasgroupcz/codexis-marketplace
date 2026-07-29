#!/usr/bin/env bash
# Installs the two datify binaries into their component locations:
#   datify-cli -> plugins/datify/bin/                  (CLI)
#   datify-cgi -> plugins/datify/components/datify/    (settings handler, run in
#                 place by the daemon)
#
# Both binaries are produced by the one-pass workspace build in the repo-root
# build.sh (build_rust_workspace); this script just installs them from the shared
# target/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # plugins/datify/src
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"                    # plugins/datify
source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

# build_plugin_crate ensures the datify crate is built (the standalone
# fallback, when run without the one-pass workspace build) and installs the CLI;
# datify-cgi comes from that same crate build, so it just installs.
build_plugin_crate "$SCRIPT_DIR/datify" "datify-cli"
install_workspace_binary datify-cgi "$PLUGIN_DIR/components/datify"

echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

#!/usr/bin/env bash
# Installs the two katastr binaries into their component locations:
#   katastr-cli -> plugins/katastr/bin/                 (system-wide CLI)
#   katastr-cgi -> plugins/katastr/components/katastr/  (dashboard handler, run in
#                  place by the daemon)
#
# Both binaries are produced by the one-pass workspace build in the repo-root
# build.sh (build_rust_workspace); this script just installs them from the shared
# target/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # plugins/katastr/src
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"                    # plugins/katastr
source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

# build_plugin_crate ensures the katastr crate is built (the standalone
# fallback, when run without the one-pass workspace build) and installs the CLI;
# katastr-cgi comes from that same crate build, so it just installs.
build_plugin_crate "$SCRIPT_DIR/katastr" "katastr-cli"
install_workspace_binary katastr-cgi "$PLUGIN_DIR/components/katastr"

echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

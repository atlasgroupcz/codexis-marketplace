#!/usr/bin/env bash
# Installs the two sbirka-listin binaries into their component locations:
#   sbirka-listin-cli -> plugins/sbirka-listin/bin/                       (CLI)
#   sbirka-listin-cgi -> plugins/sbirka-listin/components/sbirka-listin/  (dashboard
#                        handler, run in place by the daemon)
#
# Both binaries are produced by the one-pass workspace build in the repo-root
# build.sh (build_rust_workspace); this script just installs them from the shared
# target/.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # plugins/sbirka-listin/src
PLUGIN_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"                    # plugins/sbirka-listin
source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

# build_plugin_crate ensures the sbirka-listin-cli crate is built (the standalone
# fallback, when run without the one-pass workspace build) and installs the CLI;
# sbirka-listin-cgi comes from that same crate build, so it just installs.
build_plugin_crate "$SCRIPT_DIR/sbirka-listin-cli" "sbirka-listin-cli"
install_workspace_binary sbirka-listin-cgi "$PLUGIN_DIR/components/sbirka-listin"

echo "Completed plugin build: $(basename "$PLUGIN_DIR")"

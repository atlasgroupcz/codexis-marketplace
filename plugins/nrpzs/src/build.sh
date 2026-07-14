#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../../shared/scripts/plugin-build.sh"

build_plugin_crate "$SCRIPT_DIR/nrpzs-cli" "nrpzs-cli"

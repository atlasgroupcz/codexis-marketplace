#!/usr/bin/env bash
# Instaluje sid-cli do user-local bin dir.
# Guest VM má ${HOME}/.local/bin na PATH (interaktivně i pro cdx-remote-exec),
# takže není potřeba sudo ani zápis mimo domovský adresář.
set -euo pipefail

PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TARGET="${TARGET_BIN_DIR:-${HOME}/.local/bin}"

install -d "${TARGET}"
install -m 0755 "${PLUGIN_DIR}/bin/sid-cli" "${TARGET}/sid-cli"
echo "sid-cli nainstalován do ${TARGET}"

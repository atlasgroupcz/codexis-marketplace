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

# Výchozí nastavení notifikačních kanálů. Konvence sdílená s pluginy
# znamky/katastr/insolvence: shipped defaults jdou do
# ~/.local/share/<app>/settings/, uživatelský override žije jinde
# (~/.cdx/apps/sid/settings.json) a tenhle krok ho nikdy nepřepíše.
SETTINGS_TARGET="${TARGET_SHARE_DIR:-${HOME}/.local/share}/sid/settings"
if [ -f "${PLUGIN_DIR}/settings/sid/defaults.json" ]; then
  install -d "${SETTINGS_TARGET}"
  install -m 0644 "${PLUGIN_DIR}/settings/sid/defaults.json" "${SETTINGS_TARGET}/defaults.json"
  echo "výchozí nastavení nainstalováno do ${SETTINGS_TARGET}"
fi

#!/usr/bin/env bash
# Odstraní sid-cli z user-local bin dir. Konfigurace uživatele
# (~/.cdx/apps/sid/.env s API klíčem) se ZÁMĚRNĚ nemaže — přeinstalace
# pluginu by jinak vyžadovala znovu vydat klíč v SID.
set -euo pipefail

TARGET="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
rm -f "${TARGET}/sid-cli"
echo "sid-cli odstraněn z ${TARGET}"

# Shipped defaults patří k instalaci, takže odcházejí s ní. Uživatelské
# soubory v ~/.cdx/apps/sid/ (API klíč, settings.json) se ZÁMĚRNĚ nemažou —
# přeinstalace by jinak nutila znovu vydat klíč v SID.
rm -rf "${TARGET_SHARE_DIR:-${HOME}/.local/share}/sid/settings"

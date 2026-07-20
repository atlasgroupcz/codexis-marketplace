#!/usr/bin/env bash
# Odstraní sid-cli z user-local bin dir. Konfigurace uživatele
# (~/.cdx/apps/sid/.env s API klíčem) se ZÁMĚRNĚ nemaže — přeinstalace
# pluginu by jinak vyžadovala znovu vydat klíč v SID.
set -euo pipefail

TARGET="${TARGET_BIN_DIR:-${HOME}/.local/bin}"
rm -f "${TARGET}/sid-cli"
echo "sid-cli odstraněn z ${TARGET}"

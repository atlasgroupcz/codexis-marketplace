#!/usr/bin/env bash
# Dockerized, reproducible build (repo policy — README "Per-Plugin src/build.sh Contract").
#
# Na rozdíl od Rust pluginů nevzniká nativní binárka: sid-cli je TypeScript
# zbundlovaný do JEDNOHO self-contained .mjs se shebangem `#!/usr/bin/env node`.
# Guest VM má nodejs z aptu (viz cdx-daemon rootfs), takže za runtime nejsou
# potřeba žádné node_modules ani cross-compilace pro architektury.
#
# Testy běží uvnitř TÉHOŽ image jako bundle — co se testuje, to se ships.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMG="oven/bun@sha256:7156fcc0cee0194d390bfaf7f0eeda9a5e383e70cc90f31aad3a2440a033d7dc"

# --user: bez něj kontejner zapisuje dist/ jako root a hostitelský uživatel
# pak nemůže artefakty ani smazat. HOME=/tmp, protože přimapovaný uživatel
# nemá v image domovský adresář (bun tam chce cache).
docker run --rm \
  --user "$(id -u):$(id -g)" \
  -e HOME=/tmp \
  -v "${SCRIPT_DIR}/sid-cli:/app" \
  -w /app \
  "${IMG}" \
  sh -c "bun install --frozen-lockfile \
    && bun test \
    && bun build src/main.ts --target=node --format=cjs --outdir=dist --entry-naming=sid-cli.cjs \
    && { [ -f src/cgi.ts ] && bun build src/cgi.ts --target=node --format=cjs --outdir=dist --entry-naming=sid-cgi.cjs || true; }"

# Bundle → spustitelný artefakt.
#
# Formát je CJS záměrně: artefakty se instalují BEZ přípony (`sid-cli`), a
# takový soubor Node načítá jako CommonJS — ESM bundle by spadl na
# "Cannot use import statement outside a module". Shebang si nese sám bundle
# ze zdrojáku, takže ho tu nepřidáváme (jinak vznikne dvojitý a soubor je
# nevalidní JS).
emit() {
  local src="${SCRIPT_DIR}/sid-cli/dist/$1" dst="${SCRIPT_DIR}/../$2"
  [ -f "${src}" ] || { echo "chybí bundle: ${src}" >&2; exit 1; }
  head -1 "${src}" | grep -q '^#!' || { echo "bundle ${src} nemá shebang" >&2; exit 1; }
  mkdir -p "$(dirname "${dst}")"
  cp "${src}" "${dst}"
  chmod 0755 "${dst}"
  echo "OK: ${dst}"
}

emit sid-cli.cjs bin/sid-cli
# Komponenta (CGI backend) je volitelná — build ji přeskočí, dokud
# src/cgi.ts neexistuje.
[ -f "${SCRIPT_DIR}/sid-cli/dist/sid-cgi.cjs" ] && emit sid-cgi.cjs components/sid/sid-cgi

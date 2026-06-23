# Registr léčiv (SÚKL DLP) — návrh pluginu

- **Jira:** LM-117 „plugin - Registr léčiv"
- **Datum:** 2026-06-23
- **Branch:** `lm-117-registr-leciv`
- **Status:** návrh ke schválení

## 1. Účel

Plugin pro vyhledávání léčivých přípravků registrovaných v ČR z databáze SÚKL
(Databáze léčivých přípravků, DLP). Hlavní use case je v chatu: uživatel se
přirozeně zeptá („najdi lék Paralen", „složení léku s kódem 0182362", „jaká je
účinná látka v X"), model si zavolá CLI a odpoví.

Typické dotazy: identifikace léku, síla/forma/balení, účinné látky, ATC skupina,
způsob výdeje, dokumenty (SPC/PIL), ceny a úhrady.

## 2. Zdroj dat (ověřeno naživo)

Vše veřejné, **bez API klíče**.

**REST API** — `https://prehledy.sukl.cz/dlp/v1` (OpenAPI 3.0, spec na
`https://prehledy.sukl.cz/dlp.api.json`). Klíčové endpointy:

| Endpoint | Vrací |
|---|---|
| `GET /status` | status serveru |
| `GET /aktualni-davky` | aktuální dávky (typ, `datumVydani`, `platnostOd`, `verze`) |
| `GET /lecive-pripravky?typSeznamu=dlpo` | seznam **jen kódů** (~69 000 stringů) |
| `GET /lecive-pripravky/{kodSukl}` | detail léku (název, síla, forma, balení, ATC, účinné látky, výdej, DDD…) |
| `GET /slozeni/{kodSukl}` | složení (kódy látek + množství) |
| `GET /ciselnik-latky` | číselník účinných látek (kód → název) |
| `GET /ciselniky/{nazev}` | číselníky (ATC apod.) |
| `GET /dokumenty-metadata/{kodSukl}`, `GET /dokumenty/{kodSukl}/{typ}` | dokumenty (SPC/PIL) |
| `GET /cau-scau/{kodSukl}` | ceny a úhrady |

**Open-data dump** — `https://opendata.sukl.cz` (katalog DLP). Měsíční zip
(`.../soubory/SOD<YYYYMMDD>/DLP<YYYYMMDD>.zip`, ~9,4 MB → ~63 MB rozbaleno,
30 CSV). Klíčový soubor `dlp_lecivepripravky.csv` (~69 000 řádků) má
`KOD_SUKL;…;NAZEV;SILA;FORMA;BALENI;…;ATC_WHO;…`.

**Kadence (ověřeno z `/historicke-davky`):** přesně **1× měsíčně**, 26 dávek =
26 po sobě jdoucích měsíců, formát `YYYY.MM`, žádné půlměsíční. Dávka se vydává
~27. dne, je platná od 1. (`platnostOd`). Pole `verze` umožňuje opravná
re-vydání, ale v historii se reálně nedějí (drží 1).

## 3. Architektura

Vzor **ARES**, ne katastr:

- skill + jediná CLI binárka, která obaluje SÚKL API a lokální index
- **hlavní use case v chatu** — model volá CLI, odpovídá uživateli
- **žádná UI komponenta**, **žádné sledování/storage v čase** (na rozdíl od katastru — není co hlídat)
- **žádný uživatelský API klíč** (na rozdíl od katastru, který chce ČÚZK klíč)

### Klíčový problém: hledání podle názvu

SÚKL API neumí hledat podle názvu — má jen `kód → detail`, seznam vrací jen
kódy. Mapování `název → kód` API neposkytuje. Řešení: **lokální index** z
open-data dumpu. CLI si stáhne dump, z `dlp_lecivepripravky.csv` postaví lokální
index (kód ↔ název ↔ pár polí na rozlišení), a v něm hledá podle názvu offline.
Detail/složení/dokumenty/ceny pak dotahuje živě z API podle kódu.

## 4. Tři vrstvy obnovy dat

Žádná překrývající se logika, všechny tři volají stejné `refresh`:

| Vrstva | Kdy | Úkol |
|---|---|---|
| **postInstall hook** | při instalaci pluginu | stáhne dump a postaví index hned → chat nikdy nečeká na cold-start |
| **CLI lazy `if (!index_valid)`** | při hledání | pojistka, kdyby postInstall selhal nebo cache zmizela |
| **deklarovaná automatizace `0 3 1 * *`** | měsíčně (1. dne) | udržuje čerstvost |

Žádný bundling indexu do pluginu (plugin se nebude vydávat měsíčně, data by
zestárla). Žádný cron častěji než měsíčně (data jsou měsíční, častější běh by
jen redundantně stahoval tu samou dávku).

## 5. Příkazy CLI

Binárka `leciva-cli`. Výstup vždy JSON na stdout, chyby
`ERROR: …` na stderr + nenulový exit (konvence jako ares/katastr).

```
leciva-cli search "<text>"        # lokální index → [{kodSukl, nazev, sila, doplnek}]
leciva-cli detail <kodSukl>       # živý detail z API
leciva-cli slozeni <kodSukl>      # složení + názvy účinných látek (přes číselník)
leciva-cli latka "<text>"         # hledání podle účinné látky
leciva-cli dokumenty <kodSukl>    # SPC/PIL odkazy (živé API)
leciva-cli ceny <kodSukl>         # ceny / úhrady (živé API)
leciva-cli refresh                # stáhnout dump + přestavět index (idempotentní)
leciva-cli index status           # stáří/verze indexu z manifestu
leciva-cli api get <PATH>         # únikový poklop na libovolný DLP endpoint
```

Tok v chatu: „složení Paralenu" → `search paralen` → vezme kód → `slozeni <kod>`.

## 6. Lokální úložiště + atomický refresh

Data v **`~/.cdx/apps/leciva/`** — konvence repa pro data pluginů (sledované
dokumenty: `~/.cdx/apps/sledovane-dokumenty/`, stejně katastr/znamky/insolvence/
judikatura). **Ne** `~/.local/share/` (to je jen na nainstalovaný kód knihovny) a
**ne** do složky pluginu (tu marketplace installer při updatu přepíše → data by
se ztratila). `~/.cdx/apps/` přežije update pluginu a čtou z něj i UI/daemon.
Home brát z `user_home()` helperu (host-injected `CODEXIS_PUBLIC` home), ne
natvrdo `~`.

```
~/.cdx/apps/leciva/
  index.<ext>              # postavený vyhledávací index (kód ↔ název ↔ pole)
  manifest.json            # { "batch": "2026.06", "platnostOd": "...", "downloadedAt": "...", "rowCount": 69147 }
  last_check               # timestamp poslední freshness kontroly (TTL)
```

**Refresh (atomicky, nikdy nerozbije běžící index):**

1. `GET /aktualni-davky` → zjisti aktuální `platnostOd`/`verze`. Pokud se proti
   `manifest.json` nezměnilo, skonči (nic nestahuj).
2. Najdi a stáhni aktuální dump (resolve URL — viz edge-case #6) do temp.
3. Rozbal jen potřebné CSV, **překóduj cp1250 → UTF-8**, postav index do temp.
4. **Zvaliduj** (počet řádků > rozumný práh, parsuje se).
5. **Atomicky přehoď** index na místo a **až nakonec** zapiš `manifest.json`.
6. Smaž starý/temp.

Když cokoli ve 2–5 selže, manifest pořád ukazuje na starou platnou verzi →
servíruje se stará, nic se nerozbije. „Verze se updatuje až po úspěšné výměně."

**Platnost indexu** (`index_valid`) = manifest existuje **a** odkazovaný index
soubor existuje a projde základní validací. `exists()` nesmí znamenat jen
„soubor tam je".

## 7. Edge-case (požadavky na implementaci)

3. **Selhání stažení** (cold start i refresh) → jasný `ERROR:` (čeština pro uživatele), žádný pád ani prázdný výstup. Index zůstává neexistující → příští volání to zkusí znovu.
4. **Půlka/poškozené stažení** → temp + atomický rename + validace (viz §6). Nikdy poloviční index neoznačit za platný.
5. **Selhání měsíčního refresh** → nepřemazat starý index; servíruj stará data (stale ≤ měsíc), spraví se příští refresh.
6. **Resolve aktuální URL dumpu** → URL je datovaná (`SOD<YYYYMMDD>`). Odvodit z katalogu opendat nebo z `datumVydani` v `/aktualni-davky`; když se nedá najít, jasný error. Riziko: SÚKL změní strukturu katalogu.
7. **Změna schématu/kódování CSV** → parsovat podle **názvu sloupce** (CSV má hlavičku), ne podle pozice; ošetřit cp1250.
8. **Souběh volání** (paralelní tool cally v jednom tahu, víc chatů) → atomický temp+rename to dělá neškodným (poslední vyhraje, stejná data). Volitelně lock.

## 8. Struktura pluginu

Mirror ares:

```
plugins/leciva/
  .claude-plugin/plugin.json     # manifest + i18n + postInstall/onUpdate/postUninstall
  automations/
    refresh-leciva.md            # deklarovaná COMMAND automatizace, cron 0 3 1 * *, command: leciva-cli refresh
  bin/
    leciva-cli                   # build artefakt (per-platform, jako ares)
  hooks/
    install-binaries.sh          # instalace binárky + (na konci) leciva-cli refresh
    uninstall-binaries.sh        # smaže binárku I cache (rm -rf ~/.cdx/apps/leciva)
  skills/
    leciva/
      SKILL.md                   # trigger text + i18n + návod, jak volat CLI
      icon.svg
  src/                           # zdroj CLI (Rust)
  acceptance/
    e2e/                         # acceptance test (vzor ares)
  icon.svg
```

Plugin `leciva`, skill `leciva`, binárka `leciva-cli` (vzor `znamky`/`insolvence`
— místní název, ne anglický překlad). Plus zápis do
`.claude-plugin/marketplace.json` (uuid, name, source, category `legal`, i18n) a
SKILL frontmatter s `i18n` (cs/en/sk) dle konvence repa.

Deklarovaná automatizace `automations/refresh-leciva.md` (formát ověřen na `codexis-sync.md`):

```yaml
---
uuid: <nove-uuid>
type: COMMAND
title: Aktualizovat databázi léčiv
description: Stáhne aktuální databázi léčivých přípravků ze SÚKL.
command: leciva-cli refresh
cron: 0 3 1 * *
enabled: true
notify-on-success: false
notify-on-failure: false
i18n: { cs: {...}, en: {...}, sk: {...} }
---
```

Daemon ji při instalaci pluginu sám objeví a založí (`PluginAutomationDiscovery`).
Pozn.: deklarovaná automatizace se po založení **nespustí hned** — první běh je
až na nejbližší cron (proto je tu postInstall na okamžitou populaci).

## 9. Technologie

Rust (jako `ares-cli`) — binárka bez runtime závislostí. Knihovny: `reqwest`
(HTTP), `zip` (dump), `csv` (parsing), `encoding_rs` (cp1250 → UTF-8), `serde`/
`serde_json` (výstup + manifest). Per-platform build jako ares.

## 10. Mimo rozsah (YAGNI)

- žádná UI komponenta (vše v chatu)
- žádné sledování/notifikace v čase (to je katastr/monitoring vzor, ne tohle)
- žádný uživatelský API klíč
- žádný našeptávač názvu jako samostatné UI (našeptává se v chatu přes `search`)
- žádný cron častěji než měsíčně
- žádné publikování zdrojů do chatu přes daemon secret (jako kataster-sk/orsr) — případně až v2; v1 nevolá daemon, nepotřebuje `~/.cdx/env/secret`

## 11. Lifecycle (instalace / odinstalace)

- **postInstall:** nainstaluje binárku + spustí `leciva-cli refresh` (první populace cache).
- **onUpdate:** přeinstaluje binárku (data zůstávají v `~/.cdx/apps/leciva/`, přežijí update).
- **postUninstall:** smaže binárku **i** cache (`rm -rf ~/.cdx/apps/leciva`). Cache je přestavitelná, není to user state → uklízíme.
- **Automatizace:** deklarovaná (`automations/refresh-leciva.md`, Model 2) → daemon ji založí při instalaci a **sám odhlásí při odinstalaci** (reconciler ji přestane objevovat). Žádný ruční úklid.

## 12. Otevřené body

- ~~Jméno binárky / pluginu / skillu~~ — **rozhodnuto:** plugin `leciva`, skill `leciva`, binárka `leciva-cli`.
- **Formát lokálního indexu:** zatím „postavený index z CSV" — konkrétní formát (slimmed CSV vs serializovaná struktura) doladit v implementaci; 69k řádků = lineární sken v paměti stačí, žádné SQLite.
- **Resolve URL dumpu (#6):** ověřit nejstabilnější způsob (katalog vs odvození z `datumVydani`).
- **Rozsah v1 příkazů:** jistě `search`/`detail`/`slozeni`/`refresh`; `dokumenty`/`ceny`/`latka` možná až v2.

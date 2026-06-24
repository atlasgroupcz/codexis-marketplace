---
uuid: 4d8b61c5-641d-4201-85fe-259ca742eaeb
name: kataster-sk
icon: icon.svg
description: Slovenský kataster nehnuteľností (ÚGKK SR). Use for Slovak parcel lookups — parcel number, cadastral unit (katastrálne územie), area (výmera), land use, register C/E parcels, and links to title deeds (list vlastníctva). Triggers on "kataster", "kataster nehnuteľností", "parcela", "parcelné číslo", "list vlastníctva", "LV", "katastrálne územie", "výmera pozemku", "pozemok na Slovensku", "ZBGIS", "ESKN", "kataster portál". Slovak (SK) jurisdiction only — for the Czech cadastre use the katastr skill.
version: 1.3.0
jurisdictions: [SK]
i18n:
  cs:
    displayName: "Kataster nehnuteľností SR"
    summary: "Vyhledávání parcel ve slovenském katastru — parcelní čísla, výměry, druh pozemku a odkazy na list vlastníctva."
  en:
    displayName: "Slovak Land Cadastre"
    summary: "Look up parcels in the Slovak cadastre — parcel numbers, areas, land use, and links to title deeds."
  sk:
    displayName: "Kataster nehnuteľností SR"
    summary: "Vyhľadávanie parciel v slovenskom katastri — parcelné čísla, výmery, druh pozemku a odkazy na list vlastníctva."
---

# Slovak Land Cadastre (kataster nehnuteľností SR)

A single tool — **`kataster-sk-cli`** — wraps the two public endpoints of
ÚGKK SR (ZBGIS MAPKA search + ESKN ArcGIS REST) with the browser-like headers
the cadastre WAF requires.

**IMPORTANT:** The only tool in this skill is `kataster-sk-cli`. Do NOT call
`curl` or any other tool against `skgeodesy.sk` directly — raw requests are
rejected by the WAF. Assume `kataster-sk-cli` is installed and available in `PATH`.

**IMPORTANT:** If `kataster-sk-cli` outputs an `ERROR:` line, stop and report it
to the user. The cadastre services are occasionally unstable (recovery from the
January 2025 cyberattack) — do not retry blindly.

## Output Format

`kataster-sk-cli` always prints **JSON to stdout** (verbatim API response). Parse
with `jq`. On failure: `ERROR: …` on stderr, exit code 2.

## What this skill can and cannot do

| Data | Available |
|---|---|
| Parcel number, areas, land use, register C/E attributes | ✅ `kataster-sk-cli` |
| Cadastral unit lookup (name → code) | ✅ `kataster-sk-cli` |
| Owner names, LV (list vlastníctva) contents | ❌ no anonymous API — provide deep links instead |
| LV usable for legal acts (sealed PDF) | ❌ eID only via slovensko.sk — provide link |

Owner data requires a verified login on the ESKN portal (mandatory by law from
1 July 2026; an informative LV excerpt costs 6 EUR). Never attempt to scrape or
bypass the recaptcha-protected owner endpoints — always hand the user a deep link.

## Workflow

### 1. Resolve the cadastral unit (katastrálne územie) code

```bash
kataster-sk-cli ku "Ruzinov" | jq '.items[].data | {category, id, text}'
```

Entries with `category` = `katastrálne územie` carry `id` (the KU code,
e.g. `805556`) and `text` (e.g. `Ružinov (805556)`). Diacritics are optional.

### 2. Find the parcel by number within the cadastral unit

```bash
kataster-sk-cli parcela 805556 "1234/9" | jq '.items[].data | {category, id, text, route}'
```

`category` is `parcela-c` (register C) or `parcela-e` (register E), `id` is the
object id for step 3, `route` is the MAPKA deep-link path suffix.

### 3. Fetch parcel attributes

```bash
kataster-sk-cli detail 1070374091
kataster-sk-cli detail 123456 --register e
kataster-sk-cli detail 1070374091 --geometry   # adds the WGS84 polygon
```

Key attributes: `PARCEL_NUMBER`, `DESCRIPTIVE_AREA_OF_PARCEL` (official area in m²),
`GEODETIC_AREA_OF_PARCEL`, `NATURE_OF_LAND_USE_ID`, `OWNERSHIP_TYPE_ID`,
`FOLIO_ID` (internal title-deed identifier; `null` when the parcel has no LV record
in that register view). Field details: `references/arcgis-api.md`.

**Druh pozemku:** `detail` output already carries the translated land-use fields
`DRUH_POZEMKU` (official label, e.g. `Zastavaná plocha a nádvorie`) and
`DRUH_POZEMKU_KOD` (official kód per vyhláška č. 461/2009 Z. z.) — the CLI derives
them from `NATURE_OF_LAND_USE_ID`. Use them directly.

**Untranslatable code-only fields:** `PLOT_UTILISATION_ID` (spôsob využívania),
`OWNERSHIP_TYPE_ID` and the remaining `*_ID` fields are internal identifiers with
no public code list. Do not search for a mapping, do not guess labels, and do not
spawn agents to find one — state the numeric identifier and point the user to the
parcel's MAPKA page (step 4), which renders the human-readable labels.

### 4. Give the user deep links for owners and the LV

- Parcel in the MAPKA map: `https://zbgis.skgeodesy.sk/mapka/sk/kataster/detail/{route}`
  (`route` from step 2, e.g. `kataster/parcela-c/805556/1234_9` — yes, `kataster`
  appears twice in the final URL; that is the app's own detail-route format).
  `kataster-sk-cli parcela` attaches these MAPKA pages to the chat "Sources"
  panel automatically; `ku` and `detail` attach nothing — the citation for a
  parcel comes from the `parcela` step. Still cite the link in prose.
- ESKN portal (owner/LV lookup after login): `https://kataster.skgeodesy.sk/eskn-portal/`
- CICA web search: `https://cica.vugk.sk/`
- LV usable for legal acts (eID required): `https://www.slovensko.sk/sk/detail-sluzby?externalCode=ks_336485`

## Output rules

- Match the user's conversation language (Slovak data stays in Slovak where natural).
- User-facing links are only the MAPKA/ESKN/CICA/slovensko.sk pages above — never
  present raw API URLs.
- State the official area (`DESCRIPTIVE_AREA_OF_PARCEL`) when reporting výmera;
  mention the geodetic area only when it differs.
- On `ERROR:` output, say plainly that the cadastre service refused or is down
  and fall back to the MAPKA deep link.

## Reference Files

- **`references/zbgis-search.md`** — search categories and response shape
- **`references/arcgis-api.md`** — parcel layers, fields, query limits

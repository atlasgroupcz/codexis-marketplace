---
uuid: 4d8b61c5-641d-4201-85fe-259ca742eaeb
name: kataster-sk
description: Slovenský kataster nehnuteľností (ÚGKK SR). Use for Slovak parcel lookups — parcel number, cadastral unit (katastrálne územie), area (výmera), land use, register C/E parcels, and links to title deeds (list vlastníctva). Triggers on "kataster", "kataster nehnuteľností", "parcela", "parcelné číslo", "list vlastníctva", "LV", "katastrálne územie", "výmera pozemku", "pozemok na Slovensku", "ZBGIS", "ESKN", "kataster portál". Slovak (SK) jurisdiction only — for the Czech cadastre use the katastr skill.
version: 1.0.0
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

Anonymous, read-only access to Slovak cadastre parcel data via two public endpoints
of ÚGKK SR (Úrad geodézie, kartografie a katastra SR):

1. **ZBGIS MAPKA search API** — geocode cadastral units and find parcels by number.
2. **ESKN ArcGIS REST** — full parcel attributes (area, land use, ownership type).

Use `curl` directly. **Always send a browser-like `User-Agent` header** — requests
without one are rejected by the WAF:

```bash
UA="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"
```

## What this skill can and cannot do

| Data | Available |
|---|---|
| Parcel number, areas, land use, register C/E attributes | ✅ anonymously via API |
| Cadastral unit lookup (name → code) | ✅ anonymously via API |
| Owner names, LV (list vlastníctva) contents | ❌ no anonymous API — provide deep links instead |
| LV usable for legal acts (sealed PDF) | ❌ eID only via slovensko.sk — provide link |

Owner data requires a verified login on the ESKN portal (mandatory by law from
1 July 2026; an informative LV excerpt costs 6 EUR). Never attempt to scrape or
bypass the recaptcha-protected owner endpoints — always hand the user a deep link.

## Workflow

### 1. Resolve the cadastral unit (katastrálne územie) code

```bash
curl -s -H "User-Agent: $UA" \
  "https://zbgis.skgeodesy.sk/mapka/api/search/kataster?q=Ruzinov"
```

Returns `items[]`; entries with `category` = `katastrálne územie` carry
`data.id` (the KU code, e.g. `805556`), `data.municipalityCode`, and `data.text`
(e.g. `Ružinov (805556)`). Diacritics in `q` are optional.

### 2. Find the parcel by number within the cadastral unit

```bash
curl -s -H "User-Agent: $UA" \
  "https://zbgis.skgeodesy.sk/mapka/api/search/kataster/805556/mu/805556?q=1234"
```

Returns matching parcels: `data.category` is `parcela-c` (register C) or
`parcela-e` (register E), `data.id` is the parcel object id for step 3,
`data.text` is the full parcel number (e.g. `1234/9`), `data.description`
confirms the KU.

### 3. Fetch parcel attributes from ArcGIS REST

Register C parcels use `parcels_c_view`, register E parcels `parcels_e_view`:

```bash
curl -s -H "User-Agent: $UA" \
  "https://kataster.skgeodesy.sk/eskn/rest/services/VRM/parcels_c_view/MapServer/0/query?objectIds=1070374091&outFields=*&returnGeometry=false&f=json"
```

Key attributes: `PARCEL_NUMBER`, `DESCRIPTIVE_AREA_OF_PARCEL` (official area in m²),
`GEODETIC_AREA_OF_PARCEL`, `NATURE_OF_LAND_USE_ID`, `OWNERSHIP_TYPE_ID`,
`FOLIO_ID` (internal title-deed identifier; `null` when the parcel has no LV record
in that register view). Field details and code lists: `references/arcgis-api.md`.

**Only query by `objectIds` (from step 2) or by geometry.** Generic `where=`
queries are blocked by the WAF and return an HTML error page.

### 4. Give the user deep links for owners and the LV

- Parcel in the MAPKA map (replace `/` in the parcel number with `_`):
  `https://zbgis.skgeodesy.sk/mapka/sk/kataster/parcela-c/805556/1234_9`
- ESKN portal (owner/LV lookup after login): `https://kataster.skgeodesy.sk/eskn-portal/`
- CICA web search: `https://cica.vugk.sk/`
- LV usable for legal acts (eID required): `https://www.slovensko.sk/sk/detail-sluzby?externalCode=ks_336485`

## Output rules

- Match the user's conversation language (Slovak data stays in Slovak where natural).
- Never present raw API URLs (`/mapka/api/...`, `/eskn/rest/...`) as clickable links —
  user-facing links are only the MAPKA/ESKN/CICA/slovensko.sk pages above.
- State the official area (`DESCRIPTIVE_AREA_OF_PARCEL`) when reporting výmera;
  mention the geodetic area only when it differs.
- If an endpoint returns an HTML error page or empty `items`, say so plainly and
  fall back to the MAPKA deep link — the cadastre services are occasionally
  unstable (recovery from the January 2025 cyberattack).

## Reference Files

- **`references/zbgis-search.md`** — search API endpoints, categories, response shape
- **`references/arcgis-api.md`** — ArcGIS layers, fields, query limits

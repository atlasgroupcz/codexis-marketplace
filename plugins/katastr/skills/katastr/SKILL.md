---
name: katastr
description: Access to CUZK REST API KN (api-kn.cuzk.gov.cz)
---

# CUZK REST API KN quick docs (pragmatic)

## Config

- Base URL: `https://api-kn.cuzk.gov.cz`
- Auth header: `ApiKey: i2sPUMBuyNlssfXudxY1NE1pdRWzXo`
- Typical header: `Accept: application/json`
- API is GET-only.

## Response envelope

Most endpoints return:

- `data`: payload (list or object)
- `aktualnostDatK`: data freshness timestamp
- `provedenoVolani`: counter of calls
- `zpravy`: optional messages

## Sanity checks (key + service)

```bash
kn_get "/api/v1/AplikacniSluzby/Health"
kn_get "/api/v1/AplikacniSluzby/StavUctu"
kn_get "/api/v1/AplikacniSluzby/AktualnostDat"
kn_get "/api/v1/AplikacniSluzby/ProvozniInformace"
```

## Known enums (from official Swagger)

- `TypParcely`: `PKN` (parcel in KN), `PZE` (simplified evidence)
- `DruhCislovaniParcely`: `1` (stavebni parcela), `2` (pozemkova parcela)
- `TypStavby` for searching: `1` (cislo popisne), `2` (cislo evidencni)
- `TypRizeni`: `V`, `Z`, `PGP`, `PD`, `ZPV`

Official Swagger UI is served from `https://api-kn.cuzk.gov.cz/swagger/`.

## Core attorney workflows (use cases)

Notes:

- This API does not provide personal data (owners etc.). If you need owners / full LV extracts, that is typically DP/WSDP.
- The API is useful for identification, linking (address <-> building <-> parcels <-> LV number), basic attributes, and basic signals (plomby/rizeni lists if present).

### 1) Identify parcel by (k.u. code + parcel number)

Inputs:

- `KodKatastralnihoUzemi` (e.g. 638790)
- `KmenoveCisloParcely` (e.g. 545)
- `DruhCislovaniParcely` (1/2). If unsure, try both.
- `TypParcely` usually `PKN`

Search:

```bash
kn_get "/api/v1/Parcely/Vyhledani?KodKatastralnihoUzemi=638790&TypParcely=PKN&DruhCislovaniParcely=2&KmenoveCisloParcely=545"
```

Take `data[0].id`, then:

```bash
kn_get "/api/v1/Parcely/<ID>"
```

What to read from parcel detail:

- `vymera`, `druhPozemku`, `zpusobVyuziti`
- `lv.cislo` (+ `lv.katastralniUzemi`)
- `stavba.id` (if a numbered building is linked)
- `definicniBod` (S-JTSK) for spatial queries
- `rizeniPlomby` (if not empty, there is pending activity)

### 2) Neighbors + "around" (context for due diligence)

Neighbors (fast, topological):

```bash
kn_get "/api/v1/Parcely/SousedniParcely/<PARCEL_ID>"
```

Spatial queries by polygon (EPSG:5514 or EPSG:5513, meters):

- Query parameter: `SeznamSouradnic`
- Value: JSON array of points with `x,y`
- Polygon should be closed (last point equals first)

```bash
kn_get "/api/v1/Parcely/Polygon" --get --data-urlencode 'SeznamSouradnic=[{"x":-494110.17,"y":-1116432.13},{"x":-494060.17,"y":-1116432.13},{"x":-494060.17,"y":-1116382.13},{"x":-494110.17,"y":-1116382.13},{"x":-494110.17,"y":-1116432.13}]'
kn_get "/api/v1/Stavby/Polygon"  --get --data-urlencode 'SeznamSouradnic=[{"x":-494110.17,"y":-1116432.13},{"x":-494060.17,"y":-1116432.13},{"x":-494060.17,"y":-1116382.13},{"x":-494110.17,"y":-1116382.13},{"x":-494110.17,"y":-1116432.13}]'
```

Practical trick:

- Read `definicniBod` from `Parcely/{id}` or `Stavby/{id}`
- Build a square polygon around it (e.g. +/- 25 m, 50 m, 100 m) to get "okoli"

### 3) Identify building by postal address (RUIAN address point code -> KN building)

REST API KN does not accept free-text address directly. Use RUIAN address point code:

1. Get `kod` from VDP RUIAN autocomplete endpoint
2. Call KN endpoint `/api/v1/Stavby/AdresniMisto/{kod}`

RUIAN fulltext (important headers; otherwise you may get HTML error page):

```bash
curl -fsS \
  -H "X-Requested-With: XMLHttpRequest" \
  -H "Accept: application/json" \
  --get "https://vdp.cuzk.gov.cz/vdp/ruian/adresnimista/fulltext" \
  --data-urlencode "adresa=Mala Strana 66, Hladke Zivotice"
```

Take `polozky[0].kod` (RUIAN code), then:

```bash
kn_get "/api/v1/Stavby/AdresniMisto/<RUIAN_KOD>"
```

Expected: `data` is an object (not a list) containing:

- `typStavby`, `cislaDomovni`, `castObce`, `obec`
- `zpusobVyuziti` (e.g. "rodinny dum")
- `parcely[]` (with their internal IDs and parcel numbers)
- `lv.cislo`
- `definicniBod`
- `jednotky` (may be empty)

To fetch full building detail:

```bash
kn_get "/api/v1/Stavby/<STAVBA_ID>"
```

### 4) Units (apartments / non-residential units)

If you know unit number + building identity:

```bash
kn_get "/api/v1/Jednotky/Vyhledani?KodCastiObce=<KOD_CASTI_OBCE>&TypStavby=1&CisloDomovni=<CP>&CisloJednotky=<CISLO_JEDNOTKY>"
kn_get "/api/v1/Jednotky/<JEDNOTKA_ID>"
```

Where to get `KodCastiObce` and `CisloDomovni`:

- from `Stavby/AdresniMisto/{kod}` or `Stavby/{id}`
- or from territorial code lists (see below)

### 5) Proceedings / plomby (basic signal, not a full legal extract)

Look for `rizeniPlomby` on parcel/building/unit. If you have a proceeding ID:

```bash
kn_get "/api/v1/Rizeni/<RIZENI_ID>"
```

If you have official proceeding identifiers:

```bash
kn_get "/api/v1/Rizeni/Vyhledani?TypRizeni=V&Cislo=<CISLO>&Rok=<ROK>&KodPracoviste=<KOD>"
kn_get "/api/v1/Rizeni/PrijateDne?TypRizeni=V&KodPracoviste=<KOD>&DatumPrijeti=2026-02-13"
```

### 6) Code lists (decode codes for reports)

```bash
kn_get "/api/v1/CiselnikyUzemnichJednotek/Obce"
kn_get "/api/v1/CiselnikyUzemnichJednotek/KatastralniUzemi"
kn_get "/api/v1/CiselnikyUzemnichJednotek/CastiObci"
kn_get "/api/v1/CiselnikyISKN/DruhyPozemku"
kn_get "/api/v1/CiselnikyISKN/TypyStavby"
kn_get "/api/v1/CiselnikyISKN/TypyJednotky"
kn_get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiStavby"
kn_get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiParcely"
kn_get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiJednotky"
kn_get "/api/v1/CiselnikyISKN/ZpusobyOchrany"
kn_get "/api/v1/CiselnikyISKN/Pracoviste"
```

## What we verified against real data (Hladke Zivotice example)

- Parcel search works with: `KodKatastralnihoUzemi=638790`, `TypParcely=PKN`, `DruhCislovaniParcely=2`, `KmenoveCisloParcely=545`
- RUIAN address point for "Mala Strana 66, Hladke Zivotice" was resolved via VDP fulltext, then `Stavby/AdresniMisto/{kod}` returned building + linked parcel(s) + LV number.


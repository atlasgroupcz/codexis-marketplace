# ČÚZK REST API KN - raw lookup reference

Raw GET access via `katastr-cli api get "<PATH>"`. The command handles auth,
errors and JSON output. Base host is `https://api-kn.cuzk.gov.cz` - pass only the
path (e.g. `/api/v1/Parcely/12345`). The API is GET-only and returns JSON.

Official Swagger UI: `https://api-kn.cuzk.gov.cz/swagger/`

```bash
katastr-cli api get "<PATH>"
```

## Response envelope

Most endpoints return:
- `data`: payload (list or object)
- `aktualnostDatK`: data freshness timestamp
- `provedenoVolani`: counter of calls
- `zpravy`: optional messages

## Sanity checks

```bash
katastr-cli api get "/api/v1/AplikacniSluzby/Health"          # public, doesn't validate API key
katastr-cli api get "/api/v1/AplikacniSluzby/StavUctu"        # validates the key
katastr-cli api get "/api/v1/AplikacniSluzby/AktualnostDat"
katastr-cli api get "/api/v1/AplikacniSluzby/ProvozniInformace"
```

## Known enums

- `TypParcely`: `PKN` (parcel in KN), `PZE` (simplified evidence)
- `DruhCislovaniParcely`: `1` (stavebni parcela), `2` (pozemkova parcela)
- `TypStavby` for searching: `1` (cislo popisne), `2` (cislo evidencni)
- `TypRizeni`: `V`, `Z`, `PGP`, `PD`, `ZPV`

## Scope

This API does not provide personal data (owners etc.) directly. For owners / full
LV extracts, the official channel is DP/WSDP. The API is useful for identification,
linking (address ↔ building ↔ parcels ↔ LV number), basic attributes, and basic
signals (plomby/řízení lists if present).

## "Is it clean?" - quick check for a parcel

"Čisté" usually means one of two things, so clarify with the user:
- `no_plomby`: no pending proceedings (plomby/řízení) on parcel/building/unit in this API
- `no_rights_limits`: no liens/easements/rights restrictions (NOT available via this API; needs an official LV extract)

Minimal `no_plomby` workflow:
1. Resolve parcel internal ID via `Parcely/Vyhledani` (include `PoddeleniCislaParcely` for parcel numbers like `2642/9`).
2. Fetch `Parcely/{id}` and check:
   - `rizeniPlomby` (empty list = "no plomby" signal)
   - `zpusobyOchrany` (territorial/protection limit, not a plomba)
   - `lv` (cross-check LV number)
   - `stavba.id` if present → fetch `Stavby/{id}` and check `rizeniPlomby` there too
3. Optional: `LV/{lv.id}` for `rizeniPlomby` + counts of linked parcels/buildings/units.

## Identify a parcel by (k.u. code + parcel number)

Inputs:
- `KodKatastralnihoUzemi` (e.g. 638790)
- `KmenoveCisloParcely` (e.g. 545)
- `PoddeleniCislaParcely` (optional; for `2642/9` use `KmenoveCisloParcely=2642` + `PoddeleniCislaParcely=9`)
- `DruhCislovaniParcely` (1/2 - try both if unsure)
- `TypParcely` usually `PKN`

```bash
katastr-cli api get "/api/v1/Parcely/Vyhledani?KodKatastralnihoUzemi=638790&TypParcely=PKN&DruhCislovaniParcely=2&KmenoveCisloParcely=545"
katastr-cli api get "/api/v1/Parcely/<ID>"
```

What to read from parcel detail:
- `vymera`, `druhPozemku`, `zpusobVyuziti`
- `lv.cislo` (+ `lv.katastralniUzemi`)
- `stavba.id` (if a numbered building is linked)
- `definicniBod` (S-JTSK) for spatial queries
- `rizeniPlomby` (if not empty, there is pending activity)

Owner lookup: this API has no owners. Present LV number (`lv.cislo`) and point the
user to an official LV extract for ownership.

## LV detail (limited)

If you have `lv.id`:

```bash
katastr-cli api get "/api/v1/LV/<LV_ID>"
```

Available: `rizeniPlomby` at LV level, lists/counts of `parcely`, `stavby`, `jednotky`.
Not available: owners and full rights/restrictions sections.

## Neighbors & spatial queries

```bash
katastr-cli api get "/api/v1/Parcely/SousedniParcely/<PARCEL_ID>"
```

Polygon search (EPSG:5514 / EPSG:5513, meters):
```bash
katastr-cli api get "/api/v1/Parcely/Polygon?SeznamSouradnic=%5B%7B%22x%22%3A-494110.17%2C%22y%22%3A-1116432.13%7D%2C...%5D"
```

Trick: read `definicniBod` from `Parcely/{id}` or `Stavby/{id}` and build a square polygon around it (±25/50/100 m) to get "okolí".

## Identify building by postal address (RUIAN)

REST API KN does not accept free-text addresses. Use the RUIAN address point code:

1. Resolve address via VDP RUIAN fulltext (separate service, NOT via `katastr-cli` - use `curl` here as an exception):
   ```bash
   curl -fsS \
     -H "X-Requested-With: XMLHttpRequest" \
     -H "Accept: application/json" \
     --get "https://vdp.cuzk.gov.cz/vdp/ruian/adresnimista/fulltext" \
     --data-urlencode "adresa=Mala Strana 66, Hladke Zivotice"
   ```
2. Take `polozky[0].kod`, then:
   ```bash
   katastr-cli api get "/api/v1/Stavby/AdresniMisto/<RUIAN_KOD>"
   ```

Returns building info with `parcely[]`, `lv.cislo`, `definicniBod`, `jednotky`. To fetch full building detail:
```bash
katastr-cli api get "/api/v1/Stavby/<STAVBA_ID>"
```

## Units (apartments / non-residential)

```bash
katastr-cli api get "/api/v1/Jednotky/Vyhledani?KodCastiObce=<KOD>&TypStavby=1&CisloDomovni=<CP>&CisloJednotky=<CJ>"
katastr-cli api get "/api/v1/Jednotky/<JEDNOTKA_ID>"
```

## Proceedings (basic signal, not full legal extract)

By proceeding ID:
```bash
katastr-cli api get "/api/v1/Rizeni/<RIZENI_ID>"
```

By official identifier (for one-shot lookup of NOT-tracked proceedings; use `katastr-cli tracking show` for tracked ones):
```bash
katastr-cli api get "/api/v1/Rizeni/Vyhledani?TypRizeni=V&Cislo=<CISLO>&Rok=<ROK>&KodPracoviste=<KOD>"
```

Newest proceedings at a workplace ("co se dnes podalo"): use `Rizeni/PrijateDne`,
but call it **once per TypRizeni** - the endpoint accepts only `V`, `Z`, `PGP`
(it rejects `PD`/`ZPV`). Merge the lists, sort by `datumPrijeti` desc, take N.
```bash
for t in V Z PGP; do
  katastr-cli api get "/api/v1/Rizeni/PrijateDne?TypRizeni=$t&KodPracoviste=<KOD>&DatumPrijeti=<YYYY-MM-DD>"
done
```

## Code lists (decode codes for reports)

```bash
katastr-cli api get "/api/v1/CiselnikyUzemnichJednotek/Obce"
katastr-cli api get "/api/v1/CiselnikyUzemnichJednotek/KatastralniUzemi"
katastr-cli api get "/api/v1/CiselnikyUzemnichJednotek/CastiObci"
katastr-cli api get "/api/v1/CiselnikyISKN/DruhyPozemku"
katastr-cli api get "/api/v1/CiselnikyISKN/TypyStavby"
katastr-cli api get "/api/v1/CiselnikyISKN/TypyJednotky"
katastr-cli api get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiStavby"
katastr-cli api get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiParcely"
katastr-cli api get "/api/v1/CiselnikyISKN/ZpusobyVyuzitiJednotky"
katastr-cli api get "/api/v1/CiselnikyISKN/ZpusobyOchrany"
katastr-cli api get "/api/v1/CiselnikyISKN/Pracoviste"
```

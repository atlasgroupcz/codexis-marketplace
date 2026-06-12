# RPO API — field reference

Base: `https://api.statistics.sk/rpo/v1` · Docs: https://susrrpo.docs.apiary.io/
Free, anonymous, CC-BY 4.0 (the `license` field is embedded in every response).
Data refreshed nightly from source registers (ORSR, živnostenský register, …).
All endpoints below are accessed through `orsr-cli`, never directly.

## GET /search

Query params (at least one required; they combine as logical AND — `orsr-cli
search` exposes each as a flag):

| Param | `orsr-cli search` flag | Meaning |
|---|---|---|
| `identifier` | `--ico` | IČO (8 digits; the CLI pads shorter inputs) |
| `fullName` | `--name` | name fragment; matches historical names too |
| `addressMunicipality` | `--municipality` | seat municipality |
| `addressStreet` | `--street` | seat street |
| `legalForm` | `--legal-form` | legal form text |
| `legalStatus` | `--legal-status` | legal status text |
| `sourceRegister` | `--source-register` | source register text |
| `mainActivity` | `--main-activity` | main economic activity text |
| `onlyActive` | `--active` | literal `true` — only active entities |
| `establishmentAfter` / `establishmentBefore` | `--establishment-after/-before` | YYYY-MM-DD |
| `terminationAfter` / `terminationBefore` | `--termination-after/-before` | YYYY-MM-DD |
| `dbModificationDateAfter` / `dbModificationDateBefore` | `--db-modification-after/-before` | YYYY-MM-DD |

Returns `{"results": [<entity>...], "license": "..."}` — each result is the same
shape as `/entity/{id}` below. No pagination metadata; at most 500 hits.

## GET /entity/{id}

`{id}` = RPO internal id (`results[].id` from search), not IČO. Optional query
params (exposed as `orsr-cli detail` flags): `showHistoricalData=true`
(`--history`), `showOrganizationUnits=true` (`--units`).

Top-level fields (every nested object carries `validFrom` / `validTo` — an entry
with `validTo` set is historical):

| Field | Content |
|---|---|
| `id`, `dbModificationDate` | RPO id, last DB refresh |
| `identifiers[]` | IČO assignments |
| `fullNames[]` | names incl. historical |
| `addresses[]` | registered office history (`street`, `buildingNumber`, `postalCodes`, `municipality`, `country`) |
| `legalForms[]` | code-list values, e.g. `{"value": "Spoločnosť s ručením obmedzeným", "code": "112"}` |
| `establishment`, `termination` | dates of creation / dissolution |
| `activities[]` | business activities (predmety podnikania) |
| `statutoryBodies[]` | `personName` (`formatedName`, `givenNames`, `familyNames`), `stakeholderType` (e.g. `Konateľ`), address |
| `stakeholders[]` | spoločníci / founders with stakes |
| `equities[]` | share capital, e.g. `{"value": 140000.00, "currency": "EUR"}` |
| `deposits[]` | individual vklady of spoločníci |
| `legalStatuses[]`, `otherLegalFacts[]` | liquidation, konkurz, other facts |
| `authorizations[]` | oprávnenia |
| `sourceRegister` | source register, court and number, e.g. Obchodný register, Mestský súd Bratislava III, `Sro/3586/B` |
| `predecessors[]` | legal predecessors |
| `statisticalCodes` | SK NACE, ESA2010, … |

Beneficial owners (koneční užívatelia výhod) are **not** in the public API.

## orsr.sk URL patterns (deep links only — no API)

`orsr-cli links <ico>` resolves these; the table documents what the URLs mean.
The HTML behind them is windows-1250 legacy markup — never parse the content.

| URL | Purpose |
|---|---|
| `https://www.orsr.sk/hladaj_ico.asp?ICO={ico}&SID=0` | search by IČO (`SID=0` = all courts) |
| `https://www.orsr.sk/hladaj_subjekt.asp?OBMENO={name}&PF=0&SID=0&R=on` | search by name prefix (`R=on` ≈ active only) |
| `https://www.orsr.sk/vypis.asp?ID={id}&SID={court}&P=0` | aktuálny výpis (`aktualnyVypis` in `links` output) |
| `https://www.orsr.sk/vypis.asp?ID={id}&SID={court}&P=1` | úplný výpis (`uplnyVypis` in `links` output) |

`ID`/`SID` are ORSR-internal and not derivable from RPO ids.

## Register účtovných závierok (registeruz.sk)

Free JSON API, no auth. Docs: https://www.registeruz.sk/cruz-public/home/api

`orsr-cli ruz <ico>` resolves the accounting-entity id and returns the entity
record: `ico`, `dic`, `nazovUJ`, `pravnaForma`, `skNace`,
`idUctovnychZavierok[]` (financial statements), `idVyrocnychSprav[]` (annual
reports). Underlying endpoints, should deeper data ever be needed:
`uctovne-jednotky?ico=` → `uctovna-jednotka?id=` → `uctovna-zavierka?id=` →
`uctovny-vykaz?id=` (statement line items).

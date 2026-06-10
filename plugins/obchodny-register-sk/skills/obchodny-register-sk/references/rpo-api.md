# RPO API — field reference

Base: `https://api.statistics.sk/rpo/v1` · Docs: https://susrrpo.docs.apiary.io/
Free, anonymous, CC-BY 4.0 (the `license` field is embedded in every response).
Data refreshed nightly from source registers (ORSR, živnostenský register, …).

## GET /search

Query params (one required):

| Param | Meaning |
|---|---|
| `identifier` | IČO (8 digits) |
| `fullName` | name fragment; matches historical names too |

Returns `{"results": [<entity>...], "license": "..."}` — each result is the same
shape as `/entity/{id}` below. No pagination metadata.

## GET /entity/{id}

`{id}` = RPO internal id (`results[].id` from search), not IČO.

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

HTML is windows-1250 encoded legacy markup; only extract links, never parse content.

| URL | Purpose |
|---|---|
| `https://www.orsr.sk/hladaj_ico.asp?ICO={ico}&SID=0` | search by IČO (`SID=0` = all courts) |
| `https://www.orsr.sk/hladaj_subjekt.asp?OBMENO={name}&PF=0&SID=0&R=on` | search by name prefix (`R=on` ≈ active only) |
| `https://www.orsr.sk/vypis.asp?ID={id}&SID={court}&P=0` | aktuálny výpis |
| `https://www.orsr.sk/vypis.asp?ID={id}&SID={court}&P=1` | úplný výpis (history) |

`ID`/`SID` are ORSR-internal — resolve them via `hladaj_ico.asp` first; they are
not derivable from RPO ids.

## Register účtovných závierok (registeruz.sk)

Free JSON API, no auth. Docs: https://www.registeruz.sk/cruz-public/home/api

```bash
# accounting entity ids by IČO
curl -s "https://www.registeruz.sk/cruz-public/api/uctovne-jednotky?zmenene-od=2000-01-01&ico=31333532&max-zaznamov=1"
# → {"id":[154048],"existujeDalsieId":false}

# entity detail (lists financial-statement ids)
curl -s "https://www.registeruz.sk/cruz-public/api/uctovna-jednotka?id=154048"
# → { "ico", "dic", "nazovUJ", "idUctovnychZavierok": [...], "idVyrocnychSprav": [...] }

# statement detail → report ids → actual line items
curl -s "https://www.registeruz.sk/cruz-public/api/uctovna-zavierka?id={id}"
curl -s "https://www.registeruz.sk/cruz-public/api/uctovny-vykaz?id={id}"
```

# ČÚZK REST API KN - domain command reference

## Contents

1. [How this works](#how-this-works)
2. [Response envelope](#response-envelope)
3. [Identifiers: `id`, `kod`, and human numbers](#identifiers-id-kod-and-human-numbers)
4. [Commands](#commands)
   - [`search`](#search---resolve-text-to-a-ruian-code)
   - [`parcela`](#parcela---land-parcel)
   - [`okoli`](#okoli---whats-nearby)
   - [`stavba`](#stavba---building)
   - [`jednotka`](#jednotka---unit-flat-or-non-residential-space)
   - [`rizeni`](#rizeni---proceeding)
   - [`pravo-stavby`](#pravo-stavby---right-of-superficies)
   - [`sousedni`](#sousedni---neighboring-parcels)
   - [`pracoviste`](#pracoviste---cadastral-workplace-directory)
5. [Workflow: "is it clean?"](#workflow-is-it-clean)
6. [Workflow: find a building by address](#workflow-find-a-building-by-address)
7. [`katastr-cli api get` - escape hatch](#katastr-cli-api-get---escape-hatch)

## How this works

Every `katastr-cli` command below builds its own ČÚZK request URL from the
arguments you pass - you never compose a path by hand. Output is the ČÚZK
response, printed verbatim as pretty JSON; read it directly or filter with
`jq` (never `sed`/`grep` on JSON).

```bash
katastr-cli parcela --ku 638790 --cislo 545 | jq '.data[0].id'
```

The machine-readable specification is at
`https://api-kn.cuzk.gov.cz/swagger/v1.0/swagger.json` (note the `v1.0` -
`v1` returns 404). Use it to look up an enum or a field you don't see below;
don't call `curl` against it or anything else yourself - `katastr-cli` is the
only thing that talks to ČÚZK.

## Response envelope

Most commands' output is the raw ČÚZK envelope, passed through verbatim.
Three cases compose their own answer instead, because there is no single
upstream response to pass through: `search` (translates a geocoder call plus
a layer-query call into one result), `okoli` (combines a center lookup with
two polygon queries), and `pravo-stavby`'s null case (a 404 recoded as a
`{"pravoStavby": null, "dotaz": {...}}` result). Their sections below show
the actual shape. Everything else follows the envelope described here:

- `data` - the payload. A single object for an `--id`/`--adresni-misto`-style
  lookup; a list for a `Vyhledani` (search-by-attributes) lookup. Each
  command's section below says which shape to expect.
- `aktualnostDatK` - timestamp the data is current as of. Cite this alongside
  any fact you report from the response - it is the "as-of" date the user
  needs for anything with legal weight.
- `provedenoVolani` - running count of calls made with this API key.
- `zpravy` - operational messages, each `{kod, typZavaznosti, text}` with
  `typZavaznosti` one of `Informace`, `Varovani`, `Chyba`. **Always read
  `zpravy` and pass on anything in it to the user** - this is where ČÚZK
  reports things nothing else surfaces, including an approaching API-key
  expiry:

  ```json
  "zpravy": [
    { "kod": 201, "typZavaznosti": "Varovani",
      "text": "Blíží se konec platnosti ApiKey [12.08.2026 05:40:35]" }
  ]
  ```

  A `Varovani`/`Chyba` message here is not a network or usage error - the
  call still succeeded and `data` is still valid - but the message itself is
  information the user needs, so surface it instead of silently discarding it.

## Identifiers: `id`, `kod`, and human numbers

Three different kinds of identifier show up across these commands - don't
mix them up:

- **`id`** - the internal ISKN object identifier (e.g. `2034508804`). Scoped
  to one object (a parcel, building, unit, proceeding, or right of
  superficies) and stable for that object. You get it from a `Vyhledani`
  (search-by-attributes) response's `data[...].id`, then use it directly with
  that command's `--id` flag for a one-shot detail lookup.
- **`kod`** - a public code from RUIAN (the address/territorial-unit
  registry), e.g. an address-point code like `8189986`, or a
  katastrální-území / část-obce / pracoviště code. You get an address-point
  `kod` from `search`; territorial-unit codes come from the code-list
  endpoints (see `katastr-cli api get "/api/v1/CiselnikyUzemnichJednotek/..."`).
- **Human numbers** - číslo popisné/evidenční, číslo LV, číslo řízení
  (`V-123/2026-701`), katastrální-území name. These are what a person reads
  off a document, and **no human number is globally unique on its own** - a
  číslo popisné repeats across different katastrální území, a číslo LV
  repeats across different katastrální území, and so on. Always resolve a
  human number together with the territorial context it came with (k.ú.,
  část obce, pracoviště) rather than treating the bare number as an
  identifier.

## Commands

### `search` - resolve text to a RUIAN code

```
katastr-cli search <TEXT> [--typ <TYPE>]
katastr-cli search --klic <KLIC>
```

Turns something a user typed into a RUIAN `kod` that other commands take. It
talks to `ags.cuzk.gov.cz` (RUIAN over ArcGIS), not the KN API, so it does not
count against the call quota.

RUIAN keeps each kind of thing in its own register, so **`--typ` decides which
one is searched**. With it, the register answers directly: no result cap,
filtering on the server, every match returned with its code.

```bash
katastr-cli search "Nová Ves" --typ Obec
```
```json
{ "typ": "Obec", "vrstva": 12, "dotaz": "Nová Ves", "pocet": 14,
  "nalezy": [
    { "kod": 583502, "nazev": "Nová Ves", "kontext": "Brno-venkov" },
    { "kod": 596213, "nazev": "Nová Ves", "kontext": "Žďár nad Sázavou" } ] }
```

Fourteen municipalities are called Nová Ves. `kontext` is the parent that tells
them apart - the district for a municipality, the municipality for a cadastral
area or a city district. **Always show it to the user and let them choose**;
never take the first row as the answer.

`--typ` accepts: `AdresniMisto`, `Ulice`, `ZakladniSidelniJednotka`,
`KatastralniUzemi`, `MestskyObvodMestskaCastVeStatutarnimMesteAHlavnimMestePraze`,
`SpravniObvodVHlavnimMestePraze`, `CastObce`, `Obec`,
`ObecSPoverenymObecnimUradem`, `ObecSRozsirenouPusobnosti`, `Okres`,
`VyssiUzemneSamospravnyCelek`. A parcel typed as free text
(`ParcelaDefinicniBod`) has no RUIAN register - use `parcela --ku`/`--cislo`.

**Without `--typ`** the kind is what has to be found first, so the answer is a
list of suggestions rather than codes. Each carries its kind and a `klic`:

```bash
katastr-cli search "Nová Ves"
```
```json
{ "navrhy": [
    { "text": "Nová Ves I (Kolín)", "typ": "Obec", "klic": "12_697" },
    { "text": "Nová Ves (Žďár nad Sázavou)", "typ": "Obec", "klic": "12_10108" } ] }
```

`klic` names one exact record, so feeding it back needs no name matching:

```bash
katastr-cli search --klic 12_10108
```
```json
{ "typ": "Obec", "vrstva": 12,
  "nalezy": [ { "kod": 596213, "nazev": "Nová Ves", "kontext": "Žďár nad Sázavou" } ] }
```

An address is filed under its street, so what a user types is never what is
stored - `Hladke Zivotice 66` lives as `Malá Strana 66, 74247 Hladké Životice`.
`search` handles the translation itself, missing diacritics included:

```bash
katastr-cli search "hladke zivotice 66" --typ AdresniMisto
```
```json
{ "typ": "AdresniMisto", "vrstva": 1, "dotaz": "hladke zivotice 66", "pocet": 1,
  "nalezy": [ { "kod": 8189986, "nazev": "Malá Strana 66, 74247 Hladké Životice" } ] }
```

Take `nalezy[].kod` and feed it to `stavba --adresni-misto <KOD>`.

### `parcela` - land parcel

```
katastr-cli parcela --ku <KOD> --cislo <N> [--poddeleni <N>] [--druh 1|2] [--typ PKN|PZE]
                    [--puvod-ze 3|4|6]   # povinné u --typ PZE:
                                         # 3 evidence nemovitostí, 4 pozemkový katastr,
                                         # 6 přídělový plán nebo jiný podklad
katastr-cli parcela --id <ID>
```

Numeric arguments are range-checked against what the API declares before any
call is made (`--ku` 1-999999, `--cislo` 1-99999, `--poddeleni` 1-999,
`--cast-obce` 1-999999, `--cp`/`--ce` 1-99999, `--cj` 0-99999999,
`--pracoviste` and `pracoviste --kod` 1-999), so an out-of-range value costs no
quota.

Pass exactly one of `--id` or `--ku`/`--cislo` together - combining them is
rejected as a contradiction. `--ku`/`--cislo` searches by katastrální-území
code and parcel number (`--druh` defaults to `2`, pozemková parcela; use `1`
for stavební; for a number like `2642/9` pass `--cislo 2642 --poddeleni 9`)
and returns a **list**:

```bash
katastr-cli parcela --ku 638790 --cislo 545
```
```json
{ "data": [ { "id": 2034508804, "vymera": 138, "lv": {"cislo": 1064, ...},
              "rizeniPlomby": [], "zpusobyOchrany": [], "pravoStavby": null,
              "stavba": {"id": 387773804, ...}, ... } ], ... }
```

`--id` fetches one already-known parcel directly and returns a single
**object** (not a list) at `data`:

```bash
katastr-cli parcela --id 2034508804
```

For nearby parcels and buildings around this one, use the standalone
[`okoli`](#okoli---whats-nearby) command with `--parcela <ID>`.

### `okoli` - what's nearby

```
katastr-cli okoli --parcela <ID> --metru <M>
katastr-cli okoli --stavba <ID> --metru <M>
katastr-cli okoli --adresa <TEXT> --metru <M>
```

Pass exactly one of `--parcela`, `--stavba`, `--adresa` as the center and
`--metru` as the radius in meters. `--adresa` resolves the text through
`search --typ AdresniMisto` first, then uses that building's location; if the
address doesn't resolve to a single confident match, use `--stavba` with a
concrete ID instead (see [`search`](#search---resolve-text-to-a-ruian-code)).
The command builds a square of that radius around the center's
`definicniBod` and returns nearby parcels and buildings found inside it:

```bash
katastr-cli okoli --parcela 2034508804 --metru 20
```
```json
{
  "metru": 20.0,
  "parcely": { "data": [ { "id": 2034508804, "kmenoveCisloParcely": 545, ... },
                          { "id": 67786064010, "kmenoveCisloParcely": 547, ... } ], ... },
  "stavby": { "data": [ { "id": 387773804, "cislaDomovni": [66], ... } ], ... },
  "stred": { "x": -494085.17, "y": -1116407.13 }
}
```

`parcely` and `stavby` are each a full ČÚZK response envelope (see
[Response envelope](#response-envelope)), so read `zpravy` on both.

ČÚZK caps the query area at 5000 m², which in practice means **`--metru`
values above about 35 stop working**: a 36 m square already exceeds the
5000 m² polygon-area limit and the call fails with HTTP 400 "Maximum surface
area for polygon is 5000 m^2". Keep it at 35 or less.

### `stavba` - building

```
katastr-cli stavba --adresni-misto <KOD>
katastr-cli stavba --cast-obce <KOD> --cp <N>
katastr-cli stavba --cast-obce <KOD> --ce <N>
katastr-cli stavba --id <ID>
```

Pass exactly one addressing mode: `--id`, `--adresni-misto`, or
`--cast-obce` together with exactly one of `--cp`/`--ce`. `--cp` is číslo
popisné, `--ce` is číslo evidenční. Combining modes (or both `--cp` and
`--ce`) is rejected as a contradiction. `--adresni-misto` takes the `kod` a
`search --typ AdresniMisto` call resolved:

```bash
katastr-cli stavba --adresni-misto 8189986
```
```json
{
  "data": {
    "id": 387773804,
    "lv": { "cislo": 1064, "id": 66030870010, "katastralniUzemi": {"kod": 638790, "nazev": "Hladké Životice"} },
    "parcely": [ { "id": 2034508804, "kmenoveCisloParcely": 545, ... } ],
    "jednotky": [],
    "rizeniPlomby": [], "zpusobyOchrany": [], "pravoStavby": null
  },
  "zpravy": [ { "kod": 201, "typZavaznosti": "Varovani",
                "text": "Blíží se konec platnosti ApiKey [12.08.2026 05:40:35]" } ]
}
```

This API has no owners. Present `lv.cislo` and point the user to an official
LV extract for ownership - there is no LV endpoint in this API (see
[Identifiers](#identifiers-id-kod-and-human-numbers)).

### `jednotka` - unit (flat or non-residential space)

```
katastr-cli jednotka --cast-obce <KOD> --cp <N> --cj <N>
katastr-cli jednotka --id <ID>
```

Pass exactly one of `--id` or `--cast-obce`/`--cp`/`--cj` together -
combining them is rejected as a contradiction. `--cast-obce`/`--cp`/`--cj`
searches by část-obce code, číslo popisné, and číslo jednotky and returns a
list. Most buildings are not split into `jednotka` records - only units
under a formal ownership split (e.g. an SVJ) are - so a plausible-looking
number commonly comes back empty, which is not an error:

```bash
katastr-cli jednotka --cast-obce 414085 --cp 840 --cj 1
```
```json
{ "data": [], "provedenoVolani": 124, "zpravy": [...] }
```

When a number does match, `data[0]` carries `cisloJednotky`, `typJednotky`,
`zpusobVyuziti`, `lv`, `vymezenaVeStavbe` (the containing building), and
`rizeniPlomby` - same shape as `parcela`/`stavba`. `--id` fetches one unit
directly and returns a single object.

### `rizeni` - proceeding

```
katastr-cli rizeni <CISLO>
katastr-cli rizeni --id <ID>
katastr-cli rizeni --prijate-dne <DATE> --pracoviste <KOD> --typ V|Z|PGP
```

`<CISLO>` is the canonical `TYP-CISLO/ROK-PRACOVISTE` form (e.g.
`V-6051/2026-804`); it searches and returns a list:

```bash
katastr-cli rizeni V-6051/2026-804
```
```json
{
  "data": [ {
    "id": 108667944010, "typRizeni": "V", "poradoveCislo": 6051, "rok": 2026,
    "kodPracoviste": 804, "datumPrijeti": "2026-07-29T07:04:06",
    "stav": "Obeslání účastníků řízení", "stavUhrady": "N",
    "provedeneOperace": [ {"nazev": "Založení řízení", "datumProvedeni": "..."}, ... ],
    "poznamky": ["U řízení typu V se neposkytují navázaná řízení"],
    "navazanaRizeni": null
  } ]
}
```

`--id` fetches one proceeding directly (single object). `--prijate-dne`
lists proceedings a workplace received on a given date - **the server
accepts only `V`, `Z`, `PGP`** for `--typ` here (`katastr-cli` rejects `PD`
and `ZPV` itself before calling ČÚZK: the API's own specification wrongly
lists all five proceeding types for this endpoint, but the server answers
`PD`/`ZPV` with HTTP 400). To cover proceedings received at a workplace
across all types, call once per type in `V`, `Z`, `PGP` and merge.

### `pravo-stavby` - right of superficies

```
katastr-cli pravo-stavby <ID>
katastr-cli pravo-stavby --parcela <ID>
katastr-cli pravo-stavby --stavba <ID>
```

Pass exactly one of the identifier, `--parcela`, or `--stavba` - combining
two (e.g. both `--parcela` and `--stavba`) is rejected as a contradiction
instead of silently answering for one and ignoring the other.

Most parcels and buildings have no right of superficies - that is the normal
case, not an error. When there is none, `katastr-cli` prints
`{"pravoStavby": null, "dotaz": {...}}` and **exits 0**; `dotaz` names
whichever identifier was actually checked, so the null answer is
self-identifying even out of context:

```bash
katastr-cli pravo-stavby --parcela 2034508804
```
```json
{ "pravoStavby": null, "dotaz": { "parcela": "2034508804" } }
```

When one exists, `data` carries `ucelyPravaStavby`, `lv`, `parcely`,
`stavby`, `zpusobyOchrany`, `rizeniPlomby`, `datumPrijeti`, and
`datumUkonceni`. A genuinely missing parcel/building still fails as an
`ERROR:` (HTTP 404 with a different message) - only "found, but no right" is
folded into the null case above.

### `sousedni` - neighboring parcels

```
katastr-cli sousedni <ID>
```

Topologically adjacent parcels (sharing a border), by parcel `id`:

```bash
katastr-cli sousedni 2034508804
```
```json
{ "data": [ { "id": 2859071804, "kmenoveCisloParcely": 546, "poddeleniCislaParcely": 1, ... }, ... ] }
```

An empty list is a valid result (e.g. a territory without a digital
neighbor map), not an error. For a distance-based search instead of
topological adjacency, use [`okoli`](#okoli---whats-nearby) with
`--parcela <ID>`.

### `pracoviste` - cadastral workplace directory

```
katastr-cli pracoviste [--kod <KOD>]
```

Without `--kod`, lists all cadastral workplaces; with it, one workplace's
detail:

```bash
katastr-cli pracoviste --kod 802
```
```json
{ "data": { "kod": 802, "nazev": "Katastrální úřad pro Moravskoslezský kraj, Katastrální pracoviště Frýdek-Místek",
            "idds": "ivbhu6z", "email": "kp.fmistek@cuzk.gov.cz", "telefon": "558604000",
            "kodKraje": 132, "kodOkresu": 3802, "kodNadrizenehoPracoviste": 800 } }
```

`kod` here is the `KodPracoviste` used by `rizeni --pracoviste`.

## Workflow: "is it clean?"

"Čisté" usually means one of two different things - clarify with the user
which one they mean before answering:

- **no pending proceeding** (plomba/řízení) - answerable from this API.
- **no lien/easement/rights restriction** - **not** available from this API;
  needs an official LV extract.

For the first meaning:

1. Resolve the parcel: `parcela --ku <KOD> --cislo <N>` (or `--id` if already
   known). `--druh` defaults to `2` (pozemková parcela); a stavební parcela
   needs `--druh 1`. An empty `data` array means **"not found with these
   parameters, try the other `--druh`"**, not "no such parcel" - same
   treatment as the empty list `jednotka` returns (see
   [`jednotka`](#jednotka---unit-flat-or-non-residential-space)): if you're
   not sure which kind the parcel is, try both `--druh` values before
   concluding it doesn't exist.
2. From the parcel's `data`, check:
   - `rizeniPlomby` - empty means no pending proceeding signal on the parcel.
   - `zpusobyOchrany` - a protection/limit, not a plomba; report separately.
   - `pravoStavby` - already present on this response; `null` means no right
     of superficies on the parcel, anything else means there is one and it
     belongs in the answer. Only call `pravo-stavby --parcela <ID>`
     separately if you want the full detail (`ucelyPravaStavby`, `lv`, ...) -
     against a 500-call daily quota, don't spend a call re-fetching a field
     you already have.
   - `stavba.id`, if present → `stavba --id <ID>` and check `rizeniPlomby`
     and `pravoStavby` there too - a building can carry its own right of
     superficies independently of the parcel's, so the parcel's field alone
     doesn't cover it.
3. Read `zpravy` on every response along the way and pass on anything in it
   (see [Response envelope](#response-envelope)).

Report all three signals (`rizeniPlomby`, `zpusobyOchrany`, `pravoStavby`)
together - "clean" only holds if all three come back empty/null.

## Workflow: find a building by address

```bash
katastr-cli search "<ADDRESS>" --typ AdresniMisto
# take the top-level `kod` from the result, then:
katastr-cli stavba --adresni-misto <KOD>
```

`search` without `--typ` is intentionally cautious - see
[`search`](#search---resolve-text-to-a-ruian-code) above - so pin the type
to `AdresniMisto` to get a single resolvable RUIAN code for a postal address.

## `katastr-cli api get` - escape hatch

```
katastr-cli api get "<PATH>"
```

Authenticated GET straight against `api-kn.cuzk.gov.cz`, returning the JSON
body verbatim. Use this **only** when the task needs a ČÚZK endpoint none of
the commands above cover (check the swagger spec linked at the top of this
file for the full path list). If one of the named commands above fits, use
that instead - it builds the URL for you and cannot typo a parameter name.

A handful of endpoints are deliberately reachable only through `api get` -
no named command wraps them:

| Path | Returns |
| --- | --- |
| `CiselnikyUzemnichJednotek/Kraje` | List of all kraje (`kod`, `nazev`, `nutS3`). |
| `CiselnikyUzemnichJednotek/Kraje/{kod}` | One kraj by its code. |
| `CiselnikyUzemnichJednotek/Okresy` | List of all okresy (`kod`, `nazev`, `kodKraje`, `nutS4`). |
| `CiselnikyUzemnichJednotek/Okresy/{kod}` | One okres by its code. |
| `CiselnikyISKN/ZpusobyUrceniVymery` | Code list of ways a parcel's `vymera` (area) was determined. |
| `AplikacniSluzby/CiselnikZprav` | Catalog of `zpravy` message codes/templates (see [Response envelope](#response-envelope)) - a bare JSON array, not the usual `data` envelope. |

For the territorial code lists (kraje, okresy), `search` covers the same
ground better in normal use - it filters server-side and does not count
against the API-call quota - so reach for `api get` on these only when the
task genuinely needs the whole list, not just one kraj/okres.

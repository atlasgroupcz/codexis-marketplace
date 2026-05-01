---
uuid: 214caaa4-3728-4d21-b379-ab4b376b7615
name: ares
description: Český registr ekonomických subjektů (ARES, MF ČR). Use for company / sole-trader lookups by IČO or name — basic identification, business register (statutory bodies, partners), statistical register (RES), trade licence registry (ŽR), and VAT payer registry. Triggers on "ares", "ičo", "ico", "obchodní rejstřík", "živnostenský rejstřík", "plátce DPH", "registr ekonomických subjektů", "vyhledej firmu", "kdo je jednatel", "statutární orgán", "ověř firmu", "DIČ".
version: 0.1.0
i18n:
  cs:
    displayName: "ARES — registr ekonomických subjektů"
    summary: "Vyhledávání firem a OSVČ v ARES — IČO, obchodní rejstřík, RES, živnostenský rejstřík, plátci DPH."
  en:
    displayName: "ARES — Czech Business Registry"
    summary: "Look up companies and sole traders in ARES — ICO, business register, statistical register, trade licence registry, VAT payers."
  sk:
    displayName: "ARES — register ekonomických subjektov"
    summary: "Vyhľadávanie firiem a SZČO v ARES — IČO, obchodný register, RES, živnostenský register, platcovia DPH."
---

# ARES — Czech Business Registry

A single tool — **`ares-cli`** — wraps the ARES v3 public REST API
(`https://ares.gov.cz/ekonomicke-subjekty-v-be/rest`).

**IMPORTANT:** The only tool in this skill is `ares-cli`. Do NOT call `curl`, `kn`,
`cdx-cli`, or any other tool directly. Assume `ares-cli` is installed and available
in `PATH`.

**IMPORTANT:** If `ares-cli` outputs an `ERROR:` line (e.g. `HTTP 404 …`), stop
immediately and report it to the user. Do not retry blindly or guess workarounds —
the message text from ARES is authoritative.

## Output Format

`ares-cli` always prints **JSON to stdout** (verbatim ARES response). Parse with
`jq`. Never use `sed` or `grep` — fields are nested objects.

```bash
ares-cli detail 27082440 | jq '.obchodniJmeno'
ares-cli search --name "ATLAS" --pocet 5 | jq '.ekonomickeSubjekty[] | {ico, obchodniJmeno}'
```

On failure: `ERROR: HTTP <code> <reason>` on stderr, exit code 2. The body of
the ARES error response is appended verbatim — read it before retrying.

## Three commands

```bash
ares-cli detail <ICO> [--register basic|vr|res|rzp]
ares-cli search [filters …]
ares-cli api {get|post} <PATH> [<JSON_BODY>]
```

`<ICO>` accepts 1–8 digit input; `ares-cli` left-pads to 8 digits before the call.

### Registers (`--register`)

| Value     | Endpoint family                 | What it returns                                                              |
|-----------|----------------------------------|------------------------------------------------------------------------------|
| `basic`   | `/ekonomicke-subjekty`           | Default identification record (name, address, legal form, datum vzniku/zániku). |
| `vr`      | `/ekonomicke-subjekty-vr`        | Veřejný (obchodní) rejstřík — statutory bodies, partners (společníci), capital. |
| `res`     | `/ekonomicke-subjekty-res`       | Registr ekonomických subjektů (statistical: NACE, počet zaměstnanců, …).     |
| `rzp`     | `/ekonomicke-subjekty-rzp`       | Živnostenský rejstřík — list of trade licences (živnosti).                   |

> **Pozor:** ARES v3 nemá samostatný endpoint pro Registr plátců DPH. DIČ a
> aktuální stav plátcovství najdeš v **základním detailu** (`basic`) v polích
> `dic` a `seznamRegistraci.stavZdrojeDph` (`AKTIVNI` / `UKONCENY` /
> `NEEXISTUJICI`). Skupinová registrace DPH: `seznamRegistraci.stavZdrojeSkDph`.

### Search filters (`ares-cli search`)

At least one of `--name`, `--ico`, `--obec`, `--psc`, `--pravni-forma`, `--okres`
is required. Paging via `--start` (default 0) and `--pocet` (default 10, max 1000).

```bash
ares-cli search --name "ATLAS GROUP"
ares-cli search --obec "Praha" --pravni-forma 112 --pocet 20
ares-cli search --psc 11000 --name "advokát"
```

`--pravni-forma` takes the ARES legal-form code (e.g. `112` = s.r.o., `121` = a.s.,
`101` = OSVČ podnikající dle živnostenského zákona).

### Raw API

```bash
ares-cli api get "/ekonomicke-subjekty/27082440"
ares-cli api post "/ekonomicke-subjekty/vyhledat" '{"obchodniJmeno":"ATLAS","pocet":5}'
```

Use only when the structured `detail`/`search` commands cannot express the request.

## Decision tree

**User has the IČO and just wants identification?**
→ `ares-cli detail <ICO>`

**User asks "kdo je jednatel/statutár firmy X" or about partners (společníci)?**
→ `ares-cli detail <ICO> --register vr`

**User asks "je X plátce DPH" / "od kdy plátce DPH" / "DIČ"?**
→ `ares-cli detail <ICO>` a vyzobni `dic` + `seznamRegistraci.stavZdrojeRpdph`
   (ARES v3 nemá samostatný RPDPH endpoint).

**User asks about živnosti (trade licences) of a sole trader / company?**
→ `ares-cli detail <ICO> --register rzp`

**User asks about NACE / počet zaměstnanců / statistical classification?**
→ `ares-cli detail <ICO> --register res`

**User does NOT know the IČO and gives a name/address?**
→ `ares-cli search --name "..."` (combine with `--obec` / `--psc` if available
to narrow down). Pick the right candidate from `ekonomickeSubjekty[]`, then run
`detail` on that IČO.

**User wants a list of companies in a region / by legal form?**
→ `ares-cli search --obec "..." --pravni-forma <CODE> --pocet 100` (paginate
with `--start` if needed).

## User-facing output rules

- Always show **`obchodniJmeno`** + **`ico`** as the canonical company identifier.
- Address comes from `sidlo.textovaAdresa` (preformatted by ARES) — prefer it
  over reconstructing from parts.
- For statutory bodies / partners (`vr`), data sit under `zaznamy[].statutarniOrgan`
  / `zaznamy[].spolecnici`. Iterate, don't assume single record.
- Never fabricate data ARES did not return — if a field is missing, say so.

## Examples

```bash
# Verify counterparty before a contract
ares-cli detail 27082440 | jq '{ico, name: .obchodniJmeno, address: .sidlo.textovaAdresa, active: (.datumZaniku == null)}'

# Statutory bodies of a s.r.o.
ares-cli detail 27082440 --register vr | jq '.zaznamy[].statutarniOrgan'

# Is this company a VAT payer right now? (ARES v3 has no separate RPDPH endpoint)
ares-cli detail 27082440 | jq '{dic, dph: .seznamRegistraci.stavZdrojeDph, dphSkup: .seznamRegistraci.stavZdrojeSkDph}'

# Find advokátní kanceláře v Praze
ares-cli search --obec "Praha" --name "advokát" --pocet 50 \
  | jq '.ekonomickeSubjekty[] | {ico, obchodniJmeno, sidlo: .sidlo.textovaAdresa}'
```

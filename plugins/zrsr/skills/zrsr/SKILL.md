---
uuid: 7a5c2a2e-0e89-4c93-841a-0788e7aa8f13
name: zrsr
description: Slovenský register právnických osôb a podnikateľov (RPO, Štatistický úrad SR), ktorý agreguje aj Živnostenský register SR (zrsr.sk) a Obchodný register. Use for Slovak company / sole-trader (živnostník, SZČO) lookups by IČO or name — identification, trade licences (predmety podnikania), source register, addresses, legal form, name history. Triggers on "zrsr", "rpo", "živnostenský register", "živnostník", "živnosť", "SZČO", "IČO" (Slovak entity), "obchodný register SR", "vyhľadaj živnostníka", "over firmu na Slovensku", "živnostenské oprávnenie", "slovenská firma".
version: 0.1.0
jurisdictions: [SK]
i18n:
  cs:
    displayName: "ZRSR — živnostenský registr SR"
    summary: "Vyhledávání slovenských firem a živnostníků v RPO — IČO, živnostenská oprávnění, živnostenský a obchodní registr."
  en:
    displayName: "ZRSR — Slovak Trade Register"
    summary: "Look up Slovak companies and sole traders in RPO — IČO, trade licences, Trade and Business Register."
  sk:
    displayName: "ZRSR — živnostenský register SR"
    summary: "Vyhľadávanie slovenských firiem a živnostníkov v RPO — IČO, živnostenské oprávnenia, živnostenský a obchodný register."
---

# ZRSR — Slovak Trade & Business Register

A single tool — **`zrsr-cli`** — wraps the public RPO API of the Slovak
Statistical Office (`https://api.statistics.sk/rpo/v1`). RPO (Register
právnických osôb a podnikateľov) aggregates the official source registers,
including the **Živnostenský register SR** (zrsr.sk, which itself has no API)
and the **Obchodný register**.

**IMPORTANT:** The only tool in this skill is `zrsr-cli`. Do NOT call `curl`,
`kn`, `cdx-cli`, or any other tool directly. Assume `zrsr-cli` is installed and
available in `PATH`.

**IMPORTANT:** If `zrsr-cli` outputs an `ERROR:` line (e.g. `HTTP 404 …`), stop
immediately and report it to the user. Do not retry blindly or guess
workarounds — the message text from RPO is authoritative.

## Output Format

`zrsr-cli` always prints **JSON to stdout** (verbatim RPO response; only
`--limit` rewrites the `results` array). Parse with `jq`. Never use `sed` or
`grep` — fields are nested objects.

```bash
zrsr-cli detail 686930 | jq '.fullNames, .legalForms'
zrsr-cli search --name "Tatra banka" --limit 5 | jq '.results[] | {id, ico: .identifiers[0].value}'
```

On failure: `ERROR: HTTP <code> <reason>` on stderr, exit code 2. The body of
the RPO error response is appended verbatim — read it before retrying.

## Three commands

```bash
zrsr-cli detail <ICO> | --id <RPO_ID> [--history] [--units]
zrsr-cli search [filters …]
zrsr-cli api get <PATH>
```

`<ICO>` accepts 1–8 digit input; `zrsr-cli` left-pads to 8 digits before the
call. `detail <ICO>` makes two API calls (identifier search → entity fetch);
the identifier search can take **tens of seconds** — be patient, don't kill it.
When the internal RPO `id` is already known (from a previous `search`), prefer
`detail --id <RPO_ID>` — it skips the slow hop. `--history` adds historical
records (`showHistoricalData`), `--units` adds organization units.

## Search filters (`zrsr-cli search`)

All filters combine as logical AND and run server-side; at least one of the
first five (value filters) is required:

| Flag | RPO parameter | Matching |
|------|---------------|----------|
| `--name` | `fullName` | substring; matches historical names too |
| `--ico` | `identifier` | exact, left-padded to 8 digits |
| `--obec` (alias `--municipality`) | `addressMunicipality` | registered address municipality |
| `--street` | `addressStreet` | registered address street |
| `--pravna-forma` (alias `--legal-form`) | `legalForm` | legal form text, e.g. "Spoločnosť s ručením obmedzeným" |
| `--register zr\|or\|all` | `sourceRegister` | Živnostenský / Obchodný register (default `all` = no filter) |
| `--only-active` | `onlyActive` | only subjects without a termination date |
| `--limit N` | — | client-side truncation of `results` |

> **Pozor:** the RPO API caps every search at **500 records** with no paging.
> Narrow broad queries with extra filters (`--obec`, `--register`,
> `--only-active`) instead of trying to page.

```bash
zrsr-cli search --name "Tatra banka" --limit 5
zrsr-cli search --name "Kováč" --obec "Žilina" --register zr --only-active
zrsr-cli search --obec "Bratislava" --pravna-forma "Spoločnosť s ručením obmedzeným" --limit 20
```

## RPO data semantics

- Nearly every attribute is a **validFrom/validTo history array**
  (`fullNames`, `addresses`, `legalForms`, `identifiers`, `activities`, …).
  The current value is the entry **without `validTo`**; entries with `validTo`
  are historical.
- `activities[]` = predmety podnikania / **trade licences** (each with its own
  validity window).
- `sourceRegister.value.value` tells you which register the subject lives in
  ("Živnostenský register" / "Obchodný register"), with `registrationOffices`
  and `registrationNumbers` alongside.
- `establishment` / `termination` = vznik / zánik; a subject is **active** when
  `termination` is absent.
- `identifiers[].value` = IČO; top-level `id` = internal RPO id, reusable with
  `detail --id` and `/entity/{id}`.

## Raw API

```bash
zrsr-cli api get "/entity/4445617"
zrsr-cli api get "/search?fullName=Tatra%20banka"
```

Use only when the structured `detail`/`search` commands cannot express the
request (RPO is GET-only).

## Decision tree

**User has the IČO and wants identification / detail?**
→ `zrsr-cli detail <ICO>`

**User asks about živnosti / predmety podnikania (trade licences)?**
→ `zrsr-cli detail <ICO>` a vyzobni `activities` (current = bez `validTo`).

**User asks "je subjekt aktívny / zaniknutý" or "v akom registri je zapísaný"?**
→ `zrsr-cli detail <ICO>`, check `termination` + `sourceRegister`.

**User does NOT know the IČO and gives a name (+ municipality)?**
→ `zrsr-cli search --name "..."` (combine with `--obec` to narrow down).
Pick the right candidate from `results[]`, then `zrsr-cli detail --id <id>`.

**User wants živnostníci (sole traders) specifically?**
→ `zrsr-cli search … --register zr`.

**User wants a list of companies in a municipality / by legal form?**
→ `zrsr-cli search --obec "..." --pravna-forma <CODE> --limit 50`.

## Chat sources

`zrsr-cli detail` and `zrsr-cli search` automatically attach the public RPO
record of each looked-up subject (`https://api.statistics.sk/rpo/v1/entity/<id>`)
to the chat "Sources" panel — no extra step needed. When naming a subject in
your answer, cite that same URL in prose.

## User-facing output rules

- Always show the **current `fullNames` entry** + **IČO** (`identifiers[].value`)
  as the canonical subject identifier.
- Pick the history entry without `validTo`; if only historical values exist,
  say so explicitly.
- Addresses are structured (`street`, `buildingNumber`, `postalCodes`,
  `municipality.value`) — compose them, don't dump raw JSON at the user.
- Iterate history arrays, don't assume a single record.
- Never fabricate data RPO did not return — if a field is missing, say so.

## Examples

```bash
# Verify counterparty before a contract
zrsr-cli detail 686930 | jq '{ico: .identifiers[-1].value, name: (.fullNames[] | select(.validTo == null) | .value), active: (.termination == null), register: .sourceRegister.value.value}'

# Trade licences (predmety podnikania) of a subject — current only
zrsr-cli detail 686930 | jq '[.activities[] | select(.validTo == null) | .economicActivityDescription]'

# Find a živnostník by name and municipality
zrsr-cli search --name "Kováč" --obec "Žilina" --register zr --only-active --limit 10 \
  | jq '.results[] | {id, ico: .identifiers[-1].value, name: .fullNames[-1].value}'

# Current legal form
zrsr-cli detail 686930 | jq '.legalForms[] | select(.validTo == null) | .value.value'
```

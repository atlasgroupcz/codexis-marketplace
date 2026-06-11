---
uuid: 499b1cf8-5a26-49ef-8840-d664fdd30582
name: obchodny-register-sk
description: Slovenský obchodný register a register právnických osôb (RPO, Štatistický úrad SR). Use for Slovak company / legal-entity lookups by IČO or name — identification, registered office, statutory bodies (konatelia, predstavenstvo), partners (spoločníci), share capital (základné imanie), legal form, registration court and number, and full historical changes. Triggers on "obchodný register", "ORSR", "RPO", "slovenská firma", "slovenské IČO", "výpis z obchodného registra", "konateľ", "spoločník", "základné imanie", "register právnických osôb", "sídlo firmy na Slovensku". Slovak (SK) jurisdiction only — for Czech companies use the ares skill.
version: 1.2.0
jurisdictions: [SK]
i18n:
  cs:
    displayName: "Obchodný register SR"
    summary: "Vyhledávání slovenských firem — IČO, statutární orgány, sídlo, základní kapitál, historie a výpisy z orsr.sk."
  en:
    displayName: "Slovak Commercial Register"
    summary: "Look up Slovak companies — IČO, statutory bodies, registered office, share capital, history and orsr.sk extracts."
  sk:
    displayName: "Obchodný register SR"
    summary: "Vyhľadávanie slovenských firiem — IČO, štatutárne orgány, sídlo, základné imanie, história a výpisy z orsr.sk."
---

# Slovak Commercial Register (obchodný register SR)

A single tool — **`orsr-cli`** — wraps the RPO open API (Register právnických
osôb, Štatistický úrad SR; free, CC-BY 4.0, refreshed nightly), the orsr.sk
extract-link resolver, and the registeruz.sk financial-statements API.

**IMPORTANT:** The only tool in this skill is `orsr-cli`. Do NOT call `curl` or
any other tool directly. Assume `orsr-cli` is installed and available in `PATH`.

**IMPORTANT:** If `orsr-cli` outputs an `ERROR:` line (e.g. `HTTP 404 …`), stop
and report it to the user. Do not retry blindly.

## Output Format

`orsr-cli` always prints **JSON to stdout** (verbatim RPO response). Parse with
`jq` — fields are nested objects. On failure: `ERROR: …` on stderr, exit code 2.

Successful RPO/RUZ responses are **cached by URL for 24 hours**; pass `--no-cache`
when the user explicitly needs a fresh read.

### jq cookbook — use these field names, do not guess others

Search results (`.results[]`) and `detail` share the same entity shape. The
identifier lives in `identifiers[].value` (NOT `.identifier`), the register in
`sourceRegister` (an object, NOT `sourceRegisters`), and every sub-object is a
history array with `validFrom`/`validTo`:

```bash
# overview of search hits: RPO id, IČO, current name, terminated?
orsr-cli search --name "ESET" | jq '.results[] | {id, ico: .identifiers[-1].value, name: .fullNames[-1].value, terminated: .termination}'

# current statutory bodies (konatelia/predstavenstvo)
orsr-cli detail 937053 | jq '[.statutoryBodies[] | select(.validTo == null) | {name: .personName.formatedName, role: .stakeholderType.value}]'

# current share capital and registration
orsr-cli detail 937053 | jq '{imanie: [.equities[] | select(.validTo == null)], register: .sourceRegister}'

# current registered office
orsr-cli detail 937053 | jq '[.addresses[] | select(.validTo == null)]'
```

## Five commands

```bash
orsr-cli search --ico <ICO> | --name <TEXT> | [filters...]
orsr-cli detail <RPO_ID> [--history] [--units]
orsr-cli links <ICO>
orsr-cli ruz <ICO>
orsr-cli api <PATH>
```

### Search filters

At least one filter is required; filters combine (logical AND). RPO returns at
most 500 hits — narrow the query if the result is truncated.

| Flag | RPO parameter | Notes |
|---|---|---|
| `--ico <ICO>` | `identifier` | 1–8 digits, padded to 8 with leading zeros |
| `--name <TEXT>` | `fullName` | loose match, includes historical names |
| `--municipality <TEXT>` | `addressMunicipality` | seat municipality |
| `--street <TEXT>` | `addressStreet` | seat street |
| `--legal-form <TEXT>` | `legalForm` | legal form text |
| `--legal-status <TEXT>` | `legalStatus` | legal status text |
| `--source-register <TEXT>` | `sourceRegister` | source register text |
| `--main-activity <TEXT>` | `mainActivity` | main economic activity text |
| `--active` | `onlyActive=true` | only active entities |
| `--establishment-after/-before <DATE>` | `establishmentAfter/Before` | YYYY-MM-DD |
| `--termination-after/-before <DATE>` | `terminationAfter/Before` | YYYY-MM-DD |
| `--db-modification-after/-before <DATE>` | `dbModificationDateAfter/Before` | YYYY-MM-DD |

### Workflow

1. **`search --ico`** — one IČO can return multiple records (main entity plus
   organizational units/branches, including defunct ones). Pick the right one:
   prefer the record whose `fullNames` matches the expected company and that has
   no `termination` date.
   **`search --name`** — matching is loose and includes historical names of
   renamed/dissolved companies; filter and re-rank the results yourself; narrow
   with `--municipality`, `--active` etc. when the name alone is ambiguous.
2. **`detail <id>`** — `id` is the RPO internal id from search results, **not**
   the IČO. Returns the complete record with history; see `references/rpo-api.md`.
   Add `--history` (showHistoricalData) and/or `--units` (showOrganizationUnits)
   when the user asks for a full úplný výpis or organizational units.
   Highlights: `statutoryBodies[]` (konatelia with `personName`, `stakeholderType`),
   `stakeholders[]` (spoločníci), `equities[]`/`deposits[]` (základné imanie),
   `sourceRegister` (court + number, e.g. `Sro/3586/B`). Every sub-object carries
   `validFrom`/`validTo` — **filter out entries with `validTo` set when the user
   asks about the current state**; include them for history (úplný výpis).
3. **`links <ico>`** — resolves the official orsr.sk extract URLs. Present
   `aktualnyVypis` (current) and, when the user wants history, `uplnyVypis`.
4. **`ruz <ico>`** — accounting entity from registeruz.sk (financial statements
   ids in `idUctovnychZavierok`). Use when the user asks about financials and
   link `https://www.registeruz.sk/cruz-public/domain/accountingentity/show/{id}`
   (the resolved entity id; `simplesearch?q={ico}` as fallback when no id is known).
5. **`api <path>`** — raw GET escape hatch against the RPO base
   (e.g. `orsr-cli api "search?identifier=31333532"`).

## Chat sources

`orsr-cli links`, `detail` and `ruz` automatically attach the official pages
(orsr.sk výpis, registeruz.sk entity page) to the chat "Sources" panel — no
extra step needed. `search` attaches nothing; run `detail` or `links` on the
chosen entity to get the citation. Still cite the same URLs in prose.

## Output rules

- Match the user's conversation language; keep Slovak legal terms
  (konateľ, spoločník, základné imanie) in Slovak.
- Always include the IČO and the registration (`sourceRegister`: court + number)
  when identifying a company, plus the `aktualnyVypis` link from `orsr-cli links`.
- Clearly distinguish current state from historical entries (`validTo` set).
- Data is refreshed nightly — for same-day changes point the user to orsr.sk.
- RPO is **not** a VAT registry — never derive or fabricate DIČ/IČ DPH from RPO
  data; if a field is absent, say RPO did not return it.
- When citing RPO data, attribute the source: "Zdroj: Register právnických osôb,
  podnikateľov a orgánov verejnej moci, Štatistický úrad SR. Licencia: CC BY 4.0."
- This skill covers Slovak entities only. For Czech companies (ARES, Czech IČO)
  use the `ares` skill instead.

## Reference Files

- **`references/rpo-api.md`** — RPO entity field reference, orsr.sk URL patterns,
  registeruz.sk financial statements API

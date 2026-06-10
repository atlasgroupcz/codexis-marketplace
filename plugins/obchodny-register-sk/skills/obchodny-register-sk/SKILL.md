---
uuid: 499b1cf8-5a26-49ef-8840-d664fdd30582
name: obchodny-register-sk
description: Slovenský obchodný register a register právnických osôb (RPO, Štatistický úrad SR). Use for Slovak company / legal-entity lookups by IČO or name — identification, registered office, statutory bodies (konatelia, predstavenstvo), partners (spoločníci), share capital (základné imanie), legal form, registration court and number, and full historical changes. Triggers on "obchodný register", "ORSR", "RPO", "slovenská firma", "slovenské IČO", "výpis z obchodného registra", "konateľ", "spoločník", "základné imanie", "register právnických osôb", "sídlo firmy na Slovensku". Slovak (SK) jurisdiction only — for Czech companies use the ares skill.
version: 1.0.0
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

Primary data source is the **RPO open API** (Register právnických osôb, podnikateľov
a orgánov verejnej moci) of the Štatistický úrad SR — free, anonymous, JSON,
licence CC-BY 4.0, refreshed nightly. RPO aggregates the obchodný register (ORSR),
živnostenský register and other source registers.

Base URL: `https://api.statistics.sk/rpo/v1`

Use `curl` directly — no API key, no special headers needed.

## Workflow

### 1a. Search by IČO

```bash
curl -s "https://api.statistics.sk/rpo/v1/search?identifier=31333532"
```

Returns `results[]`. **One IČO can return multiple records** (the main entity plus
organizational units/branches, including defunct ones). Pick the right one:
prefer the record whose `fullNames` matches the expected company, without a
`termination` date, with `sourceRegister` = obchodný register when the user asks
about ORSR data.

### 1b. Search by name

```bash
curl -s "https://api.statistics.sk/rpo/v1/search?fullName=ESET"
```

Matching is loose and includes **historical names** (renamed and dissolved
companies match too) and there is no pagination metadata — filter and re-rank
the results yourself before presenting them.

### 2. Full detail

```bash
curl -s "https://api.statistics.sk/rpo/v1/entity/937053"
```

`{id}` is the RPO internal id from the search results — **not** the IČO
(IČO in the path returns 404). The response carries the complete register
content with history; see `references/rpo-api.md` for the field rundown.
Highlights:

- `statutoryBodies[]` — konatelia/predstavenstvo with `personName`,
  `stakeholderType` and address
- `stakeholders[]` — spoločníci/zakladatelia with ownership stakes
- `equities[]` / `deposits[]` — základné imanie and vklady
- `sourceRegister` — registration court and number (e.g. `Sro/3586/B`)
- every sub-object has `validFrom`/`validTo` — **filter out entries with a
  `validTo` in the past when the user asks about the current state**; use them
  when the user asks for history (úplný výpis semantics)

### 3. Deep link to the official orsr.sk extract

The legally citable human-readable výpis lives on `orsr.sk`. Resolve the internal
ids first (response is windows-1250 HTML — only extract the link, do not parse
the page content):

```bash
curl -s -H "User-Agent: Mozilla/5.0" \
  "https://www.orsr.sk/hladaj_ico.asp?ICO=31333532&SID=0" | grep -ao "vypis[^\"']*" | head -2
```

This yields `vypis.asp?ID={id}&SID={court}&P=0`. Present to the user:

- Aktuálny výpis: `https://www.orsr.sk/vypis.asp?ID={id}&SID={court}&P=0`
- Úplný výpis (with history): `https://www.orsr.sk/vypis.asp?ID={id}&SID={court}&P=1`

### 4. Optional: financial statements (účtovné závierky)

Register účtovných závierok has a free JSON API — see the end of
`references/rpo-api.md`. Use when the user asks about financials, and link
`https://www.registeruz.sk/cruz-public/domain/accountingentity/simplesearch?q={ico}`
for browsing.

## Output rules

- Match the user's conversation language; keep Slovak legal terms
  (konateľ, spoločník, základné imanie) in Slovak.
- Always include the IČO and the registration (`sourceRegister`: court + number)
  when identifying a company, plus the orsr.sk deep link.
- Never present raw API URLs (`api.statistics.sk/...`) as user-facing links.
- Clearly distinguish current state from historical entries (`validTo` set).
- Data is refreshed nightly — for same-day changes point the user to orsr.sk.
- This skill covers Slovak entities only. For Czech companies (ARES, czech IČO)
  use the `ares` skill instead.

## Reference Files

- **`references/rpo-api.md`** — RPO entity field reference, orsr.sk URL patterns,
  registeruz.sk financial statements API

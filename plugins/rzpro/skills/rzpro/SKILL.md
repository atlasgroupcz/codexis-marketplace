---
uuid: 9a4c1f72-6b8e-4d03-a5f1-2c7b0e9d6a48
name: rzpro
icon: icon.svg
description: Registr zdravotnických prostředků (RZPRO, KSRZIS). Use for Czech medical device lookups by device name or evidence number — manufacturer, device kind, risk class, intended purpose, notified body, variants/catalogue numbers, and registered persons and their activities. Triggers on "zdravotnický prostředek", "RZPRO", "evidenční číslo", "notifikovaný prostředek", "výrobce zdravotnických prostředků", "registr ZP", "najdi prostředek", "diagnostický prostředek", "IVD".
version: 1.1.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "RZPRO — registr zdravotnických prostředků"
    summary: "Vyhledávání zdravotnických prostředků v registru RZPRO — podle názvu nebo evidenčního čísla, výrobce, varianty, osoby."
  en:
    displayName: "RZPRO — Medical Devices Registry"
    summary: "Look up medical devices in the RZPRO registry — by name or evidence number, manufacturer, variants, persons."
  sk:
    displayName: "RZPRO — register zdravotníckych pomôcok"
    summary: "Vyhľadávanie zdravotníckych pomôcok v registri RZPRO — podľa názvu alebo evidenčného čísla, výrobca, varianty, osoby."
---

# RZPRO — registr zdravotnických prostředků

Structured access to the Czech Register of Medical Devices (**RZPRO**, KSRZIS).
A single tool — **`rzpro-cli`** — wraps the ai-scraper search API. There is no
local database; every command is an API call.

**IMPORTANT:** The only tool in this skill is `rzpro-cli`. Do NOT call `curl` or
any other tool. Assume `rzpro-cli` is installed and available in `PATH`.

**IMPORTANT:** When `rzpro-cli` prints an `error:` line, stop and report it. Do
NOT guess workarounds and do NOT blindly retry. When looking up several items,
run each independently.

## Commands

### search
- Devices: `rzpro-cli search CZRZP [OPTIONS]`
- Persons/companies: `rzpro-cli search CZRZP-OS [OPTIONS]`

See `rzpro-cli search CZRZP --help` / `CZRZP-OS --help` or
`references/search-rzpro.md` for the filter lists. Output is JSON on stdout:
`{ "results": [...], "total": N, "offset": N, "limit": N }`. Process it with `jq`.

### get
`rzpro-cli get <rzpro://…> [--dry-run]`:

| Resource | Returns |
|---|---|
| `rzpro://doc/<DOC_ID>/meta` | full device record — kind, risk class, intended purpose, manufacturer, variants/catalogue numbers, validity |
| `rzpro://osoba/<KEY>` | a registered person/company by IČO or Registrační číslo, with the devices it notified |
| `rzpro://doc/values/<FIELD>` | distinct values + counts — `FIELD` ∈ `druh\|riziko\|stat` |

## Workflow

1. Device by name: `rzpro-cli search CZRZP --query "<text>"` → pick the `docId`
   (e.g. `CZRZP00000019`) → `rzpro-cli get rzpro://doc/<docId>/meta`. Words match
   device name, trade name and manufacturer together.
2. Device by evidence number: `rzpro-cli search CZRZP --evidencni-cislo <n>` →
   take the `docId` → `get …/doc/<docId>/meta`.
3. Which device a catalogue number belongs to: `rzpro-cli search CZRZP
   --katalogove-cislo <n>`.
4. A company (activities + notified devices): `rzpro-cli get rzpro://osoba/<IČO or
   Registrační číslo>`. To search companies by name: `rzpro-cli search CZRZP-OS
   --query "<name>"`.
5. Variants and linked persons of a device live inside the `/meta` record.

## Result fields (device search)

Each item in `results[]`: `docId` (internal id — for `get`, never shown to the
user), `url` (public KSRZIS portal link — the **citable source**), `evidencniCislo`,
`nazev`, `obchodniNazev`, `stav`, `isActive`, `druh`, `miraRizika`, `vyrobce`
(manufacturer-of-record, incl. foreign makers), `platnostDo`.

## Presenting results

Present readably — device name, kind, risk class, manufacturer, validity — and
cite the `url`. Do NOT dump raw JSON; pull only what you need with `jq`. Never
show a raw `docId` (`CZRZP…`), a `CZRZP` / `CZRZP-OS` source code, a `rzpro://`
address, or an env-var name to the user.

## Pagination & truncation

`search` returns `total` but only `limit` results (**default 20, max 100**). When
`total` exceeds what you received there are more — page with `--offset` or raise
`--limit`. **Do NOT draw a global conclusion** from a truncated page. Deep paging
is capped: `offset + limit <= 10000`.

## Notes

- Risk classes are tied to the device kind (general `I`–`III`; active implantable
  `AI`; IVD `IVD*` / `IVDR *`) — "implantable class III" yields 0; use `AI`. See
  `references/search-rzpro.md`.
- `vyrobce` is the manufacturer-of-record and is always present, including foreign
  makers. The linked Czech-registered persons (with notifier/edit rights) are in
  the `/meta` record; that list is empty for foreign manufacturers.

## Reference Files

- **`references/search-rzpro.md`** — full device + person filter lists.

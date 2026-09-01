---
uuid: 399947b3-821e-4b60-8777-931e7e138bce
name: sbirka-listin
icon: icon.svg
description: Sbírka listin veřejných rejstříků ČR (obchodní/spolkový/nadační/… rejstřík, Ministerstvo spravedlnosti). Use to list a subject's filed documents by IČO - účetní závěrky, rozvahy, výkazy zisku a ztráty, výroční zprávy, stanovy, společenské/zakladatelské smlouvy, notářské zápisy - to download one of those documents, and to watch a company for newly filed documents. NOT company identification (name, statutory bodies, VAT) - that is the ares skill. Triggers on "sbírka listin", "účetní závěrka", "výroční zpráva", "rozvaha", "výkaz zisku a ztráty", "společenská smlouva", "zakladatelská listina", "stanovy firmy", "dokumenty firmy", "listiny firmy", "sleduj sbírku listin", "hlídat nové dokumenty firmy". Czech (CZ) jurisdiction only.
version: 1.0.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Sbírka listin ČR"
    summary: "Dokumenty subjektu ze Sbírky listin podle IČO - účetní závěrky, stanovy a další listiny, stažení a hlídání přírůstků."
    examplePrompts:
      - "Vypiš, jaké listiny má firma s IČO 27082440 založené ve sbírce listin."
      - "Stáhni poslední účetní závěrku firmy s IČO 45274649."
      - "Hlídej, jestli firma s IČO 00177041 založí do sbírky listin novou listinu."
  en:
    displayName: "Czech Collection of Deeds"
    summary: "A subject's filed documents by IČO - financial statements, articles and other deeds, with download and new-filing watching."
    examplePrompts:
      - "List the documents company 27082440 has filed in the Collection of Deeds."
      - "Download the latest financial statements of company 45274649."
      - "Watch company 00177041 and tell me when it files a new document."
  sk:
    displayName: "Zbierka listín ČR"
    summary: "Dokumenty subjektu zo Zbierky listín podľa IČO - účtovné závierky, stanovy a ďalšie listiny, so stiahnutím a strážením prírastkov."
    examplePrompts:
      - "Vypíš, aké listiny má firma s IČO 27082440 založené v zbierke listín."
      - "Stiahni poslednú účtovnú závierku firmy s IČO 45274649."
      - "Sleduj, či firma s IČO 00177041 založí do zbierky listín novú listinu."
---

# Sbírka listin - Czech Collection of Deeds

A single tool - **`sbirka-listin-cli`** - reads the Sbírka listin of the Czech
public registers (obchodní, spolkový, nadační, … rejstřík) from the Ministry of
Justice portal, keyed by **IČO**. It lists a subject's filed documents,
downloads one, and watches a subject for new filings.

**IMPORTANT:** The only tool in this skill is `sbirka-listin-cli`. Do NOT call
`curl`, `cdx-cli`, or any other tool to reach the register directly. Assume
`sbirka-listin-cli` is installed and available in `PATH`.

**IMPORTANT:** This skill is **only** the Sbírka listin - the *documents* a
company files. It does NOT return the register entry itself (name, sídlo,
jednatelé/statutární orgán, základní kapitál, plátce DPH). For that use the
**ares** skill. A típical company check uses both: `ares` for who the company is,
`sbirka-listin` for what it has filed.

**IMPORTANT:** Stop only on a genuine `ERROR:` line (e.g. `HTTP 404 …`, bad
input, network) - report it, don't guess workarounds and don't blindly retry.
When looking up several subjects, run each independently; one `ERROR:` doesn't
mean skip the rest.

## Output Format

`sbirka-listin-cli` prints **JSON to stdout**, built from the register pages.
Parse with `jq` - never use `sed`/`grep`. On failure: `ERROR: <message>` on
stderr, exit code 2.

## Commands

```bash
sbirka-listin-cli listiny <ICO>                  # list a subject's filed documents
sbirka-listin-cli dokument <ICO> <CISLO>         # download one document to disk
sbirka-listin-cli sledovani add <ICO>            # start watching a subject
sbirka-listin-cli sledovani list                 # watched subjects, changed ones first
sbirka-listin-cli sledovani show <ICO>           # full stored state of one subject
sbirka-listin-cli sledovani remove <ICO>         # stop watching
sbirka-listin-cli sledovani check [<ICO>]        # check for new filings (all if no ICO)
sbirka-listin-cli sledovani confirm <ICO> [--index N]   # mark changes as read
sbirka-listin-cli notify show                    # current notification settings
sbirka-listin-cli notify set [--email|--no-email] [--inapp|--no-inapp]
sbirka-listin-cli notify recipients add|remove|set|clear [<EMAIL>...]
```

`<ICO>` accepts 1-8 digits; it is left-padded to 8 before the call.
`<CISLO>` is the full číslo listiny - the register's identifier of the document
(`"B 8573/SL95/MSPH"`; quote it, it has a space). Copy it verbatim from the
`cislo` field of `listiny` output - do not invent or abbreviate it.

## `listiny` - the document list

The output is `{ico, nazev, spisovaznacka, sidlo, pocet_listin, listiny[]}`, one
entry per filed document:

```bash
# every filed document: číslo, filing date, type
sbirka-listin-cli listiny 27082440 | jq -r '.listiny[]
  | "\(.cislo) | \(.zalozeno) | \(.typ)"'

# the financial statements for a given year (type carries the year in [brackets])
sbirka-listin-cli listiny 27082440 | jq '.listiny[]
  | select(.typ | contains("účetní závěrka [2024]"))'

# the číslo of the newest document - pass it straight to `dokument`
sbirka-listin-cli listiny 27082440 | jq -r '.listiny[-1].cislo'
```

Key fields per entry: `cislo` (the document's identifier, e.g.
`B 8573/SL95/MSPH`), `typ` (a sentence - the kind plus, for statements, the
`[year]`), `zalozeno` (the date filed into the collection), `stran`. An empty
`listiny` means the subject exists but has filed nothing - not an error.

> **Note:** rozvaha and výkaz zisku a ztráty are *parts of* the účetní závěrka,
> not separate filings - they come inside the one "účetní závěrka" document.

## `dokument` - download one document

```bash
sbirka-listin-cli dokument 27082440 "B 8573/SL91/MSPH"
# → {"ico":"27082440","cislo":"B 8573/SL91/MSPH",
#    "path":"…/dokumenty/27082440/ucetni-zaverka-2023__B-8573-SL91-MSPH.pdf",
#    "url":"https://or.justice.cz/ias/content/download?id=…","reused":false}
```

Takes the IČO and the full číslo listiny (copy the `cislo` from `listiny` output,
in quotes). Saves the file under the app's own directory and returns its `path`.
Already downloaded (`reused:true`) → no second fetch. Most documents are PDF;
older statements of listed companies (ČEZ, banks) are XHTML/ESEF - the extension
follows the actual file.

**To answer a question about a document's *contents*** (e.g. "what was the
profit?", "may this director sign alone?"): download it, then read the file at
the path it prints. A scanned PDF reads too - it goes through OCR.

```bash
sbirka-listin-cli dokument 27082440 "B 8573/SL91/MSPH" | jq -r .path
```

Do this only when the user asks about contents; for "show me the documents" the
`listiny` list and the `url` are enough - cite the `url` and let the user open it.

## Watching for new filings

```bash
sbirka-listin-cli sledovani add 27082440     # today's list becomes the baseline
sbirka-listin-cli sledovani check            # daily automation runs this
sbirka-listin-cli sledovani list             # subjects with unread changes first
sbirka-listin-cli sledovani show 27082440    # one subject's full state + change history
sbirka-listin-cli sledovani confirm 27082440 # mark this subject's changes read
```

`add` records the current list as the baseline, so the first `check` reports only
what arrives afterwards. `check` returns `{results, notifikace}`: each result's
`change.nove_listiny` lists newly filed documents (unread until `confirm`), and
`notifikace` says what was sent. The daily automation "Hlídač sbírky listin"
(07:00) runs `check` and notifies on new filings.

**Notifications.** `check` sends an e-mail and/or in-app (bell) alert on new
filings, per `notify show`. Defaults: in-app on, e-mail off, no extra recipients
(the signed-in user always gets the e-mail; `notify recipients add` adds others).
E-mail and the bell only fire when run through the daemon (the automation), not
from a bare terminal.

**Never add tracking without explicit user intent.** If the user asks to be told
about new filings, offer it: "Mám firmu 27082440 přidat do hlídání sbírky
listin?" and add only on confirmation.

## Decision tree

- "jaké dokumenty / listiny má firma X" → `listiny <ICO>`, present the list + dates
- "dej mi účetní závěrku firmy X za rok R" → `listiny`, filter the type, take its `cislo`
- "kolik firma X vydělala / co je v té závěrce" → `listiny` to find the `cislo`, then
  `dokument <ICO> "<cislo>"`, then read the file
- "kdo je jednatel / sídlo / plátce DPH firmy X" → **not this skill - use `ares`**
- "dej mi vědět, když firma X něco založí" → offer, then `sledovani add <ICO>`

## Chat sources

`listiny` attaches the subject's official register page (`or.justice.cz`) to the
chat "Sources" panel automatically - no extra step. Cite that same URL in prose;
it is the authoritative source of record.

## Output rules

- Identify and cite documents by their full `cislo` (`B 8573/SL95/MSPH`) - the
  register's official "Číslo listiny". A compact list may lead with the type and
  date, but state the `cislo` for any specific document you point to.
- Never fabricate a document the register did not list. If a subject has filed
  nothing, say so.
- Present `zalozeno` (the filing date) with each document - it is what tells the
  user how recent a filing is.
- The register is the source of truth; relay `typ` as written (it carries the
  year and document kind), don't paraphrase away the `[year]`.

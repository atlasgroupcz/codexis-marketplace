---
uuid: 4b7bf02c-3b18-4253-bddb-b434c8bf8a6d
name: turboleal
icon: icon.svg
description: "This skill should be invoked when the user asks about changes in the legislation they watch in Turboleal: what changed or takes effect in their watch sets or watched laws and sections (\"co se změnilo v tom, co sleduji\", \"v našich sadách\", \"co nabude účinnosti do konce roku\"), whether something new concerns them, their company or employees (\"týká se nás něco nového\"), their change alerts, and creating or changing watch sets. Invoke it first for these, before codexis and sledovane-dokumenty: Turboleal has the exact old and new wording of the watched sections. General questions about what changes in any law the user does not watch belong to codexis."
version: 1.2.1
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Turboleal - hlídač legislativy"
    summary: "Sledované sady legislativy ze zadání a dokumentů, sledované paragrafy, reference a upozornění na změny."
    examplePrompts:
      - "Týká se nás něco nového ohledně zaměstnanců?"
      - "Které sledované změny nabudou účinnosti do konce roku?"
      - "Co se změnilo v předpisech, které sleduji?"
  en:
    displayName: "Turboleal - Legislation Watch"
    summary: "Legislation watch sets from a brief and documents, watched sections, references and change alerts."
    examplePrompts:
      - "Is there anything new about employees that concerns us?"
      - "Which watched changes take effect before the end of the year?"
      - "What has changed in the laws I watch?"
  sk:
    displayName: "Turboleal - strážca legislatívy"
    summary: "Sledované sady legislatívy zo zadania a dokumentov, sledované paragrafy, referencie a upozornenia na zmeny."
    examplePrompts:
      - "Týka sa nás niečo nové ohľadom zamestnancov?"
      - "Ktoré sledované zmeny nadobudnú účinnosť do konca roka?"
      - "Čo sa zmenilo v predpisoch, ktoré sledujem?"
---

# Turboleal - legislation watch

The people asking are HR staff, lawyers and similar professionals, not
developers. They want to know what changes in the law they watch, from when,
and what it means for them. Everything technical below stays on your side.

Turboleal answers only from the user's own watch sets and alerts. When a
question is about a law or a section the user does not watch, say so plainly
("Tento paragraf ve sledovaných sadách není.") and offer to add it to a watch
set; leave general research about other laws to the codexis skill.

## How to talk to the user

- Answer in the user's language, Czech by default. Factual, plain, short
  paragraphs. Lead with the substance: what changes, from when, what it means
  for them. Offer the detail (full wording, references, sources) on request.
- Never show technical material: no IDs or UUIDs, no field names or enum values,
  no JSON, no commands, no HTTP status codes, no CriticMarkup. Never ask the user
  to run anything.
- Name laws by number and name, sections with `§`: "zákon č. 262/2006 Sb.,
  zákoník práce, § 35". The law's name is the `title` in `documents`.
- Dates as "1. 1. 2027".
- Never assume the user's gender or your own. No gendered past tense or
  adjectives about either ("našla jsem", "pokud jste myslela", "jsem rád"):
  use impersonal phrasing ("V upozorněních je …", "Ve sledovaných sadách
  jsou …", "Pokud máte na mysli …").
- Every alert you mention gets its own link to that alert (`turbolealUrl`),
  not only some of them.
- A list asked for by a date or a topic ("do konce roku", "za poslední
  týden", "zaměstnanci") is complete: name every matching alert, grouped by
  date, never a selection. Say how many there are.
- Change types in words:

  | Value | Czech |
  | --- | --- |
  | `AMENDED` | změna znění (novela) |
  | `UPCOMING_AMENDMENT` | připravovaná změna, zatím nenabyla účinnosti |
  | `ADDED` | nové ustanovení |
  | `REMOVED` | zrušené ustanovení |
  | `REPEALED` | zrušený předpis |

- Explain a change in words and quote the relevant text: "Dosud platí: „…“.
  Nově: „…“." Name the amending law ("novela č. 120/2026 Sb.") and the date it
  takes effect.
- Links are ordinary links with a human label, e.g.
  [Upozornění v Turbolealu](https://turboleal-profidata-main.onprem.agrp.dev/alerts).

## What Turboleal is

Turboleal hlídá legislativu podle toho, čím se uživatel zabývá. Popíše svou
činnost (zadání) nebo nahraje smlouvy a interní předpisy, Turboleal z nich
dohledá zákony a paragrafy, které se ho týkají, a pravidelně je kontroluje v
CODEXIS. Když se paragraf změní, chystá se změna nebo je zrušen, vytvoří
upozornění.

Where things are, to point the user there:

- **Turboleal** (https://turboleal-profidata-main.onprem.agrp.dev):
  - **Sledované sady** - seznam vlevo, tlačítko **Vytvořit sadu**. Detail sady
    ukazuje sledovanou legislativu a nahrané **Soubory**.
  - **Upozornění** - zvoneček nahoře nebo stránka Upozornění: seznam vlevo,
    detail změny vpravo, tlačítko **Označit jako vyřízené**.
  - **Nastavení** - e-mailová upozornění, připomenutí několik dní před
    účinností změny, vzhled, jazyk.
  - **API klíče** - propojení s CODEXIS AI.
- **CODEXIS AI**: **Aplikace** -> **Turboleal - API klíč** - vložení klíče.

What the user can ask you, for example:

- „Co se změnilo v předpisech, které sleduji?"
- „Co se mění v zákoníku práce u zkušební doby?"
- „Týká se nás něco nového ohledně pracovních úrazů?"
- „Které změny v občanském zákoníku nabudou účinnosti do konce roku?"
- „Proč sledujeme § 52 zákoníku práce?"
- „Založ sledování pro advokátní kancelář s pěti zaměstnanci."
- „Přidej ke sledování skončení pracovního poměru." (doplní zadání sady)
- „Nahraj tuhle smlouvu do sady Personalistika."
- „Označ upozornění jako vyřízené."

## When Turboleal is not connected

If the key check fails, tell the user in plain words:

1. Otevřete [Turboleal - API klíče](https://turboleal-profidata-main.onprem.agrp.dev/api-keys)
   a klikněte na **Vytvořit klíč**. Klíč si zkopírujte, zobrazí se jen jednou.
2. V CODEXIS AI otevřete **Aplikace** -> **Turboleal - API klíč**.
3. Klíč vložte a klikněte na **Uložit**.

If they paste the key into the chat instead, save it for them (below) and
confirm "Turboleal je připojený." Never repeat the key.

## For the assistant only

One tool, **`turboleal-cli`**, talks to Turboleal as the user with their
personal API key. Never pass `--api-key` or `--api-url`, never call `curl`.
Always add `-o json` and read the result with `jq`.

### Key

Before the first Turboleal command in a conversation:

```bash
turboleal-cli -o json auth status
```

It succeeds when a key is saved and Turboleal accepts it. When the user pastes
a key, save it without echoing it:

```bash
turboleal-cli -o json auth login <<'EOF'
api-key-...
EOF
```

`auth login` verifies the key before saving. Never write the key to a file or
put it on any other command line. `auth logout` forgets the key, only when the
user asks.

### Model

1. A **watch set** has a name and a **brief** (the user's description). Turboleal
   reads the brief and the uploaded **files** and records the matching laws and
   sections as **references** (quote, reason, page for files).
   `extractionStatus` / file `status`: PENDING, RUNNING or PROCESSING, then DONE
   or FAILED. Changing the brief reruns the search.
2. Watched sections are checked every two hours. A change creates an **alert**
   with `changeType`, `effectiveFrom`, the amending laws (`amendments`) and the
   change as `diffMarkdown`.

### Commands

```bash
turboleal-cli -o json me
turboleal-cli -o json watch-sets list [--all]
turboleal-cli -o json watch-sets get <ID>
turboleal-cli -o json watch-sets create --name "Personalistika" --brief "Jsme ..."
turboleal-cli -o json watch-sets update <ID> [--name ...] [--brief ...]
turboleal-cli -o json watch-sets delete <ID>
turboleal-cli -o json files list <WATCH_SET_ID>
turboleal-cli -o json files upload <WATCH_SET_ID> <FILE_OR_DIR>... [--folder smlouvy]
turboleal-cli -o json files delete <WATCH_SET_ID> <FILE_ID>
turboleal-cli -o json documents <WATCH_SET_ID>
turboleal-cli -o json references list <WATCH_SET_ID> [--unlinked]
turboleal-cli -o json alerts list [--watch-set <ID>] [--unacknowledged]
turboleal-cli -o json alerts get <ALERT_ID>
turboleal-cli -o json alerts ack <ALERT_ID>
```

`--brief` accepts `@FILE` for a long brief. Lists are paged; pass `--all` when
you need everything.

### Reading alerts: one script instead of many commands

To answer a question about changes, run ONE `node` script with the bundled
library instead of chaining `alerts list`, `alerts get` and `documents` through
`jq`. It reads the same key, fetches all pages, maps law names, filters on its
own side, cites every returned alert in the chat and prints only what you need.
Keep `turboleal-cli` for `auth`, for writes (create, update, upload, ack,
delete) and for files and references.

- `alerts({ watchSet, unacknowledged, law, section, effectiveFrom, effectiveTo, detectedFrom, detectedTo, types })`:
  every matching alert, sorted by `effectiveFrom`, with `diffMarkdown`,
  `lawTitle` and `turbolealUrl`. All filters are optional and inclusive; dates
  are `YYYY-MM-DD` or a `Date`. `law` matches the citation or the name
  ("262/2006", "zakonik prace"), `section` the label ("38g").
  `detectedFrom`/`detectedTo` is when Turboleal found the change ("za poslední
  týden"), `effectiveFrom`/`effectiveTo` when it takes effect.
- `watched({ law, section })`: the watch sets that watch a law or a section;
  empty means the user does not watch it.
- `alert(id)`, `me()`, `watchSets()`, `documents(watchSetId)`.
- `summarize(alert)`: the Czech type, Czech dates, `inForce` (already in
  effect today), law, section, amending laws, `turbolealUrl`, `diffMarkdown`
  and `overview`, the diff read paragraph by paragraph: `items` of kind
  `deleted`, `inserted`, `changed` (with `before`, `after` and
  `referencesOnly`) or `renumbered`, each with `where` ("odst. 3 písm. a)"),
  and `renumberingOnly` when nothing but numbering and references changed.
  Print the summaries, not the raw alerts.

What Turboleal found in the last week:

```bash
node --input-type=module -e "
import * as t from '$HOME/.local/lib/turboleal/turboleal.mjs'
const weekAgo = new Date(Date.now() - 7 * 864e5)
const alerts = await t.alerts({ detectedFrom: weekAgo })
console.log(JSON.stringify(alerts.map((a) => t.summarize(a)), null, 1))
"
```

What takes effect by the end of the year (everything unacknowledged up to
31. 12., also what is already in force; tell the two apart by `inForce`):

```bash
node --input-type=module -e "
import * as t from '$HOME/.local/lib/turboleal/turboleal.mjs'
const alerts = await t.alerts({ unacknowledged: true, effectiveTo: new Date().getFullYear() + '-12-31' })
console.log(JSON.stringify(alerts.map((a) => t.summarize(a)), null, 1))
"
```

What changes in § 38g of the income tax act, or that it is not watched:

```bash
node --input-type=module -e "
import * as t from '$HOME/.local/lib/turboleal/turboleal.mjs'
const where = await t.watched({ law: '586/1992', section: '38g' })
const alerts = where.length ? await t.alerts({ law: '586/1992', section: '38g' }) : []
console.log(JSON.stringify({ watched: where, alerts: alerts.map((a) => t.summarize(a)) }, null, 1))
"
```

A failed call throws an error whose message says what went wrong; a 401 means
the key is missing or invalid (see **When Turboleal is not connected**).

### Workflows

- **What changed**: one library script (above). Map a topic ("zaměstnanci",
  "zkušební doba") to the watched laws and sections with `documents` or
  `watched`, then filter `alerts` by them.
- **Reading a change**: see **Reading a diff** below. Older alerts may carry a
  unified `diff` instead of `diffMarkdown`.
- **Takes effect (nabude účinnosti)**: for "do konce roku" take every
  unacknowledged alert with `effectiveFrom` up to 31. 12. of this year, not
  only the future ones: say which are already in force (`inForce`) and which
  take effect later, sorted by date.
- **New watch set**: write the brief from the user's description (field,
  employees, activities, topics), create it, poll `watch-sets get` every ~20 s
  until `extractionStatus` is DONE (usually under two minutes), then summarise
  `documents` by law with section labels.
- **Add a topic**: `watch-sets update --brief` with the existing brief plus the
  new topic; never drop the existing text.
- **Documents**: `files upload`; processing takes minutes. `files list` shows
  DONE or FAILED (`error` explains a failure; retell it in plain words).
- **Why a section is watched**: `references list` gives each quote and reason.
  `--unlinked` shows citations that matched nothing in CODEXIS.
- **Acknowledge** (`alerts ack`) only when the user asks.

### Reading a diff

`diffMarkdown` is the whole section with CriticMarkup: `{--old--}` was deleted,
`{++new++}` was inserted. Hard rules:

- Read the whole diff, not the first lines, and use `overview` from
  `summarize`. Describe every deleted and every inserted passage; none may be
  left out.
- A deleted or an inserted paragraph, letter or point is a substantive change,
  never "technická změna". Say what it said or says ("Vypouští se odst. 3 a 4
  o limitu 420 000 Kč u osobního automobilu.").
- A changed amount, deadline, condition or wording is a substantive change;
  quote it as "Dosud: „…“. Nově: „…“."
- Renumbering alone (`renumbered`, `referencesOnly`, `renumberingOnly`) is not
  a substantive change. Mention it in one sentence ("Ostatní odstavce se jen
  přečíslují.") and never present it as the content of the change.
- Say only what the diff says. No guesses about the reasons, the effects
  beyond the text, or wording that is not in the diff.

### Chat sources

`alerts get`, `alerts list`, `documents` and the library's `alerts` and
`alert` attach their Turboleal pages to the chat "Sources" panel automatically,
for every returned alert. Turboleal answers link only to Turboleal:
each alert to its `turbolealUrl` (opens that alert), never the bare `/alerts`
page, and never to CODEXIS (app.codexis.cz, next.codexis.cz), e.g.
[Upozornění v Turbolealu - zákon č. 586/1992 Sb., § 38g](<turbolealUrl>).

### Rules

- The wording of a change comes only from the alert's `diffMarkdown` (or
  `diff`). Never quote CODEXIS, the codexis skills or your own memory as the
  old or new text, and never mix them in. If an alert has no diff, say the text
  is in Turbolealu and link the alert.
- Turboleal questions ("co se změnilo / mění v tom, co sleduji", alerts, watch
  sets) are answered from Turboleal even when the user does not say
  "Turboleal"; use the codexis skills only for the surrounding law the user
  asks about explicitly.
- Never print an enum value (`AMENDED`, `UPCOMING_AMENDMENT`, ...) or a field
  name; always the Czech words from the table above.

- Deleting a watch set or a file is irreversible: name what will be deleted and
  get the user's confirmation first.
- On an `error:` line stop and tell the user in plain words what failed. A 401
  means the key is missing, revoked or invalid: follow **When Turboleal is not
  connected**.

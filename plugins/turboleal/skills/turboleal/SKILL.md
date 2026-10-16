---
uuid: 4b7bf02c-3b18-4253-bddb-b434c8bf8a6d
name: turboleal
icon: icon.svg
description: "This skill should be invoked when the user asks about changes in the legislation they watch in Turboleal: what changed or takes effect in their watch sets or watched laws and sections (\"co se změnilo v tom, co sleduji\", \"v našich sadách\", \"co nabude účinnosti do konce roku\"), what changes in a specific law or section (\"co se mění v § 38g zákona o daních z příjmů\", \"jaké změny čekají zákoník práce\", \"mění se § 52 zákoníku práce?\"), whether something new concerns them, their company or employees (\"týká se nás něco nového\"), their change alerts, and creating or changing watch sets. Invoke it first for these, before codexis and sledovane-dokumenty: it checks whether the law or section is watched and then has the exact old and new wording of the change. Only when Turboleal says the law is not watched do general questions about it belong to codexis."
version: 1.3.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Turboleal - hlídač legislativy"
    summary: "Sledované sady legislativy ze zadání a dokumentů, sledované paragrafy, reference a upozornění na změny."
    examplePrompts:
      - "Týká se nás něco nového ohledně zaměstnanců?"
      - "Které sledované změny nabudou účinnosti do konce roku?"
      - "Co se mění v § 38g zákona o daních z příjmů?"
  en:
    displayName: "Turboleal - Legislation Watch"
    summary: "Legislation watch sets from a brief and documents, watched sections, references and change alerts."
    examplePrompts:
      - "Is there anything new about employees that concerns us?"
      - "Which watched changes take effect before the end of the year?"
      - "What changes in section 38g of the Income Tax Act?"
  sk:
    displayName: "Turboleal - strážca legislatívy"
    summary: "Sledované sady legislatívy zo zadania a dokumentov, sledované paragrafy, referencie a upozornenia na zmeny."
    examplePrompts:
      - "Týka sa nás niečo nové ohľadom zamestnancov?"
      - "Ktoré sledované zmeny nadobudnú účinnosť do konca roka?"
      - "Čo sa mení v § 38g zákona o daniach z príjmov?"
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

**Never assume anyone's gender, neither the user's nor your own.** This holds
for the answer and for every short message between tool calls. No gendered
past tense, conditional or adjective in the first or second person:

| Never | Instead |
| --- | --- |
| „Našel jsem“, „Našla jsem“ | „V upozorněních je …“, „Nalezeno 15 upozornění.“ |
| „Podíval jsem se“, „Zkontrolovala jsem“ | „Ve sledovaných sadách je …“, „Kontrola sledovaných sad:“ |
| „abych mohl“, „abych mohla“ | „Pro úplný přehled …“, „K tomu je potřeba …“ |
| „pokud jste myslel/a“, „jste měla“ | „Pokud máte na mysli …“, „Máte k dispozici …“ |
| „abyste dostal/a“, „byl/a byste rád/a“ | „Upozornění přijde e-mailem.“, „Chcete …?“ |
| „jsem rád“, „jsem si jistá“ | „Rádi pomůžeme.“ or leave it out |

- Answer in the user's language, Czech by default. Factual, plain, short
  paragraphs. Lead with the substance: what changes, from when, what it means
  for them. Offer the detail (full wording, references, sources) on request.
- Never show technical material: no IDs or UUIDs, no internal document or
  version IDs (`CR6904_2026_01_01`, `CR26785`), no field names or enum values,
  no JSON, no commands, no HTTP status codes, no CriticMarkup. Never ask the
  user to run anything. A law is its number and name, a change is its link.
- Name laws by number and name, sections with `§`: "zákon č. 262/2006 Sb.,
  zákoník práce, § 35". The law's name is the `title` in `documents`.
- Dates as "1. 1. 2027".
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
Always add `-o json`. Use it for the key, for writes (create, update, upload,
ack, delete) and for files, documents and references. Read alerts with the
library in a `script` program (below), never with `alerts list` chains,
`jq` or `node -e`.

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
turboleal-cli -o json alerts ack <ALERT_ID>
```

`--brief` accepts `@FILE` for a long brief. Lists are paged; pass `--all` when
you need everything.

### Reading alerts: one `script` program per question

The plugin installs a library at `~/.local/lib/turboleal/turboleal.cjs`. Use it
ONLY from the `script` tool, with ONE program per question. It reads the
same key, fetches all pages, maps law names, filters on its own side and
queues every returned alert for the chat "Sources" panel.

A `script` program may import only `node:` builtins, written in single
quotes, so load the library with `createRequire`. Every program starts and
ends like this:

    import { createRequire } from 'node:module'
    import { homedir } from 'node:os'

    const t = createRequire(import.meta.url)(homedir() + '/.local/lib/turboleal/turboleal.cjs')

    export default async function () {
      // ... t.watched / t.alerts ...
      await t.flushSources()
      return result
    }

`await t.flushSources()` before `return` is mandatory: without it the
Turboleal links never reach the Sources panel.

The library:

- `t.watched({ law, section })`: the watch sets that watch a law or a section
  (a whole watched law matches every section); `[]` means not watched.
- `t.alerts({ watchSet, unacknowledged, law, section, effectiveFrom, effectiveTo, detectedFrom, detectedTo, types, limit })`:
  matching alerts sorted by `effectiveFrom`. All filters optional and
  inclusive; dates `YYYY-MM-DD`. `law` matches the citation or the name,
  ignoring case and diacritics ("262/2006", "zakonik prace"); `section`
  matches exactly ("38g" is § 38g, never § 38ga). `detectedFrom/detectedTo`
  is when Turboleal found the change ("za poslední týden"),
  `effectiveFrom/effectiveTo` when it takes effect. `limit` caps the count.
- `t.summarize(alert)`: the compact item to return: Czech `type`, `law`,
  `citation`, `section`, `effectiveFrom`, `inForce` (already in effect today),
  `detectedAt`, `amendments`, `acknowledged`, `turbolealUrl`, `id` (only for
  `alerts ack`) and `overview`: up to six `changes` (`deleted` or `inserted`
  with `text`, `changed` with `before` and `after`, each with `where` such as
  "odst. 3 písm. a)", texts cut to about 200 characters), `more` (changes not
  listed), `renumbered` (renumbering and reference-only edits) and
  `renumberingOnly`.
- `t.detail(alert)`: the summary plus `fullOverview` (every paragraph, same
  item kinds plus `renumbered` and `referencesOnly`) and `diffMarkdown` (an
  older alert carries a unified diff there instead). Only for the few alerts you explain word for word.
- `t.alert(id)`, `t.me()`, `t.watchSets()`, `t.documents(watchSetId)`.

Rules:

- Return compact results only: `summarize` for lists, `detail` only for one
  or a few chosen alerts, never raw alerts or whole documents. First the
  compact list, then, if the user wants the exact wording, a second program
  with `detail` for the chosen ones.
- A question about a specific law or section ("Co se mění v § 38g zákona o
  daních z příjmů?") starts with `t.watched`. Watched: answer from its alerts
  with links. Not watched: say "Tento paragraf ve sledovaných sadách není."
  and offer to add it to a watch set; only then may codexis research it.
- Several laws or sections go into the same program under `Promise.all`.

#### 1. A specific section: watched or not, and what changes

    import { createRequire } from 'node:module'
    import { homedir } from 'node:os'

    const t = createRequire(import.meta.url)(homedir() + '/.local/lib/turboleal/turboleal.cjs')

    export default async function () {
      const PROVISIONS = [['586/1992', '38g'], ['262/2006', '52']]
      const answers = await Promise.all(PROVISIONS.map(async ([law, section]) => {
        const watched = await t.watched({ law, section })
        const alerts = watched.length ? await t.alerts({ law, section }) : []
        return {
          law, section,
          watchedIn: watched.map((w) => w.watchSetName),
          alerts: alerts.map((a) => t.summarize(a))
        }
      }))
      await t.flushSources()
      return answers
    }

#### 2. What takes effect by the end of the year, or was found last week

    import { createRequire } from 'node:module'
    import { homedir } from 'node:os'

    const t = createRequire(import.meta.url)(homedir() + '/.local/lib/turboleal/turboleal.cjs')

    export default async function () {
      const day = (offset) => new Date(Date.now() + offset * 864e5).toISOString().slice(0, 10)
      const [byYearEnd, lastWeek] = await Promise.all([
        t.alerts({ unacknowledged: true, effectiveTo: new Date().getFullYear() + '-12-31' }),
        t.alerts({ detectedFrom: day(-7) })
      ])
      await t.flushSources()
      return {
        byYearEnd: byYearEnd.map((a) => t.summarize(a)),
        lastWeek: lastWeek.map((a) => t.summarize(a))
      }
    }

"Do konce roku" includes what is already in force; tell the two apart by
`inForce`.

#### 3. The exact wording of chosen alerts

    import { createRequire } from 'node:module'
    import { homedir } from 'node:os'

    const t = createRequire(import.meta.url)(homedir() + '/.local/lib/turboleal/turboleal.cjs')

    export default async function () {
      const IDS = ['<id from the summary>', '<id from the summary>']
      const alerts = await Promise.all(IDS.map((id) => t.alert(id)))
      await t.flushSources()
      return alerts.map((a) => t.detail(a))
    }

A failed call throws an error whose message says what went wrong; a 401
means the key is missing or invalid (see **When Turboleal is not connected**).

### Workflows

- **What changed**: one `script` program (above). Map a topic
  ("zaměstnanci", "zkušební doba") to the watched laws and sections with
  `t.documents` or `t.watched`, then filter `t.alerts` by them in the same
  program.
- **Reading a change**: see **Reading a diff** below.
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

- Read every change in `overview`; when `more` is above zero or you quote the
  wording, read `fullOverview` and `diffMarkdown` from `t.detail`. Describe
  every deleted and every inserted passage; none may be left out.
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

`t.alerts` and `t.alert` queue the Turboleal page of every returned alert (up
to 50) and `await t.flushSources()` sends them to the chat "Sources" panel.
Turboleal answers link only to Turboleal: each alert to its `turbolealUrl`
(opens that alert), never the bare `/alerts` page, and never to CODEXIS
(app.codexis.cz, next.codexis.cz), e.g.
[Upozornění v Turbolealu - zákon č. 586/1992 Sb., § 38g](<turbolealUrl>).

### Rules

- The wording of a change comes only from the alert's `overview` and
  `t.detail`. Never quote CODEXIS, the codexis skills or your own memory as the
  old or new text, and never mix them in. If `t.detail` has no
  `diffMarkdown`, say the text is in Turbolealu and link the alert.
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

---
uuid: 4b7bf02c-3b18-4253-bddb-b434c8bf8a6d
name: turboleal
icon: icon.svg
description: "This skill should be invoked whenever the user asks what changes, changed or takes effect in Czech laws or sections (\"co se mění v § …\", \"co se změnilo\", \"nabude účinnosti\", \"do konce roku\") or whether something new concerns them, their company or employees (\"týká se nás něco nového\"), and for watch sets, watched legislation and change alerts. Invoke it first, before codexis and sledovane-dokumenty: Turboleal has the user's watched sections with the exact old and new wording; use codexis afterwards only for extra context."
version: 1.1.5
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Turboleal - hlídač legislativy"
    summary: "Sledované sady legislativy ze zadání a dokumentů, sledované paragrafy, reference a upozornění na změny."
    examplePrompts:
      - "Co se mění v zákoníku práce u zkušební doby?"
      - "Které změny v občanském zákoníku nabudou účinnosti do konce roku?"
      - "Co se změnilo v předpisech, které sleduji?"
  en:
    displayName: "Turboleal - Legislation Watch"
    summary: "Legislation watch sets from a brief and documents, watched sections, references and change alerts."
    examplePrompts:
      - "What is changing in the Labour Code about probation periods?"
      - "Which Civil Code changes take effect before the end of the year?"
      - "What has changed in the laws I watch?"
  sk:
    displayName: "Turboleal - strážca legislatívy"
    summary: "Sledované sady legislatívy zo zadania a dokumentov, sledované paragrafy, referencie a upozornenia na zmeny."
    examplePrompts:
      - "Čo sa mení v zákonníku práce pri skúšobnej dobe?"
      - "Ktoré zmeny v občianskom zákonníku nadobudnú účinnosť do konca roka?"
      - "Čo sa zmenilo v predpisoch, ktoré sledujem?"
---

# Turboleal - legislation watch

The people asking are HR staff, lawyers and similar professionals, not
developers. They want to know what changes in the law, from when, and what it
means for them. Everything technical below stays on your side.

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

### Workflows

- **What changed**: `alerts list --unacknowledged`, then `alerts get` for each
  relevant alert. Filter by law, section label and `effectiveFrom` yourself to
  answer "zkušební doba" or "do konce roku"; use `documents` to map the topic to
  sections and to get the law's name.
- **Reading a change**: `diffMarkdown` is the section text with CriticMarkup:
  `{--old--}` was deleted, `{++new++}` was inserted. Turn it into "dosud / nově"
  with quotes. Older alerts may carry a unified `diff` instead.
- **Takes effect (nabude účinnosti)**: an alert takes effect in the future when
  `effectiveFrom` is after today (usually `UPCOMING_AMENDMENT`). For "do konce
  roku" keep alerts with `effectiveFrom` up to 31. 12. of this year and sort by
  date.
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

### Chat sources

`alerts get`, `alerts list` and `documents` attach their sources to the chat
"Sources" panel automatically: the CODEXIS text of each changed or watched law
or section, and the alert in Turbolealu. Nothing to do on your side. Cite the
same links in your answer with human labels: link each alert to its
`turbolealUrl` (opens that alert in Turbolealu, never the bare `/alerts` page)
and each law or section to its `codexisUrl`, e.g.
[zákon č. 586/1992 Sb., o daních z příjmů, § 38g](https://app.codexis.cz/doc/...)
from `codexisUrl`.

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

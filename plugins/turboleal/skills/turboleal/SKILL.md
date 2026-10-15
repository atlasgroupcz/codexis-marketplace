---
uuid: 4b7bf02c-3b18-4253-bddb-b434c8bf8a6d
name: turboleal
icon: icon.svg
description: Turboleal legislation watch for Czech law. Use when the user wants to watch legislation relevant to their business, contracts or internal rules, see which laws and sections a watch set tracks and why, upload documents to a watch set, or read and acknowledge alerts about amended, upcoming, removed or repealed sections. Triggers on "turboleal", "hlídač legislativy", "sledovaná sada", "hlídej legislativu", "upozornění na změnu zákona", "co se změnilo v zákonech, které sleduji".
version: 1.0.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Turboleal - hlídač legislativy"
    summary: "Sledované sady legislativy ze zadání a dokumentů, sledované paragrafy, reference a upozornění na změny."
    examplePrompts:
      - "Založ v Turbolealu sledovanou sadu pro pekárnu se dvěma zaměstnanci a noční prací."
      - "Které paragrafy hlídá moje sada Pekárna a proč?"
      - "Ukaž nevyřízená upozornění na změny legislativy a co se v paragrafech změnilo."
  en:
    displayName: "Turboleal - Legislation Watch"
    summary: "Legislation watch sets from a brief and documents, watched sections, references and change alerts."
    examplePrompts:
      - "Create a Turboleal watch set for a small bakery with two employees working nights."
      - "Which sections does my watch set Pekárna watch and why?"
      - "Show unacknowledged legislation alerts and what changed in the sections."
  sk:
    displayName: "Turboleal - strážca legislatívy"
    summary: "Sledované sady legislatívy zo zadania a dokumentov, sledované paragrafy, referencie a upozornenia na zmeny."
    examplePrompts:
      - "Založ v Turbolealu sledovanú sadu pre pekáreň s dvoma zamestnancami a nočnou prácou."
      - "Ktoré paragrafy stráži moja sada Pekárna a prečo?"
      - "Ukáž nevybavené upozornenia na zmeny legislatívy a čo sa v paragrafoch zmenilo."
---

# Turboleal - legislation watch

One tool - **`turboleal-cli`** - talks to the Turboleal API as the signed-in user.
The runtime provides the API URL and the user's token, so never pass `--token`
or `--api-url` yourself. Do not call `curl` or other tools against Turboleal.

Always add `-o json` and read the result with `jq`.

## How Turboleal works

1. A **watch set** has a name and a **brief**: what the user does, in their words.
2. Turboleal's AI reads the brief and the uploaded **files**, finds the Czech laws
   and sections in CODEXIS that apply, and records them as **references** (quote
   plus reason, page for files). `extractionStatus` / file `status` go
   PENDING, RUNNING or PROCESSING, then DONE or FAILED.
3. Every two hours it checks the watched sections for new versions and creates
   **alerts**: AMENDED, UPCOMING_AMENDMENT, REMOVED or REPEALED, with a diff.

## Commands

```bash
turboleal-cli -o json me
turboleal-cli -o json watch-sets list [--all]
turboleal-cli -o json watch-sets get <ID>
turboleal-cli -o json watch-sets create --name "Pekárna" --brief "Jsem pekař ..."
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

`--brief` and `update --brief` accept `@FILE` to read a long brief from a file.
Lists are paged; pass `--all` when you need everything.

## Workflows

- **New watch set**: write the brief from the user's own description (business,
  employees, activities, regulated topics), create it, then poll
  `watch-sets get` every ~20 s until `extractionStatus` is DONE (usually under
  two minutes) and summarise `documents` by law with section labels.
- **Documents**: upload with `files upload`; processing takes minutes. Check
  `files list` for DONE or FAILED (`error` explains a failure).
- **Why is a section watched**: `documents` lists the sections; `references list`
  gives each quote and reason. `--unlinked` shows citations that matched nothing
  in CODEXIS.
- **Changes**: `alerts list --unacknowledged`, then `alerts get` for the diff and
  amendments. Acknowledge only when the user asks.

## Rules

- Deleting a watch set or a file is irreversible: confirm with the user first and
  name what will be deleted.
- Stop on an `ERROR:` line and report it. HTTP 401 means the user's session token
  is missing or expired.
- Answer in the user's language and cite laws as in the output (`262/2006 Sb.`,
  `§ 52`).

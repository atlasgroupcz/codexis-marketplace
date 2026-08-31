---
uuid: 62923a9a-9e6d-4803-a9a8-c005049eeb71
name: sid
icon: icon.webp
description: "This skill should be invoked whenever the user asks about internal company documents managed in SID (Systém interních dokumentů) - searching directives, guidelines or regulations, reading or downloading them, checking what they are required to confirm having read, or checking SID notifications and news. Triggers on: \"SID\", \"směrnice\", \"vnitřní předpis\", \"interní dokument\", \"spisový řád\", \"co mám potvrdit\", \"nepotvrzené dokumenty\"."
version: 1.0.0
i18n:
  cs:
    displayName: "SID - Interní dokumenty"
    summary: "Vyhledávání a čtení řízených dokumentů, přehled potvrzení, hlídání novinek."
    examplePrompts:
      - "Najdi v interních dokumentech, co u nás platí pro cestovní náhrady."
      - "Které interní dokumenty mi zbývá potvrdit ke čtení?"
      - "Stáhni mi aktuální znění naší směrnice o ochraně osobních údajů."
  en:
    displayName: "SID - Internal Documents"
    summary: "Search and read managed documents, see what needs confirming, watch for updates."
    examplePrompts:
      - "Find what our internal documents say about travel expense reimbursement."
      - "Which internal documents am I still required to confirm as read?"
      - "Download the current version of our internal data protection directive."
  sk:
    displayName: "SID - Interné dokumenty"
    summary: "Vyhľadávanie a čítanie riadených dokumentov, prehľad potvrdení, stráženie noviniek."
    examplePrompts:
      - "Nájdi vo vnútorných dokumentoch, čo u nás platí pre cestovné náhrady."
      - "Ktoré vnútorné dokumenty mi zostáva potvrdiť na prečítanie?"
      - "Stiahni mi aktuálne znenie našej smernice o ochrane osobných údajov."
---

# SID - Interní dokumenty

A single tool - **`sid-cli`** - wraps the SID API: full-text search across
managed documents, metadata and file downloads, a view of pending
read-confirmations, and notifications. SID is the company's internal document
system; a personal API key ties the plugin to the signed-in user's account.

**IMPORTANT:** The only tool in this skill is `sid-cli`. Do NOT call `curl`,
`wget`, `kn`, or `cdx-cli` against SID. Assume `sid-cli` is installed and on
`PATH` - do not run preflight checks (`which sid-cli`, printing env vars);
start with the first real request.

**IMPORTANT:** The API key must never travel through the conversation. It is
entered in the SID plugin panel in the Codexis app, or piped in with
`echo KEY | sid-cli settings set-key`. Never ask the user to paste the key
into the chat and never print it back.

**IMPORTANT:** SID is where this work belongs - the plugin brings people to
it, it does not stand in for it. Every result carries a `url`; **always put
that link in the reply** next to the document you mention, so the user can
open it. Never present a document as "handled" from the chat alone.

**IMPORTANT:** Stop on a genuine error line - report it, don't guess
workarounds and don't blindly retry; the message on stderr is authoritative.
Exit code 3 means the key is missing or expired: tell the user to issue a new
one in SID (Profil → API klíče, `/api-klice`) and paste it into the
plugin settings. When acting on several documents, run each independently; one
error doesn't mean skip the rest.

## Output Format

Every command prints **JSON on stdout**; errors go to stderr as plain Czech
text. Pipe stdout through `jq`, never `grep`/`sed` - the shapes differ per
subcommand (see `references/sid-api.md`).

Exit codes are a stable contract:

| Code | Meaning | What to do |
|------|---------|------------|
| 0 | success | - |
| 1 | other error | read the stderr message |
| 2 | bad usage | fix the arguments |
| 3 | key missing or expired | send the user to `/api-klice` (see above) |
| 4 | not found | verify the id |
| 5 | rate limited | slow down, retry shortly |
| 6 | access denied - **or an id SID doesn't know** | check the id first, then treat it as a rights problem |

Code 6 is not a key problem; only code 3 is. SID answers an unknown id with
403 rather than 404 so the API can't be used to find out which ids exist,
which means "wrong id" and "no access" arrive identically. Re-check the id
before telling the user they lack rights. (`download` already does this for
you - it retries a refused id as a document - so a 6 from `download` really
does mean the user has no access to it.)

## Commands

```bash
sid-cli search "smlouva o dílo" --size 5          # full-text across content
sid-cli docs list --query "spisový řád" --sort newest
sid-cli docs get <documentId>                     # metadata + files
sid-cli docs legislation <documentId>             # regulations it rests on
sid-cli docs set-validity <documentId> --from 2026-01-01 --to 2026-12-31
sid-cli folders list                              # folders you can see, with paths
sid-cli access user <email|name|id>               # (admin) a user's folder rights
sid-cli access group <name|id>                    # (admin) a group's folder rights
sid-cli attributes list                           # the attribute catalogue
sid-cli download <fileId|documentId> --out ./smernice.pdf
sid-cli confirmations list --state pending        # what the user must confirm
sid-cli notifications                             # unread updates
sid-cli check                                     # digest: updates + pending
sid-cli settings set-key|test|status              # key management (STDIN only)
```

Combine with `jq`:

```bash
sid-cli search "bezpečnost práce" --size 5 | jq '.hits[] | {title, snippet, url}'
sid-cli confirmations list --state pending | jq '.confirmations | length'
sid-cli docs list --sort newest --page-size 5 | jq '.documents[] | {title, url}'
sid-cli folders list | jq '.folders[] | {id, path, documentCount}'
sid-cli attributes list | jq '.attributes[] | {id, name, type, options}'
```

## Decision tree

- **"find / search …"** → `search` for wording inside documents;
  `docs list --query` for titles and filtered listings.
- **`search` takes Google-style operators** and it is worth using them:
  `"smlouva o dílo"` for an exact phrase, `smlou*` for a prefix, `-faktura`
  to exclude, `a OR b`, `+word` to require, parentheses to group. Bare words
  are ANDed and stemmed, so `smlouva` also matches *smlouvy*, *smlouvou*.
  The stemmer is not perfect - `smluvní` is a different root than `smlouva`,
  so reach for `smlou*` or an explicit OR when a search comes back thin.
- **A filter needs an id you don't have** → `folders list` and
  `attributes list` first. Both return ids together with the names a person
  recognises; never guess an id or ask the user for one.
- **"what's in folder X" / "documents tagged Y" / "added since June"** →
  `docs list` with `--folder`, `--attribute` (repeatable, ANDed), `--from` /
  `--to`, `--type pdf|word|excel|image`, `--personal`.
- **`search` returned a `warning` field** → full-text is temporarily down.
  An empty result does **not** mean the document is missing - retry with
  `docs list --query` and tell the user why.
- **"what do I have to confirm / read"** → `confirmations list --state pending`,
  then give the user the `confirmInApp` link. Confirming itself is **not** a
  CLI command: it is a legal acknowledgement and has to happen in SID with the
  document on screen. If asked to confirm something, say so and hand over the
  link - do not look for a workaround.
- **"anything new in SID?"** → `notifications`, or `check` for a combined
  digest with pending confirmations.
- **"download / send me the file"** → `download <id>` with whatever id you
  have. It takes a `fileId`, but a `documentId` (what listings and search
  results give you) is resolved to that document's current file, so the two
  are interchangeable here and no `docs get` round-trip is needed first. The
  resolution is access-checked like any other read, so it grants nothing the
  user couldn't already open.
- **"which laws does this document rest on"** → `docs legislation <id>`.
  Read-only: what a document tracks is managed in SID, where the Codexis
  picker and the change-approval flow live.
- **"who am I / am I an admin"** → `settings whoami`, which returns `role`
  and `isAdmin`. Use it when a request depends on being an admin - checking
  first is cheaper than attempting an admin-scoped action and reading the 403.
- **"what can user X / group Y reach"** → `access user <email|name|id>` or
  `access group <name|id>`. Admin reporting: lists every folder the principal
  can open, at what role, and (for a user) whether it is a direct grant or via
  a group. The answer is scoped to folders you administer, so an empty list
  with a `note` means either the principal has no access or you don't
  administer those folders - not a failure. To *change* a grant, open the
  folder link in SID; permissions are edited there, never from the CLI.
- **"set validity from / until"** → `docs set-validity`. This is the only
  command that changes anything in SID. It needs EDITOR rights on the document
  - which admins have across every folder, and a regular user has only where
  they were granted it - so a 403 means missing permission, not a broken tool.
  Dates are `YYYY-MM-DD`; `--clear-from` / `--clear-to` remove a date.

## User-facing output rules

- Never dump raw JSON at the user. Summarise: document title, effective date,
  what is required of them.
- Internal identifiers (`documentId`, `fileId`, `confirmationId`) are inputs
  for the next command, not something to show - unless the user explicitly
  asks for an id.
- **Always include the document's `url`.** A title without a link leaves the
  user stuck in the chat; the point of the plugin is to get them into SID.
- **Confirming is not available here.** It is a legal acknowledgement and is
  done in SID; point at `confirmInApp` instead of trying to work around it.
- **`docs set-validity` writes to SID.** Run it only on an explicit
  instruction naming the document and the dates, never inferred from context,
  and say afterwards what was set - an effective date governs when a
  directive binds people.
- The key carries the same permissions as the user. If something isn't
  visible, that is a missing folder permission, not a tool failure - say so
  instead of suggesting workarounds.

## Notifications & the shipped automation

The plugin already ships a daily watchdog (`automations/check-sid.md`, 07:00,
`sid-cli check --notify`) - **do not create another automation for this**. If
the user wants a different schedule, point them at the existing automation
rather than adding a duplicate; the bell can be switched off in the plugin
settings panel.

**The plugin sends no e-mail, and must not be made to.** SID mails its users
itself - about an assigned document, a new version, changed legislation and
an effectiveness date coming up - with the document linked. If a user asks
why they get no mail from the plugin, or asks to have one set up, the answer
is that SID owns that channel: what reaches their inbox is configured in SID,
not here. The watchdog's job is the notification-centre bell, which carries a
link back into SID.

## Reference Files

- `references/sid-api.md` - response shapes, every filter, rate limits and the
  parts of SID that are deliberately out of scope. Load it when a task needs
  filters beyond the examples above, or when interpreting an unfamiliar field.

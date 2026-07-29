---
uuid: 1c427cf8-4d37-4304-b2e1-fbabf245dae4
name: sledovane-dokumenty
icon: icon.svg
description: Track changes in Czech legislation via CODEXIS. Use when user wants to monitor laws, track amendments, or check for legislative changes.
version: 1.1.1
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "Sledované dokumenty"
    summary: "Sledování změn v české legislativě přes CODEXIS - upozornění na novelizace a srovnání verzí."
    examplePrompts:
      - "Sleduj zákoník práce a dej vědět, až vyjde jeho novela."
      - "U sledovaného občanského zákoníku mě zajímá hlavně promlčení - ulož si to jako poznámku."
      - "Hlídej složku s mými smlouvami a upozorni mě, když se změní zákon, na který odkazují."
  en:
    displayName: "Document Tracker"
    summary: "Monitor changes in Czech legislation via CODEXIS - amendment alerts and version diffs."
    examplePrompts:
      - "Track the Czech Labour Code and tell me when an amendment is published."
      - "For the tracked Civil Code I mainly care about limitation periods - save that as a note."
      - "Watch the folder with my contracts and alert me when a law they reference changes."
  sk:
    displayName: "Sledované dokumenty"
    summary: "Sledovanie zmien v českej legislatíve cez CODEXIS - upozornenia na novelizácie a porovnanie verzií."
    examplePrompts:
      - "Sleduj český zákonník práce a daj vedieť, keď vyjde jeho novela."
      - "Pri sledovanom občianskom zákonníku ma zaujíma hlavne premlčanie - ulož si to ako poznámku."
      - "Sleduj priečinok s mojimi zmluvami a upozorni ma, keď sa zmení zákon, na ktorý odkazujú."
---

# Sledované dokumenty - Document Tracking

Track changes in Czech legislation by monitoring CODEXIS documents for new versions and computing text diffs.

**IMPORTANT:** For document data use only `cdx-sledovane-dokumenty` - do NOT call `cdx-cli`, `curl`, or any other data tool directly. `cdxctl` is the platform's automation and notification tool: use it to set up recurring monitoring or send an on-demand status e-mail, never to fetch document data.

**IMPORTANT:** Assume `cdx-cli`, `cdxctl`, and `cdx-sledovane-dokumenty` are already installed, configured, available in `PATH`, and invokable. Do not run setup or preflight checks.

**IMPORTANT:** Stop only on a genuine `ERROR:` line (bad input, auth/key rejected, network, not-found) - report it, don't guess workarounds and don't blindly retry. Adding an already-tracked document or removing one that isn't tracked is a safe no-op (prints `OK:`), so re-running `add`/`remove` is fine. When acting on several documents, run each independently; one `ERROR:` doesn't mean skip the rest.

## Output Format

`cdx-sledovane-dokumenty` has three kinds of output. Pick the right way to consume each - mixing them up (e.g. `grep` on JSON, or `jq` on plaintext) will fail.

- **`list`, `check`, `group list`, `note list`, `related list`, `related types`** - human-readable plain text. Read it directly. **Do not parse with `sed` or `grep`** - the layout is for humans, not machines. Summarize what you see for the user.

- **`add`, `remove`, `confirm`, `group add`, `group remove`, `group delete`, `note add`, `note remove`, `related add`, `related remove`** - a single `OK: ...` line on success or `ERROR: ...` on failure. No parsing needed; the whole output is the status.

- **`show`** - structured JSON (full state including diffs and related baselines). Read it directly, or filter with `jq` for a specific field. Never with `sed`/`grep`.

  ```bash
  cdx-sledovane-dokumenty show CR13986 | jq '.changes[-1]'
  cdx-sledovane-dokumenty show CR13986 | jq '.notes'
  ```

## Commands

```bash
# Add a document to tracking (verifies it exists, saves baseline version)
cdx-sledovane-dokumenty add CR13986

# Add with specific paragraphs to track
cdx-sledovane-dokumenty add CR26785 --parts paragraf89,paragraf2991

# List all tracked documents
cdx-sledovane-dokumenty list

# Check all tracked documents for changes (versions + related documents)
cdx-sledovane-dokumenty check

# Check a specific document
cdx-sledovane-dokumenty check CR13986

# Show full state of a tracked document (includes diff if change detected)
cdx-sledovane-dokumenty show CR26785

# Confirm a change (moves baseline to current version)
cdx-sledovane-dokumenty confirm CR13986

# Remove a document from tracking
cdx-sledovane-dokumenty remove CR13986

# Add a document to a group (creates the group if it doesn't exist)
cdx-sledovane-dokumenty group add CR13986 "Pracovní právo"

# Remove a document from a group
cdx-sledovane-dokumenty group remove CR13986 "Pracovní právo"

# List all groups
cdx-sledovane-dokumenty group list

# Delete a group entirely
cdx-sledovane-dokumenty group delete "Pracovní právo"

# Add a note - AI will focus on these topics when summarizing changes
cdx-sledovane-dokumenty note add CR13986 "zajímá mě pracovní doba a výpovědní lhůty"

# List notes for a document
cdx-sledovane-dokumenty note list CR13986

# Remove a note by index
cdx-sledovane-dokumenty note remove CR13986 0

# Add a relation type to track
cdx-sledovane-dokumenty related add CR13986 PROVADECI_PREDPIS

# Remove a relation type from tracking
cdx-sledovane-dokumenty related remove CR13986 PROVADECI_PREDPIS

# List currently tracked relation types
cdx-sledovane-dokumenty related list CR13986

# Show all available relation types with counts (from API)
cdx-sledovane-dokumenty related types CR13986
```

## Personalization by Profession

If the user mentions their profession (e.g. "jsem účetní", "pracuji jako advokát", "jsem HR manažer"), proactively suggest relevant documents to track based on their field. Use the `codexis` skill to find the most important laws for their profession, then offer to add them via `cdx-sledovane-dokumenty add`. For example:
- Účetní → zákon o účetnictví, zákon o DPH, zákon o daních z příjmů
- Advokát → občanský zákoník, trestní zákoník, občanský soudní řád
- HR manažer → zákoník práce, zákon o zaměstnanosti

## How It Works

1. User asks to track a document → use `cdx-sledovane-dokumenty add <codexisId>`
2. `cdx-sledovane-dokumenty` verifies the document exists in CODEXIS, saves the current version as baseline
3. After adding, run `cdx-sledovane-dokumenty group list` and assign the document to a fitting group with `cdx-sledovane-dokumenty group add`. If no suitable group exists, create one - the command creates the group automatically (e.g. "Pracovní právo", "Daňové zákony", "Soukromé právo"). A document can belong to multiple groups.
4. If the user mentions specific interests or asks questions about a tracked document (e.g. "zajímá mě pracovní doba", "dej mi vědět jestli se mění výpovědní lhůty"), always save them as notes with `cdx-sledovane-dokumenty note add <codexisId> "<text>"`. These notes personalize AI summaries when changes are detected. Do this proactively - the user doesn't need to explicitly ask for it. Write notes from the user's perspective in 1st person (e.g. "zajímá mě pracovní doba", "jsem účetní") or as instructions in 2nd person (e.g. "sleduj výpovědní lhůty", "zaměř se na BOZP"). Never write in 3rd person (e.g. "uživatel je účetní", "uživatele zajímá").
5. Later, `cdx-sledovane-dokumenty check` compares the current CODEXIS version against the baseline
5. If a new version exists, `cdx-sledovane-dokumenty` computes a text diff and stores the change
6. User reviews the change on the Sledované dokumenty app page or via `cdx-sledovane-dokumenty show`
7. `cdx-sledovane-dokumenty confirm` marks the change as reviewed and advances the baseline

## Related Document Tracking

Beyond text changes (new versions/amendments), `cdx-sledovane-dokumenty` can track changes in **related documents** - new case law, implementing regulations, related legislation, etc. The CODEXIS API provides relation types for each document.

### How it works

1. Add relation types to track: `cdx-sledovane-dokumenty related add CR13986 PROVADECI_PREDPIS`
2. `cdx-sledovane-dokumenty` downloads the current set of related document IDs for the type and saves them as a baseline
3. During `cdx-sledovane-dokumenty check`, the current related IDs are compared against the baseline - new or removed documents are reported as `related_change` entries
4. Changes appear in the UI alongside version changes

### Choosing relation types

Run `cdx-sledovane-dokumenty related types CR13986` to see all available types with counts. There are no defaults - the user (or AI) chooses which types to track based on the document and their needs.

For types with very high counts (e.g. `JUDIKATURA` with 1000+), consider whether the user really needs to track all of them - the baseline can be large and checks slower.

### Recommended workflow

**IMPORTANT:** `related` subcommands only work on documents that are already tracked. Always `add` first, then configure related tracking.

After adding a document, briefly mention that related tracking is available (e.g. "Můžu také zapnout sledování souvisejících dokumentů, chcete?"). Do NOT enable it automatically - wait for the user to ask. If the user wants it:
1. Run `cdx-sledovane-dokumenty related types <codexisId>` to see available types
2. Recommend types based on the document and user's needs
3. Add chosen types: `cdx-sledovane-dokumenty related add <codexisId> <type>`

## Finding the codexisId

If the user asks to track a law not in the table below, use the `codexis` skill to resolve the codexisId first, then come back and call `cdx-sledovane-dokumenty add`.

**Note:** The codexisId for tracking is the **base document ID** (e.g., `CR13986`), not the version ID (e.g., `CR13986_2027_01_01`). Strip the date suffix (`_YYYY_MM_DD`).

### Common codexisIds

| Law | codexisId |
|---|---|
| Zákoník práce (262/2006 Sb.) | CR13986 |
| Občanský zákoník (89/2012 Sb.) | CR26785 |
| Zákon o obchodních korporacích (90/2012 Sb.) | CR26787 |
| Zákon o DPH (235/2004 Sb.) | CR8977 |
| Zákon o daních z příjmů (586/1992 Sb.) | CR3643 |

## Automations

Two automations ship with the plugin and run on a schedule: `automations/check-dokumenty.md` (a plain `check`, weekly) for tracked acts, and `automations/check-slozky.md` (a plain `folder check`, daily) for watched folders.

**Don't create an automation for periodic checks** - the shipped ones already run them. A change is recorded once; a later run finds it already in the change log and skips it, so a second automation would have nothing to report and would never notify. The user reads changes in the Sledované dokumenty UI component, so an AI-prompt automation generating reports adds nothing either.

Only if the user wants a different cadence: disable the shipped automation first (`cdxctl automation list`, then `cdxctl automation disable <id>`), then create the replacement - otherwise the shipped run records the change first and yours sees it as already handled.

## Watched folders (legislation referenced by the user's own documents)

Besides tracking individual CODEXIS acts by id, this tool can watch a **folder of
the user's own documents** (contracts, `.docx`/`.pdf`/`.txt`) and track changes in
the **legislation those documents reference**. The user picks a folder in the
Sledované dokumenty app and starts extraction; everything below is also available
from the CLI.

```bash
# Start watching a folder (scans files, computes checksums; the daily check ships via automations/check-slozky.md)
cdx-sledovane-dokumenty folder add /home/<user>/my-contracts --name "Smlouvy"

# Harvest legislative references with AI (ONE pass per document, only docs that need it)
cdx-sledovane-dokumenty folder harvest /home/<user>/my-contracts

# List watched folders with counts + change status
cdx-sledovane-dokumenty folder list

# Deterministic change check (no AI) - this is what the daily automation runs
cdx-sledovane-dokumenty folder check

# Force a full re-harvest of a folder (e.g. user explicitly refreshes)
cdx-sledovane-dokumenty folder refresh /home/<user>/my-contracts

# Confirm detected changes (advances the baseline)
cdx-sledovane-dokumenty folder confirm /home/<user>/my-contracts

# Inspect a folder's contract + tracking state as JSON
cdx-sledovane-dokumenty folder show /home/<user>/my-contracts

# Stop watching (add --purge to also delete the hidden .watched/ folder)
cdx-sledovane-dokumenty folder remove /home/<user>/my-contracts

# Folder picker support
cdx-sledovane-dokumenty folder browse [/some/dir]

# Notification settings - a persistent app-wide setting
cdx-sledovane-dokumenty notify show
cdx-sledovane-dokumenty notify set --email --no-inapp

# Extra e-mail recipients (the signed-in user always gets a copy)
cdx-sledovane-dokumenty notify recipients add|remove|set|clear <email...>
```

The signed-in user always receives the notification e-mail; extra recipients are additive. You don't need to add their own address - it's redundant and gets dropped automatically.

| User wants | Do |
|---|---|
| alerted when a referenced act changes | `notify set --email` - `folder check` then e-mails on a real change |
| the current status in your reply | `folder show <root>` |
| a status e-mail right now, change or not | `cdxctl notification email send` (see the cdxctl skill) |

### How it works

1. `folder add` scans the tree, computes a SHA-256 per document and writes the
   skeleton `<root>/.watched/watched.json` (see
   `references/watched-json-contract.md`). It also bootstraps a daily
   `folder check` automation (idempotent).
2. `folder harvest` sends each document that needs it (new or changed checksum)
   to the daemon **once** and stores the referenced legislation back into
   `watched.json`, with a canonical `uri` and - for Czech acts - a `codexisId`.
3. `folder check` (daily, **no AI**) compares each referenced act's current
   CODEXIS version against the stored baseline and records changes.
4. On change, the user is notified **from the daemon** (e-mail and/or in-app),
   per the configurable notification settings.
5. References are re-harvested **only** when a document's checksum changes or the
   user explicitly runs `folder refresh`.

`.watched/watched.json` is a stable, documented contract - deterministic tools
read it as the integration boundary. Never hand-edit it; use the CLI.

## Storage

CODEXIS-id tracking:
- State files: `~/.cdx/apps/sledovane-dokumenty/<codexisId>/state.json`
- Related baselines: `~/.cdx/apps/sledovane-dokumenty/<codexisId>/related_<TYPE>.json`
- Groups file: `~/.cdx/apps/sledovane-dokumenty/groups.json`

Watched folders:
- Index of watched roots: `~/.cdx/apps/sledovane-dokumenty/watched-folders.json`
- Notification settings: `~/.cdx/apps/sledovane-dokumenty/folder-watch-settings.json`
- Per-folder contract: `<root>/.watched/watched.json` (+ `tracking-state.json`)

The Sledované dokumenty UI component reads from these locations automatically.

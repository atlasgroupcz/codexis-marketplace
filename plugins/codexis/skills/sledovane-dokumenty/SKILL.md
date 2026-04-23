---
uuid: 1c427cf8-4d37-4304-b2e1-fbabf245dae4
name: sledovane-dokumenty
description: Track changes in Czech legislation via CODEXIS. Use when user wants to monitor laws, track amendments, or check for legislative changes.
version: 1.0.0
i18n:
  cs:
    displayName: "Sledované dokumenty"
    summary: "Sledování změn v české legislativě přes CODEXIS — upozornění na novelizace a srovnání verzí."
  en:
    displayName: "Document Tracker"
    summary: "Monitor changes in Czech legislation via CODEXIS — amendment alerts and version diffs."
  sk:
    displayName: "Sledované dokumenty"
    summary: "Sledovanie zmien v českej legislatíve cez CODEXIS — upozornenia na novelizácie a porovnanie verzií."
---

# Sledované dokumenty — Document Tracking

Track changes in Czech legislation by monitoring CODEXIS documents for new versions and computing text diffs.

**IMPORTANT:** The only tool in this skill is `cdx-sledovane-dokumenty`. Do NOT call `cdx-cli`, `cdxctl`, `curl`, or any other tool directly.

**IMPORTANT:** Assume `cdx-cli`, `cdxctl`, and `cdx-sledovane-dokumenty` are already installed, configured, available in `PATH`, and invokable. Do not run setup or preflight checks.

**IMPORTANT:** If `cdx-sledovane-dokumenty` outputs an ERROR, stop immediately and report it to the user. Do not retry or guess.

## Output Format

`cdx-sledovane-dokumenty` has three kinds of output. Pick the right way to consume each — mixing them up (e.g. `grep` on JSON, or `jq` on plaintext) will fail.

- **`list`, `check`, `group list`, `note list`, `related list`, `related types`** — human-readable plain text. Read it directly. **Do not parse with `sed` or `grep`** — the layout is for humans, not machines. Summarize what you see for the user.

- **`add`, `remove`, `confirm`, `group add`, `group remove`, `group delete`, `note add`, `note remove`, `related add`, `related remove`** — a single `OK: ...` line on success or `ERROR: ...` on failure. No parsing needed; the whole output is the status.

- **`show`** — structured JSON (full state including diffs and related baselines). Read it directly, or filter with `jq` for a specific field. Never with `sed`/`grep`.

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

# Add a note — AI will focus on these topics when summarizing changes
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
3. After adding, run `cdx-sledovane-dokumenty group list` and assign the document to a fitting group with `cdx-sledovane-dokumenty group add`. If no suitable group exists, create one — the command creates the group automatically (e.g. "Pracovní právo", "Daňové zákony", "Soukromé právo"). A document can belong to multiple groups.
4. If the user mentions specific interests or asks questions about a tracked document (e.g. "zajímá mě pracovní doba", "dej mi vědět jestli se mění výpovědní lhůty"), always save them as notes with `cdx-sledovane-dokumenty note add <codexisId> "<text>"`. These notes personalize AI summaries when changes are detected. Do this proactively — the user doesn't need to explicitly ask for it. Write notes from the user's perspective in 1st person (e.g. "zajímá mě pracovní doba", "jsem účetní") or as instructions in 2nd person (e.g. "sleduj výpovědní lhůty", "zaměř se na BOZP"). Never write in 3rd person (e.g. "uživatel je účetní", "uživatele zajímá").
5. Later, `cdx-sledovane-dokumenty check` compares the current CODEXIS version against the baseline
5. If a new version exists, `cdx-sledovane-dokumenty` computes a text diff and stores the change
6. User reviews the change on the Sledované dokumenty app page or via `cdx-sledovane-dokumenty show`
7. `cdx-sledovane-dokumenty confirm` marks the change as reviewed and advances the baseline

## Related Document Tracking

Beyond text changes (new versions/amendments), `cdx-sledovane-dokumenty` can track changes in **related documents** — new case law, implementing regulations, related legislation, etc. The CODEXIS API provides relation types for each document.

### How it works

1. Add relation types to track: `cdx-sledovane-dokumenty related add CR13986 PROVADECI_PREDPIS`
2. `cdx-sledovane-dokumenty` downloads the current set of related document IDs for the type and saves them as a baseline
3. During `cdx-sledovane-dokumenty check`, the current related IDs are compared against the baseline — new or removed documents are reported as `related_change` entries
4. Changes appear in the UI alongside version changes

### Choosing relation types

Run `cdx-sledovane-dokumenty related types CR13986` to see all available types with counts. There are no defaults — the user (or AI) chooses which types to track based on the document and their needs.

For types with very high counts (e.g. `JUDIKATURA` with 1000+), consider whether the user really needs to track all of them — the baseline can be large and checks slower.

### Recommended workflow

**IMPORTANT:** `related` subcommands only work on documents that are already tracked. Always `add` first, then configure related tracking.

After adding a document, briefly mention that related tracking is available (e.g. "Můžu také zapnout sledování souvisejících dokumentů, chcete?"). Do NOT enable it automatically — wait for the user to ask. If the user wants it:
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

A periodic `cdx-sledovane-dokumenty check` automation is created automatically when the user adds a document. **Do NOT create additional automations**. The existing COMMAND automation handles periodic checks. The user views changes in the Sledované dokumenty UI component — there is no need for AI-generated reports via automation prompts.

## Storage

State files: `~/.cdx/apps/sledovane-dokumenty/<codexisId>/state.json`
Related baselines: `~/.cdx/apps/sledovane-dokumenty/<codexisId>/related_<TYPE>.json`
Groups file: `~/.cdx/apps/sledovane-dokumenty/groups.json`

The Sledované dokumenty UI component reads from this directory automatically.

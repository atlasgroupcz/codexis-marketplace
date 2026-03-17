---
name: sledovane-dokumenty
description: Track changes in Czech legislation via CODEXIS. Use when user wants to monitor laws, track amendments, or check for legislative changes.
version: 1.0.0
---

# Sledované dokumenty — Document Tracking

Track changes in Czech legislation by monitoring CODEXIS documents for new versions and computing text diffs.

**IMPORTANT:** The only tool in this skill is `./sd`. Do NOT call `cdx`, `curl`, or any other tool directly.

**IMPORTANT:** If `sd` outputs an ERROR, stop immediately and report it to the user. Do not retry or guess.

## Commands

```bash
# Add a document to tracking (verifies it exists, saves baseline version)
./sd add CR13986

# Add with specific paragraphs to track
./sd add CR26785 --parts paragraf89,paragraf2991

# List all tracked documents
./sd list

# Check all tracked documents for changes
./sd check

# Check a specific document
./sd check CR13986

# Show full state of a tracked document (includes diff if change detected)
./sd show CR26785

# Confirm a change (moves baseline to current version)
./sd confirm CR13986

# Remove a document from tracking
./sd remove CR13986

# Add a document to a group (creates the group if it doesn't exist)
./sd group add CR13986 "Pracovní právo"

# Remove a document from a group
./sd group remove CR13986 "Pracovní právo"

# List all groups
./sd group list

# Delete a group entirely
./sd group delete "Pracovní právo"

# Add a note — AI will focus on these topics when summarizing changes
./sd note add CR13986 "zajímá mě pracovní doba a výpovědní lhůty"

# List notes for a document
./sd note list CR13986

# Remove a note by index
./sd note remove CR13986 0
```

## Personalization by Profession

If the user mentions their profession (e.g. "jsem účetní", "pracuji jako advokát", "jsem HR manažer"), proactively suggest relevant documents to track based on their field. Use the `codexis` skill to find the most important laws for their profession, then offer to add them via `./sd add`. For example:
- Účetní → zákon o účetnictví, zákon o DPH, zákon o daních z příjmů
- Advokát → občanský zákoník, trestní zákoník, občanský soudní řád
- HR manažer → zákoník práce, zákon o zaměstnanosti

## How It Works

1. User asks to track a document → use `./sd add <codexisId>`
2. `sd` verifies the document exists in CODEXIS, saves the current version as baseline
3. After adding, run `./sd group list` and assign the document to a fitting group with `./sd group add`. If no suitable group exists, create one — the command creates the group automatically (e.g. "Pracovní právo", "Daňové zákony", "Soukromé právo"). A document can belong to multiple groups.
4. If the user mentions specific interests or asks questions about a tracked document (e.g. "zajímá mě pracovní doba", "dej mi vědět jestli se mění výpovědní lhůty"), always save them as notes with `./sd note add <codexisId> "<text>"`. These notes personalize AI summaries when changes are detected. Do this proactively — the user doesn't need to explicitly ask for it. Write notes from the user's perspective in 1st person (e.g. "zajímá mě pracovní doba", "jsem účetní") or as instructions in 2nd person (e.g. "sleduj výpovědní lhůty", "zaměř se na BOZP"). Never write in 3rd person (e.g. "uživatel je účetní", "uživatele zajímá").
5. Later, `./sd check` compares the current CODEXIS version against the baseline
5. If a new version exists, `sd` computes a text diff and stores the change
6. User reviews the change on the Sledované dokumenty app page or via `./sd show`
7. `./sd confirm` marks the change as reviewed and advances the baseline

## Finding the codexisId

If the user asks to track a law not in the table below, use the `codexis` skill to resolve the codexisId first, then come back and call `./sd add`.

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

A periodic `sd check` automation is created automatically when the user adds a document. **Do NOT create additional automations** (no `cdxctl automation create`, no agent automations with prompts). The existing COMMAND automation handles periodic checks. The user views changes in the Sledované dokumenty UI component — there is no need for AI-generated reports via automation prompts.

## Storage

State files: `~/.cdx/apps/sledovane-dokumenty/<codexisId>/state.json`
Groups file: `~/.cdx/apps/sledovane-dokumenty/groups.json`

The Sledované dokumenty UI component reads from this directory automatically.

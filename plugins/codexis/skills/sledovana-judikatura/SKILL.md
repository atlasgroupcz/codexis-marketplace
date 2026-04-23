---
uuid: 5cdb1ccf-e079-49a9-a618-a3f3bb60dc20
name: sledovana-judikatura
description: Track case law topics and generate periodic reports. Use when user wants to monitor new jurisprudence, track judicial decisions, or get regular case law summaries. Triggers on "sleduj judikaturu", "judikatura", "nová rozhodnutí", "case law monitoring", "soudní rozhodnutí".
version: 1.0.0
i18n:
  cs:
    displayName: "Sledovaná judikatura"
    summary: "Pravidelné sledování nové judikatury podle tématu s AI reporty o trendech a dopadech."
  en:
    displayName: "Case Law Monitor"
    summary: "Periodic tracking of new case law by topic with AI-generated reports on trends and impact."
  sk:
    displayName: "Sledovaná judikatúra"
    summary: "Pravidelné sledovanie novej judikatúry podľa témy s AI reportmi o trendoch a dopadoch."
---

# Sledovaná judikatura — Case Law Monitoring

Monitor Czech (and European) case law by topic. AI searches for new judicial decisions, analyzes trends, and generates periodic reports.

**IMPORTANT:** This skill uses TWO tools:
- `cdx-sledovana-judikatura` — state management (topics, areas, reports)
- `cdx-cli search JD` — searching CODEXIS case law database (use the `codexis` skill for search syntax)

**IMPORTANT:** Assume `cdx-cli`, `cdxctl`, and `cdx-sledovana-judikatura` are already installed, configured, available in `PATH`, and invokable. Do not run setup or preflight checks.

**IMPORTANT:** If `cdx-sledovana-judikatura` outputs an ERROR, stop immediately and report it to the user. Do not retry or guess.

## Output Format

`cdx-sledovana-judikatura` outputs are designed for shell composition, not parsing:

- **`init`** prints the new topic UUID on stdout (one line) and a status message on stderr. Capture with shell substitution directly:
  ```bash
  UUID=$(cdx-sledovana-judikatura init "Náhrada škody")
  ```
  Do **not** parse the output with `sed`/`grep`/`jq` — there is nothing to parse, the UUID is the entire stdout.

- **`list`**, **`show`**, **`list-reports`**, **`area list`**, **`note list`** print human-readable plain text. Read them visually; do not pipe them to parsers.

- **Action commands** (`area add`, `note add`, `set-baseline`, `delete`, `confirm-report`, `set-automation`, `touch`, `save-report`) print a single `OK: ...` line on success or `ERROR: ...` on failure.

- **`show-report`** is the only command that prints structured JSON, because the report itself is structured data (decisions table + summaries) the user wants to consume programmatically.

## State Management Commands

```bash
# Create a new tracked topic
cdx-sledovana-judikatura init "Náhrada škody"
cdx-sledovana-judikatura init "Okamžité zrušení PP" --notes "zajímá mě role GDPR"

# List all tracked topics
cdx-sledovana-judikatura list

# Show full topic state with reports summary
cdx-sledovana-judikatura show <uuid>

# Delete a topic and all its reports
cdx-sledovana-judikatura delete <uuid>

# Add a sub-area to the topic
cdx-sledovana-judikatura area add <uuid> "Odpovědnost za vady výrobku"

# Remove a sub-area by index
cdx-sledovana-judikatura area remove <uuid> 0

# List sub-areas
cdx-sledovana-judikatura area list <uuid>

# Set baseline summary for an area (AI-generated description of current judicial view)
cdx-sledovana-judikatura area set-baseline <uuid> 0 "Soudy aktuálně vychází z principu..."

# Add user notes (personalization for AI reports)
cdx-sledovana-judikatura note add <uuid> "přelomové rozsudky NS"

# List / remove notes
cdx-sledovana-judikatura note list <uuid>
cdx-sledovana-judikatura note remove <uuid> 0

# Save a report (AI-generated, from stdin or file)
cdx-sledovana-judikatura save-report <uuid> --file /tmp/report.json

# List all reports for a topic
cdx-sledovana-judikatura list-reports <uuid>

# Show full report
cdx-sledovana-judikatura show-report <uuid> <report_id>

# Mark report as read
cdx-sledovana-judikatura confirm-report <uuid> <report_id>

# Store automation ID (call after creating automation)
cdx-sledovana-judikatura set-automation <uuid> <automation_id>

# Update last_check_at without creating a report (use when no new decisions found)
cdx-sledovana-judikatura touch <uuid>
```

## Notes vs Automation

Notes store **user interests and report format preferences** — things that help AI tailor its analysis:
- "zajímá mě zejména změna ustálených výkladů"
- "formát: tabulka + shrnutí pro advokáta, HR, EN management"
- "jsem samostatný advokát, pracovní právo"

Do NOT store check frequency in notes — that belongs in the automation cron schedule.

## How It Works

### Step 1: User describes their interest

User provides a topic description, e.g.:
- "Zajímá mě nová judikatura v oblasti insolvenčních řízení"
- "Podívej se na novou judikaturu k okamžitému zrušení pracovního poměru"
- "Zajímá mě přelomová česká i evropská judikatura k náhradě škody"

### Step 2: Initialize topic and identify sub-areas

Capture the new UUID from `init` directly with shell substitution and reuse it
for subsequent commands. Example end-to-end pattern:

```bash
UUID=$(cdx-sledovana-judikatura init "Náhrada škody" --notes "samostatný advokát, zajímá mě role GDPR")
cdx-sledovana-judikatura area add "$UUID" "Odpovědnost za vady výrobku"
cdx-sledovana-judikatura area add "$UUID" "Náhrada nemajetkové újmy"
cdx-sledovana-judikatura area add "$UUID" "Promlčení nároků"
cdx-sledovana-judikatura note add "$UUID" "formát: tabulka + shrnutí pro advokáta"
```

Decisions you make in this step:

1. Whether the topic is broad enough to split into sub-areas:
   - If yes — identify 3–7 key sub-areas and `area add` each
   - If no — work with the topic as a whole (no areas needed)
   - If unclear — ask the user whether to split or how to divide it
2. What user-specific notes to save (profession, perspective, format preferences) via `--notes` on `init` and/or `note add`

### Step 3: Build baseline (initial analysis)

For each area (or the topic as a whole if no areas):

1. Search CODEXIS for relevant case law: `cdx-cli search JD --query "<relevant query>" --sort DATE --limit 20`
   - Use multiple targeted queries per area if needed
   - Apply filters: `--court`, `--doc-type`, `--from`, `--has-legal-sentence`, etc.
   - Refer to the `codexis` skill for full search syntax and flags
2. Read the most important decisions to understand the current judicial approach
3. Write a baseline summary and save it: `cdx-sledovana-judikatura area set-baseline <uuid> <index> "<summary>"`

### Step 4: Generate the first report

Create a report JSON and save it. The report should contain:

```json
{
  "report_id": "2026-03-25",
  "checked_at": "2026-03-25T10:00:00Z",
  "period_from": null,
  "period_to": "2026-03-25",
  "found_count": 5,
  "areas": [
    {
      "name": "Odpovědnost za vady výrobku",
      "documents": [
        {
          "codexisId": "JD12345",
          "title": "Rozsudek NS 25 Cdo 1234/2025",
          "spZn": "25 Cdo 1234/2025",
          "court": "Nejvyšší soud",
          "doc_type": "Rozsudek",
          "issued_on": "2026-03-10",
          "legal_sentence": "...",
          "summary": "O co šlo: ... Jak dopadlo: ...",
          "changes_established_view": true,
          "practical_conclusion": "..."
        }
      ],
      "area_summary": "V této oblasti se NS posunul směrem k..."
    }
  ],
  "overall_summary": "Celkové shrnutí trendů...",
  "summary_for_lawyer": "Pro advokáta: ...",
  "summary_for_hr": "Pro HR: ...",
  "summary_en": "Executive summary in English...",
  "confirmed_on": null
}
```

Write the report JSON to a temp file and save:
```bash
cat > /tmp/judikatura_report.json << 'REPORT_EOF'
{ ... report JSON ... }
REPORT_EOF
cdx-sledovana-judikatura save-report <uuid> --file /tmp/judikatura_report.json
```

### Step 5: Set up automation

Create a monthly automation that runs an AI agent to check for new case law. `cdxctl automation create` prints the created automation as JSON, so extract the ID with `jq -r .id` and store it on the topic — that way deleting the topic also removes the automation.

```bash
AUTO_ID=$(cdxctl automation create \
    --title "Sledovaná judikatura – Náhrada škody" \
    --cron "0 8 1 * *" \
    --prompt "Check the tracked case law topic $UUID. Load the topic state with cdx-sledovana-judikatura show $UUID. Search CODEXIS for new judicial decisions since last_check_at using cdx-cli search JD. Compare findings against the baseline summaries. Generate a report and save it with cdx-sledovana-judikatura save-report." \
    --skill "codexis:sledovana-judikatura" \
    --skill "codexis:codexis" \
    --max-turns 30 | jq -r .id)

cdx-sledovana-judikatura set-automation "$UUID" "$AUTO_ID"
```

## Tailoring Reports to User

Pay attention to what the user asks for. Different users want different report formats:

- **"Udělej to i v tabulce"** — include a structured table of decisions (consider generating a grid visualization via the visualization skill)
- **"Shrnutí pro advokáta / HR / management"** — add role-specific summaries at the end
- **"Anglický souhrn"** — add English executive summary
- **"Porovnej mezi sebou"** — compare decisions, highlight conflicts or evolution
- **"Jestli byl judikát překonán"** — check if newer decisions override earlier ones

Save these preferences as notes so automation agents replicate them:
```bash
cdx-sledovana-judikatura note add <uuid> "formát: tabulka + shrnutí pro advokáta, HR, EN management"
```

## Periodic Check (for automation agents)

When running a periodic check:

1. Load topic: `cdx-sledovana-judikatura show <uuid>`
2. Note `last_check_at` — search only for decisions since that date
3. For each area, search CODEXIS: `cdx-cli search JD --query "..." --from <last_check_at_date>`
4. If new relevant decisions found:
   - Compare against baseline summaries — does anything change the established view?
   - Generate report with `period_from` = last check, `period_to` = now
   - Save: `cdx-sledovana-judikatura save-report <uuid> --file report.json`
   - Update baseline if the judicial view has shifted: `cdx-sledovana-judikatura area set-baseline <uuid> <index> "<updated summary>"`
5. If nothing new — do not create a report, but update the last check timestamp: `cdx-sledovana-judikatura touch <uuid>`

## Storage

Topic files: `~/.cdx/apps/sledovana-judikatura/<uuid>/topic.json`
Reports: `~/.cdx/apps/sledovana-judikatura/<uuid>/reports/<report_id>.json`

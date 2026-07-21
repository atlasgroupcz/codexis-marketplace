---
uuid: b678385e-6770-40bd-a33e-1d73e790f865
name: cdxctl
icon: icon.svg
description: Manage Codexis platform objects with the cdxctl CLI - create, list, update, enable/disable, or delete custom agents, skills, scheduled automations, plugins, marketplaces, in-app frontend notifications (the bell/sheet, NOT OS notify-send/dbus), outbound email, and tabular extraction from a folder of files. Use when the user asks to build, edit, schedule, install, or remove any of these platform objects from chat.
version: 2.3.0
i18n:
  cs:
    displayName: "Správa platformy"
    summary: "Vytváření a správa vlastních agentů, dovedností, automatizací a doplňků přímo z konverzace."
  en:
    displayName: "Platform Management"
    summary: "Create and manage custom agents, skills, automations, and plugins directly from chat."
  sk:
    displayName: "Správa platformy"
    summary: "Vytváranie a správa vlastných agentov, zručností, automatizácií a doplnkov priamo z konverzácie."
---

# cdxctl - Platform Management CLI

`cdxctl` manages automations, marketplaces, and plugins via the shell. Output is JSON by default (use `--table` for human-readable).

Always go through `cdxctl` for these platform objects - never hand-edit the underlying files or reach for OS tools (no `notify-send`, `dbus`, `crontab`, manual JSON/markdown edits). The CLI is the single source of truth.

## Automations

```bash
# List all automations
cdxctl automation list

# Create an automation
cdxctl automation create \
    --title "Daily Report" \
    --repeat DAILY --day-part MORNING \
    --prompt "Generate a daily summary report" \
    --description "Runs every morning" \
    --agent "<id>" \
    --skill "<id>" \
    --skill "<id>" \
    --max-turns 10 \
    --work-dir "/home/user/project"

# Create a COMMAND automation
cdxctl automation create-command \
    --title "Tracked Documents Check" \
    --repeat WEEKLY --day-part MORNING --days-of-week 1 \
    --command "cdx-sledovane-dokumenty check" \
    --description "Weekly tracked-document change check"

# Update (partial - only specified fields change; never touches enabled)
cdxctl automation update <id> --title "New Title"
cdxctl automation update <id> --repeat WEEKLY --day-part EVENING --days-of-week 1 --days-of-week 4
cdxctl automation update <id> --prompt "Updated prompt" --max-turns 30

# Enable / disable (separate from update - see note below)
cdxctl automation enable <id>
cdxctl automation disable <id>

# Delete
cdxctl automation delete <id>

# Manually trigger a run
cdxctl automation trigger <id>
```

**Enable / disable vs update:** toggling the on/off state is its own command, not a
flag on `update`. `enable`/`disable` go through the daemon's `setAutomationEnabled`
mutation, which works for **both** your own automations and read-only automations
shipped by a plugin. `update` only edits USER automations (the daemon rejects edits
to shipped/plugin automations) and never changes the enabled state.

**Disabling a shipped (plugin) automation:** you cannot edit or delete a plugin-shipped
automation, but you can disable one - e.g. to replace it with your own on a different
schedule. Find it with `cdxctl automation list`, then `cdxctl automation disable <id>`.
The disable is remembered per user and survives plugin updates and daemon restarts.

**Required fields for create:** `--title`, `--repeat` (+ `--day-part`), `--prompt`

**Required fields for create-command:** `--title`, `--repeat` (+ `--day-part`), `--command`

**Writing `--prompt`:** The prompt is a prepared instruction the system runs later, unattended, as the first message of each scheduled run. Write it as the task the automation should perform, imperative - e.g. `--prompt "Check the documents in /home/codexis/Smlouvy and notify me about issues"`. Do **not** write it as a scheduling request ("every morning I want…"); the schedule is `--repeat`/`--day-part`, not the prompt. Do **not** put `cdxctl automation create/update/delete` (or "create an automation that…") inside the prompt - this command already creates the automation, so a nested create makes the run register more automations instead of doing the work.

**IDs:** Use the `id` or `uuid` from `automation list` output. Both Node IDs (base64) and raw UUIDs work.

## Marketplaces

```bash
# List all marketplaces
cdxctl marketplace list

# Add a git marketplace
cdxctl marketplace add --source "https://github.com/org/repo" --source-type git --git-ref "main"

# Add a local marketplace
cdxctl marketplace add --source "/path/to/local/dir" --source-type local

# Remove a marketplace by ID
cdxctl marketplace remove <id>

# Update one marketplace (git pull) by ID
cdxctl marketplace update <id>

# Update all marketplaces
cdxctl marketplace update
```

## Plugins

```bash
# List installed plugins for a marketplace ID
cdxctl plugin list --marketplace <id>

# List available (not yet installed) plugins
cdxctl plugin list --available
cdxctl plugin list --marketplace <id> --available

# Install a plugin by ID
cdxctl plugin install <id>

# Uninstall a plugin by ID
cdxctl plugin uninstall <id>
```

## Agents

```bash
cdxctl agent list --editable-only

# Write the system prompt to a temp file. Pass its path.
mkdir -p ./.tmp
cat > ./.tmp/koncipient-pp.md <<'EOF'
Posuď zákonnost výpovědi ze strany zaměstnavatele.
Na vstupu dostaneš text výpovědi, pracovní smlouvu a popis okolností.
Vycházej ze zákoníku práce (§ 48-62) a judikatury NS.
Ověř výpovědní důvod, výpovědní dobu, doručení a ochrannou dobu.
Výstupem je tabulka: zjištění, ustanovení, riziko, doporučená náprava.
EOF
cdxctl agent create --name koncipient-pp --description "Posuzuje zákonnost výpovědí podle zákoníku práce" \
  --display-name cs:"Koncipient pro pracovní právo" --display-name en:"Employment-law associate" \
  --summary cs:"Posoudí zákonnost výpovědi a navrhne nápravu." --summary en:"Judges a dismissal and proposes a fix." \
  --instructions-file ./.tmp/koncipient-pp.md

cdxctl agent update <id> --description "New description"
cdxctl agent delete <id>
```

An agent is a reusable persona. It has its own model, tools, and instructions.

Write the system prompt to a temp file under `./.tmp/`. Pass its path with `--instructions-file`.

The model and jurisdictions come from the active chat.

Every new agent needs `--display-name` and `--summary`. Pass each as `lang:text` and repeat it once per language. `--display-name` is what the user sees on the Agents page; `--name` is the technical slug the system uses (lowercase, hyphens).

Give the display name and summary in the prompt's language and in English. A Czech prompt gets Czech and English. Always add English.

`update` keeps every field you do not pass. Pass only the fields you change. On update, `--display-name cs:"…"` replaces the text for the languages you pass and keeps the rest.

Run `cdxctl skill list` first. Read the agent's job and give it every skill that fits - pass each with `--skill`, in the same run as the create. A legal agent gets the legal-research skill; a document agent gets the document skills. Contract, filing, and compliance work is legal work, so it gets a legal skill on top of any document skill. Choose the fitting skills yourself from the agent's purpose. `cdxctl` shows the accepted values for `--skill` and `--model`.

Match the legal skill to the agent's jurisdiction. Each legal skill in `cdxctl skill list` covers one country's law. Pick the one that covers the agent's jurisdiction. When the list has no legal skill for that jurisdiction, leave the legal skill out and keep the document skills.

See `references/agent-examples.md` for worked examples: the legal-vs-document choice, the jurisdiction match, restricting tools, and a non-lossy update.

Leave `--tool` and `--disallowed-tool` out so the agent keeps every tool. Set them only to restrict the agent on purpose.

When you confirm the new agent to the user, keep it plain and non-technical: the display name, what the agent does, and its skills. Say it in the user's words. Leave out the slug, the command, and other internals.

### Writing the instructions file

Use short sentences. Keep each paragraph to two or three sentences.

Say the agent's job and the input it gets. Name the sources to use, like `zákoník práce § 52`. Say what to check and the output shape, like `table: finding, section, risk`.

```
Posuď zákonnost výpovědi ze strany zaměstnavatele.
Na vstupu dostaneš text výpovědi, pracovní smlouvu a popis okolností.
Vycházej ze zákoníku práce (§ 48-62) a judikatury NS.
Ověř výpovědní důvod, výpovědní dobu, doručení a ochrannou dobu.
Výstupem je tabulka: zjištění, ustanovení, riziko, doporučená náprava.
```

## Skills

Use these commands for custom skill CRUD. `cdxctl` returns the skill `id`, `sourcePath`, and `pathInfo`, so the agent can locate `SKILL.md`, edit it in the shell, and then push the updated markdown back through the API.

```bash
# List all skills
cdxctl skill list

# List only editable custom skills
cdxctl skill list --editable-only

# Create a custom skill from an existing SKILL.md file
cdxctl skill create --file /path/to/SKILL.md

# Create a custom skill from stdin
cat <<'EOF' | cdxctl skill create --stdin
---
name: custom-skill
description: Use when the user needs a custom workflow.
---

# Instructions

Describe what the agent should do.
EOF

# Update an existing skill from a file
cdxctl skill update <id> --file /path/to/SKILL.md

# Update an existing skill from stdin
cat /path/to/SKILL.md | cdxctl skill update <id> --stdin

# Delete an editable custom skill
cdxctl skill delete <id>
```

**IDs:** `cdxctl skill update` and `cdxctl skill delete` accept the GraphQL `id` from `cdxctl skill list`, a base64 Node ID, or a raw skill name like `my-skill`.

**Current SKILL.md format:** For existing skills, preserve the `uuid` when editing. For new skills, you don't need to include it, it gets generated automatically.

**Recommended agent workflow:**
1. Run `cdxctl skill list --editable-only` to find the target skill.
2. Read the current `SKILL.md` only for reference.
3. Prepare updated content in a temporary file or via stdin.
4. Do not edit the real skill file directly.
5. Do not manually change `uuid`; preserve it only if already present.
6. Run `cdxctl skill update <id-or-name> --file <temp-path>` or use `--stdin`.
7. Use `cdxctl skill create` for new skills and `cdxctl skill delete` for removal.

## Tabular Extraction

Extract structured data from files in a folder. Define columns (what to extract), then start extraction - the backend processes each file with AI.

```bash
# Check current extraction state for a folder
cdxctl tabular status ~/invoices

# Add columns (what data to extract from each file)
cdxctl tabular add-column ~/invoices --name "Invoice Number" --col-type text --description "The invoice number or ID"
cdxctl tabular add-column ~/invoices --name "Date" --col-type date --description "Invoice date"
cdxctl tabular add-column ~/invoices --name "Total" --col-type currency --description "Total amount on the invoice"
cdxctl tabular add-column ~/invoices --name "Paid" --col-type boolean --description "Whether the invoice has been paid"
cdxctl tabular add-column ~/invoices --name "Line Items" --col-type list --description "List of items on the invoice"
cdxctl tabular add-column ~/invoices --name "Priority" --col-type tag \
    --description "Invoice priority" \
    --option "high:RED" --option "medium:YELLOW" --option "low:GREEN"

# Remove a column by ID (from status output)
cdxctl tabular remove-column ~/invoices --column-id <id>

# Start the extraction (processes all files in folder).
# Use --wait to block until the extraction reaches DONE (or FAILED) - it polls
# for you. Never hand-roll a status-polling shell loop; the terminal field is
# `status` ("DONE"/"FAILED"), not `state`.
cdxctl tabular start ~/invoices --wait

# Get results (flattened rows with column values)
cdxctl tabular results ~/invoices
```

**Column types:** `text`, `date`, `number`, `currency`, `boolean`, `list`, `tag`, `tags`

**Tag/tags types** require `--option` flags in `value:COLOR` format. Available colors: RED, GREEN, BLUE, YELLOW, ORANGE, PURPLE, PINK, CYAN, TEAL, AMBER, EMERALD, INDIGO, VIOLET, FUCHSIA, ROSE, SKY, LIME, SLATE, GRAY, ZINC, NEUTRAL, STONE.

**Workflow:** add columns → `start --wait` (blocks until DONE) → results.

## Notifications

Create file-based notifications that appear in the frontend bell/sheet. Apps and automations inside VMs use these to notify users.

```bash
# Create a notification (triggers daemon refresh automatically)
cdxctl notification create --message "Export completed: report.xlsx"

# Create with a link (clicking navigates to the URL and marks as confirmed)
cdxctl notification create \
    --message "Chat completed - click to view" \
    --link "/chat/Q2hhdDoxMjYtMzZhOTFk..."

# Create with a shell action (executed on daemon refresh)
cdxctl notification create \
    --message "Backup finished" \
    --action "echo done > /tmp/backup_status"

# Create with extra custom fields
cdxctl notification create \
    --message "New data available" \
    --extra source=pipeline \
    --extra priority=high

# List notifications (last 7 days)
cdxctl notification list

# List only unseen notifications
cdxctl notification list --unseen

# Mark a notification as seen
cdxctl notification seen <id>

# Mark a notification as confirmed
cdxctl notification confirm <id>
```

**File format:** Notifications are JSON files at `~/.cdx/notifications/YYYY/MM/DD/HH/n_{timestamp_ms}_{uuid}.json` with fields: `message` (required), `action`, `link`, `seen`, `confirmed`, plus any custom fields.

### Outbound email

Send an email. Omit `--to` to let the daemon default to the **signed-in user** (recipient is taken from the JWT). Provide `--to` / `--cc` / `--bcc` (each repeatable) to target additional addresses. Subject + body are required, attachments optional.

```bash
# Default: mail yourself
cdxctl notification email send \
    --subject "Export ready" \
    --body "Your report finished generating." \
    --attach /tmp/report.xlsx

# Explicit recipients
cdxctl notification email send \
    --to client@example.com \
    --cc colleague@example.com \
    --bcc archive@atlasgroup.cz \
    --subject "Status report" \
    --body "See attached." \
    --attach /tmp/report.pdf
```

The mail is sent from the agent's address and is threaded into the current chat, so when the
recipient hits **Reply** their answer continues this same conversation. Pass `--no-thread` for mail
that should not come back - a reply to it starts a new chat instead.

```bash
cdxctl notification email send --subject "Newsletter" --body "..." --no-thread
```

Replying into a chat requires the reply to be S/MIME signed with a trusted certificate; unsigned
replies are dropped.

## Output

- **Default:** JSON to stdout (machine-parseable)
- **Table:** Add `--table` flag for human-readable columns
- **Errors:** Printed to stderr

## Schedule Format

An automation repeats `DAILY`, `WEEKLY` or `MONTHLY` and runs inside a six-hour
window, not at an exact time. The daemon picks a time within the window that stays
the same for a given automation and day, which spreads load across users.

| Flag | Values |
|---|---|
| `--repeat` | `DAILY`, `WEEKLY`, `MONTHLY` |
| `--day-part` | `NIGHT` 00:00-06:00, `MORNING` 06:00-12:00, `AFTERNOON` 12:00-18:00, `EVENING` 18:00-24:00 (default `MORNING`) |
| `--days-of-week` | Required for `WEEKLY`. `0`=Sunday..`6`=Saturday, repeat the flag for several days |
| `--day-of-month` | Required for `MONTHLY`. `1`-`28`, so every month has the day and no run is ever skipped |

| Example | Meaning |
|---|---|
| `--repeat DAILY --day-part MORNING` | Every day, some time between 06:00 and 12:00 |
| `--repeat WEEKLY --day-part EVENING --days-of-week 1 --days-of-week 4` | Mondays and Thursdays, between 18:00 and 24:00 |
| `--repeat MONTHLY --day-part NIGHT --day-of-month 15` | 15th of each month, between 00:00 and 06:00 |

**Exact times and sub-daily schedules are not supported.** There is no "every 15
minutes" or "at 9:00 sharp" - pick the window the work belongs in.

### Deprecated: `--cron`

`--cron` still accepts a 5-field expression and translates it to the nearest
day-part window (`0 9 * * *` becomes `DAILY MORNING`). It is a client-side shim -
the daemon has no cron field any more - and it rejects anything that cannot be
expressed as a window, such as `*/15 * * * *` or `0 9 * * 1-5`. Use
`--repeat`/`--day-part` in new work.

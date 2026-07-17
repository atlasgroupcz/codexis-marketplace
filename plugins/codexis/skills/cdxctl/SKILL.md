---
uuid: b678385e-6770-40bd-a33e-1d73e790f865
name: cdxctl
icon: icon.svg
description: Manage Codexis platform objects with the cdxctl CLI — create, list, update, enable/disable, or delete custom agents, skills, scheduled automations (cron), plugins, marketplaces, in-app frontend notifications (the bell/sheet, NOT OS notify-send/dbus), outbound email, and tabular extraction from a folder of files. Use when the user asks to build, edit, schedule, install, or remove any of these platform objects from chat.
version: 1.7.0
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

# cdxctl — Platform Management CLI

`cdxctl` manages automations, marketplaces, and plugins via the shell. Output is JSON by default (use `--table` for human-readable).

Always go through `cdxctl` for these platform objects — never hand-edit the underlying files or reach for OS tools (no `notify-send`, `dbus`, `crontab`, manual JSON/markdown edits). The CLI is the single source of truth.

## Automations

```bash
# List all automations
cdxctl automation list

# Create an automation
cdxctl automation create \
    --title "Daily Report" \
    --cron "0 9 * * *" \
    --prompt "Generate a daily summary report" \
    --description "Runs every morning at 9 AM" \
    --agent "<id>" \
    --skill "<id>" \
    --skill "<id>" \
    --max-turns 10 \
    --work-dir "/home/user/project"

# Create a COMMAND automation
cdxctl automation create-command \
    --title "Tracked Documents Check" \
    --cron "0 8 * * 1" \
    --command "cdx-sledovane-dokumenty check" \
    --description "Weekly tracked-document change check"

# Update (partial — only specified fields change; never touches enabled)
cdxctl automation update <id> --title "New Title"
cdxctl automation update <id> --cron "0 */2 * * *"
cdxctl automation update <id> --prompt "Updated prompt" --max-turns 30

# Enable / disable (separate from update — see note below)
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
automation, but you can disable one — e.g. to replace it with your own on a different
schedule. Find it with `cdxctl automation list`, then `cdxctl automation disable <id>`.
The disable is remembered per user and survives plugin updates and daemon restarts.

**Required fields for create:** `--title`, `--cron` (5-field unix cron), `--prompt`

**Required fields for create-command:** `--title`, `--cron`, `--command`

**Writing `--prompt`:** The prompt is a prepared instruction the system runs later, unattended, as the first message of each scheduled run. Write it as the task the automation should perform, imperative — e.g. `--prompt "Check the documents in /home/codexis/Smlouvy and notify me about issues"`. Do **not** write it as a scheduling request ("every morning I want…"); the schedule is `--cron`, not the prompt. Do **not** put `cdxctl automation create/update/delete` (or "create an automation that…") inside the prompt — this command already creates the automation, so a nested create makes the run register more automations instead of doing the work.

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

Use these commands for custom/local agent CRUD. `cdxctl` returns the agent `id`, `sourcePath`, and `pathInfo`, so the agent can locate the markdown file in the shell, edit it, and push the updated markdown back through the API. Agents live one markdown file per agent at `~/.cdx/agents/<name>.md` on the daemon side — always go through `cdxctl agent create`/`update`, never write that file directly.

```bash
# List all agents
cdxctl agent list

# List only editable custom/local agents
cdxctl agent list --editable-only

# Create a custom/local agent from an existing markdown file
cdxctl agent create --file /path/to/local-agent.md

# Create a custom/local agent from stdin
cat <<'EOF' | cdxctl agent create --stdin
---
name: custom-local-agent
description: Helps with a focused local workflow.
tools: Read, Bash
maxTurns: 8
i18n:
  cs:
    displayName: "Vlastní lokální agent"
    summary: "Pomáhá s konkrétním lokálním workflow."
  en:
    displayName: "Custom Local Agent"
    summary: "Helps with a focused local workflow."
  sk:
    displayName: "Vlastný lokálny agent"
    summary: "Pomáha s konkrétnym lokálnym workflow."
---

You are a custom local agent.
EOF

# Update an existing agent from a file
cdxctl agent update <id> --file /path/to/local-agent.md

# Update an existing agent from stdin
cat /path/to/local-agent.md | cdxctl agent update <id> --stdin

# Delete an editable custom/local agent
cdxctl agent delete <id>
```

**IDs:** `cdxctl agent update` and `cdxctl agent delete` accept the GraphQL `id` from `cdxctl agent list`, a base64 Node ID, or a raw local agent name like `my-agent`.

**Agent frontmatter contract:**
- `name` — required, `^[a-z][a-z0-9-]*$`, max 64 chars, no consecutive hyphens (e.g. `contract-reviewer`). If omitted from a `--file` payload, it falls back to the filename.
- `description` — required, max 1024 chars, one sentence. This is tool-facing context, not the user-facing summary.
- `uuid` — optional; the daemon generates one automatically when omitted. Only include it when updating an existing agent (preserve the original) — never invent one for a new agent.
- `model`, `maxTurns`, `tools`, `disallowedTools`, `skills`, `jurisdictions` — all optional; set only what the user actually asked for.
- `i18n.<lang>.displayName` and `i18n.<lang>.summary` — **mandatory** for `cs`, `sk`, and `en`. This is what the Agents page renders instead of the raw `name` slug, so a missing translation shows a blank or fallback title. `displayName` must be human-formatted (e.g. "Contract Reviewer", not `contract-reviewer`); `summary` is a one-sentence, user-facing description in that language — distinct from the frontmatter-level `description`, which stays a single technical sentence for tool-facing context.
- The persona body (everything after the closing `---`) is the system prompt: who the agent is, what it is expert at, how it answers, what it refuses. Write it in the user's language.

**Creating an agent from a persona description:** when the user describes an agent they want (rather than handing you a ready markdown file), design it yourself before calling `cdxctl agent create`:
1. If the purpose, name, or personality is unclear, ask one short clarifying question before writing anything.
2. Pick a concise `name` slug, a one-sentence `description`, and write the persona instructions that define the agent's expertise, tone, and boundaries.
3. Fill in `i18n.cs`, `i18n.sk`, and `i18n.en` with a human-formatted `displayName` and a one-sentence `summary` for each.
4. Run `cdxctl agent create --stdin` with the composed markdown. `cdxctl` rejects a `name` that collides with an existing agent — on that error, pick a different name and retry instead of overwriting the existing one.
5. Confirm to the user what was created and that the agent is now available on the Agents page.

**Recommended agent edit workflow:**
1. Run `cdxctl agent list --editable-only` to find the target agent and its `sourcePath.absolutePath`.
2. Read or edit the markdown file in the shell.
3. Apply the change with `cdxctl agent update <id> --file <path>` or pipe the markdown with `--stdin`.
4. Use `cdxctl agent create` for new local agents and `cdxctl agent delete` for removal.

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

Extract structured data from files in a folder. Define columns (what to extract), then start extraction — the backend processes each file with AI.

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
# Use --wait to block until the extraction reaches DONE (or FAILED) — it polls
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
    --message "Chat completed — click to view" \
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
that should not come back — a reply to it starts a new chat instead.

```bash
cdxctl notification email send --subject "Newsletter" --body "..." --no-thread
```

Replying into a chat requires the reply to be S/MIME signed with a trusted certificate; unsigned
replies are dropped.

## Output

- **Default:** JSON to stdout (machine-parseable)
- **Table:** Add `--table` flag for human-readable columns
- **Errors:** Printed to stderr

## Cron Format

5-field unix cron: `minute hour day-of-month month day-of-week`

| Expression | Meaning |
|---|---|
| `0 9 * * *` | Daily at 9:00 |
| `0 */2 * * *` | Every 2 hours |
| `*/15 * * * *` | Every 15 minutes |
| `0 9 * * 1-5` | Weekdays at 9:00 |
| `0 0 1 * *` | First day of each month at midnight |

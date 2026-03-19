---
name: cdxctl
description: Use when the user asks to create, list, update, or delete custom/local agents or skills, create, list, update, delete, or trigger automations, manage plugin marketplaces (add, remove, update), install or uninstall plugins, extract tabular data from files in a folder, or create, list, and manage notifications. Provides the cdxctl CLI for platform management operations.
version: 1.4.0
---

# cdxctl — Platform Management CLI

`cdxctl` manages automations, marketplaces, and plugins via the shell. Output is JSON by default (use `--table` for human-readable).

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
    --agent "plugin:agent-name" \
    --skill "plugin:skill-a" \
    --skill "plugin:skill-b" \
    --max-turns 10 \
    --work-dir "/home/user/project"

# Create a COMMAND automation
cdxctl automation create-command \
    --title "Tracked Documents Check" \
    --cron "0 8 * * 1" \
    --command "cdx-sledovane-dokumenty check" \
    --description "Weekly tracked-document change check"

# Update (partial — only specified fields change)
cdxctl automation update <id> --title "New Title"
cdxctl automation update <id> --cron "0 */2 * * *" --enabled true
cdxctl automation update <id> --prompt "Updated prompt" --max-turns 30

# Delete
cdxctl automation delete <id>

# Manually trigger a run
cdxctl automation trigger <id>
```

**Required fields for create:** `--title`, `--cron` (5-field unix cron), `--prompt`

**Required fields for create-command:** `--title`, `--cron`, `--command`

**IDs:** Use the `id` or `uuid` from `automation list` output. Both Node IDs (base64) and raw UUIDs work.

## Marketplaces

```bash
# List all marketplaces
cdxctl marketplace list

# Add a git marketplace
cdxctl marketplace add --source "https://github.com/org/repo" --type git --ref "main"

# Add a local marketplace
cdxctl marketplace add --source "/path/to/local/dir" --type local

# Remove a marketplace by name
cdxctl marketplace remove <name>

# Update one marketplace (git pull)
cdxctl marketplace update <name>

# Update all marketplaces
cdxctl marketplace update
```

## Plugins

```bash
# List installed plugins for a marketplace
cdxctl plugin list --marketplace "marketplace-name"

# List available (not yet installed) plugins
cdxctl plugin list --available
cdxctl plugin list --marketplace "marketplace-name" --available

# Install a plugin
cdxctl plugin install --marketplace "marketplace-name" --name "plugin-name"

# Uninstall a plugin
cdxctl plugin uninstall --marketplace "marketplace-name" --name "plugin-name"
```

## Agents

Use these commands for custom/local agent CRUD. `cdxctl` returns the agent `id`, `sourcePath`, and `pathInfo`, so the agent can locate the markdown file in the shell, edit it, and push the updated markdown back through the API.

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
---

You are a custom local agent.
EOF

# Update an existing agent from a file
cdxctl agent update <id-or-name> --file /path/to/local-agent.md

# Update an existing agent from stdin
cat /path/to/local-agent.md | cdxctl agent update <id-or-name> --stdin

# Delete an editable custom/local agent
cdxctl agent delete <id-or-name>
```

**IDs:** `cdxctl agent update` and `cdxctl agent delete` accept the GraphQL `id` from `cdxctl agent list`, a base64 Node ID, or a raw local agent name like `my-agent`.

**Recommended agent workflow:**
1. Run `cdxctl agent list --editable-only` to find the target agent and its `sourcePath.absolutePath`.
2. Read or edit the markdown file in the shell.
3. Apply the change with `cdxctl agent update <id-or-name> --file <path>` or pipe the markdown with `--stdin`.
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
cdxctl skill update <id-or-name> --file /path/to/SKILL.md

# Update an existing skill from stdin
cat /path/to/SKILL.md | cdxctl skill update <id-or-name> --stdin

# Delete an editable custom skill
cdxctl skill delete <id-or-name>
```

**IDs:** `cdxctl skill update` and `cdxctl skill delete` accept the GraphQL `id` from `cdxctl skill list`, a base64 Node ID, or a raw skill name like `my-skill`.

**Recommended agent workflow:**
1. Run `cdxctl skill list --editable-only` to find the target skill and its `sourcePath.absolutePath`.
2. Read or edit the `SKILL.md` file in the shell.
3. Apply the change with `cdxctl skill update <id-or-name> --file <path>` or pipe the markdown with `--stdin`.
4. Use `cdxctl skill create` for new custom skills and `cdxctl skill delete` for removal.

## Tabular Extraction

Extract structured data from files in a folder. Define columns (what to extract), then start extraction — the backend processes each file with AI.

```bash
# Check current extraction state for a folder
cdxctl tabular status ~/invoices

# Add columns (what data to extract from each file)
cdxctl tabular add-column ~/invoices --name "Invoice Number" --type text --description "The invoice number or ID"
cdxctl tabular add-column ~/invoices --name "Date" --type date --description "Invoice date"
cdxctl tabular add-column ~/invoices --name "Total" --type currency --description "Total amount on the invoice"
cdxctl tabular add-column ~/invoices --name "Paid" --type boolean --description "Whether the invoice has been paid"
cdxctl tabular add-column ~/invoices --name "Line Items" --type list --description "List of items on the invoice"
cdxctl tabular add-column ~/invoices --name "Priority" --type tag \
    --description "Invoice priority" \
    --option "high:RED" --option "medium:YELLOW" --option "low:GREEN"

# Remove a column by ID (from status output)
cdxctl tabular remove-column ~/invoices --column-id <id>

# Start the extraction (processes all files in folder)
cdxctl tabular start ~/invoices

# Get results (flattened rows with column values)
cdxctl tabular results ~/invoices
```

**Column types:** `text`, `date`, `number`, `currency`, `boolean`, `list`, `tag`, `tags`

**Tag/tags types** require `--option` flags in `value:COLOR` format. Available colors: RED, GREEN, BLUE, YELLOW, ORANGE, PURPLE, PINK, CYAN, TEAL, AMBER, EMERALD, INDIGO, VIOLET, FUCHSIA, ROSE, SKY, LIME, SLATE, GRAY, ZINC, NEUTRAL, STONE.

**Workflow:** add columns → start → poll status/results until done.

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

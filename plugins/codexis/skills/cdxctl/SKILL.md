---
name: cdxctl
description: Use when the user asks to create, list, update, delete, or trigger automations, manage plugin marketplaces (add, remove, update), install/uninstall plugins, or extract tabular data from files in a folder. Provides the cdxctl CLI for platform management operations.
version: 1.1.0
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

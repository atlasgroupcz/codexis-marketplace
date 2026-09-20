# Automation examples

Worked `cdxctl automation create` calls. Copy the shape, swap the content. An automation runs
the way the chat it is created in is set up (model, jurisdiction, skills) and in the chat's
current working directory, so pass a flag only to override the chat.

## Daily legal monitoring (inherits the chat setup)

Create it from a chat already on the right model and jurisdiction, with the skill selected.

```bash
cdxctl automation create \
  --title "Judikatura watch" \
  --repeat DAILY --day-part MORNING \
  --prompt "Report new Supreme Court rulings on employment notice periods from the last day."
```

## Same, overriding the setup explicitly

```bash
cdxctl automation create \
  --title "Judikatura watch" \
  --repeat DAILY --day-part MORNING \
  --prompt "Report new Supreme Court rulings on employment notice periods from the last day." \
  --model CLAUDE_SONNET_5 \
  --jurisdiction CZ \
  --skill sledovana-judikatura
```

## Contract check with a project, a connection and an input file

The run works in the project "Audit" (its files at `~/project/files`), sees the drive
"Smlouvy" at `~/Smlouvy`, and reads the uploaded checklist at `~/checklist.md`.
Without `--project` and `--connection` the automation takes the project and the connections
of the chat it is created in.

```bash
cdxctl automation create \
  --title "Contract check" \
  --repeat WEEKLY --day-part MORNING --days-of-week 1 \
  --prompt "Compare every contract in ~/Smlouvy with ~/checklist.md and list the gaps." \
  --project "Audit" \
  --connection "Smlouvy" \
  --file ./checklist.md
```

Add another input file later:

```bash
cdxctl automation update <id> --file ./exceptions.md
```

See what the automation stores, add to it, and drop what it no longer needs:

```bash
cdxctl automation files list <id>
cdxctl automation files add <id> --file ./exceptions.md
cdxctl automation files remove <id> checklist.md
```

## Weekly command automation

Runs a shell command, not an agent turn. No model or jurisdiction needed.

```bash
cdxctl automation create-command \
  --title "Tracked documents check" \
  --repeat WEEKLY --day-part MORNING --days-of-week 1 \
  --command "cdx-sledovane-dokumenty check"
```

## Updating (keeps what you do not pass)

```bash
cdxctl automation update <id> --day-part EVENING
```

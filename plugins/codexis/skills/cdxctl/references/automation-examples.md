# Automation examples

Worked `cdxctl automation create` calls. Copy the shape, swap the content. An automation runs
the way the chat it is created in is set up (model, jurisdiction, skills, work dir), so pass a
flag only to override the chat.

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
  --skill sledovana-judikatura \
  --work-dir /home/codexis/judikatura
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

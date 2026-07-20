---
uuid: b678385e-6770-40bd-a33e-1d73e790f865
name: cdxctl
icon: icon.svg
description: Manage Codexis platform objects with the cdxctl CLI — create, list, update, enable/disable, or delete custom agents, skills, scheduled automations, plugins, marketplaces, in-app frontend notifications (the bell/sheet, NOT OS notify-send/dbus), outbound email, and tabular extraction from a folder of files. Use when the user asks to build, edit, schedule, install, or remove any of these platform objects from chat.
version: 2.0.0
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

# cdxctl

`cdxctl` manages platform objects from the shell. Output is JSON; add `--table` for humans. Always go through it — never hand-edit the underlying files and never reach for `notify-send`, `dbus` or `crontab`.

## Agents

```bash
cdxctl agent defaults                 # model and jurisdictions of this chat, plus the turn budget
cdxctl agent list --editable-only
cdxctl agent create --name hr-vypoved --description "Posuzuje zákonnost výpovědi" \
  --model "$(cdxctl agent defaults | jq -r .model)" \
  --tool shell --memory "Posuď zákonnost výpovědi. …"
cdxctl agent update <id> --description "New description"
cdxctl agent delete <id>
```

An agent is a reusable persona with its own model, tools and instructions. **Run `cdxctl agent defaults` first** — it reports the model and jurisdictions of the chat you are in, and those are what the user is working with. Pass them to `create` unless the user asks for something else; never invent a model or ask which one to use. `update` keeps every field you do not pass, and a wrong skill or model is rejected with the list of accepted values.

### Writing `--memory`

Five sentences, one job each, and skip the role — it is already in the name. In order: the task, the input, the sources to use, what to check, the output and its shape.

Name a concrete source in sentence 3 and make sentence 5 checkable. "Search the law" and "answer clearly" are worthless; "zákoník práce § 52" and "table: finding, section, risk" are instructions.

```
Posuď zákonnost výpovědi ze strany zaměstnavatele.
Na vstupu dostaneš text výpovědi, pracovní smlouvu a popis okolností.
Vycházej ze zákoníku práce (§ 48–62) a judikatury NS.
Ověř výpovědní důvod, výpovědní dobu, doručení a ochrannou dobu.
Výstupem je tabulka: zjištění, ustanovení, riziko, doporučená náprava.
```

## Automations

An automation runs a prompt or a shell command on a schedule.

```bash
cdxctl automation list
cdxctl automation create --title "AML screening" --prompt "…" --repeat MONTHLY \
  --day-part NIGHT --day-of-month 15 --agent hr-vypoved --max-turns 25
cdxctl automation create-command --title "Update plugins" \
  --command "cdxctl marketplace update" --repeat DAILY --day-part NIGHT
cdxctl automation update <id> --title "New title"
cdxctl automation enable <id>   # or disable / trigger / delete
```

Write `--prompt` the same way as `--memory` above. An automation may never create another automation.

### Schedule

A run lands somewhere inside a six-hour window, never at an exact time. The daemon keeps that time stable per automation and day, which spreads load across users.

- `--repeat` — `DAILY`, `WEEKLY`, `MONTHLY`
- `--day-part` — `NIGHT` 00-06, `MORNING` 06-12, `AFTERNOON` 12-18, `EVENING` 18-24
- `--days-of-week` — `WEEKLY` only. `0`=Sunday..`6`=Saturday, repeat the flag
- `--day-of-month` — `MONTHLY` only. `1`-`28`, so no month skips it

There is no "every 15 minutes" and no "at 9:00 sharp". `--cron` still maps a 5-field expression onto the nearest window, but refuse anything finer.

## Skills

A skill is a markdown instruction file the AI loads on demand. Keep the `uuid` when editing an existing one; new skills get one generated.

```bash
cdxctl skill list --editable-only
cdxctl skill create --file ./SKILL.md      # or --stdin
cdxctl skill update <id> --file ./SKILL.md
cdxctl skill delete <id>
```

## Marketplaces, plugins and the rest

```bash
cdxctl marketplace list|add|remove|update
cdxctl plugin list|install <id>|uninstall <id>
cdxctl notification create --title "Done" --body "…"   # the in-app bell
cdxctl notification email send --to a@b.cz --subject "…" --body "…"
cdxctl tabular status|add-column|remove-column|start|results
```

Every `<id>` accepts the GraphQL id, a base64 node id, or the plain name shown by `list`.

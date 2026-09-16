---
uuid: b678385e-6770-40bd-a33e-1d73e790f865
name: cdxctl
icon: icon.svg
description: Manages Codexis platform objects with the cdxctl CLI - creates, lists, updates, enables/disables, or deletes custom agents, skills, scheduled automations, plugins, marketplaces, in-app frontend notifications (the bell/sheet, NOT OS notify-send/dbus), outbound email, and tabular extraction from a folder of files, installs and removes Ubuntu packages on the user VM, stores a plugin's secrets and settings as env values for it (API keys a plugin asks for), and files bug reports about the platform itself. Use when the user asks to build, edit, schedule, install, or remove any of these platform objects from chat, or when something in the platform is broken and should be reported (report a bug, nahlásit chybu, něco nefunguje).
version: 2.9.0
i18n:
  cs:
    displayName: "Správa platformy"
    summary: "Vytváření a správa vlastních agentů, dovedností, automatizací a doplňků přímo z konverzace."
    examplePrompts:
      - "Vytvoř automatizaci, která mi každé pondělí ráno zkontroluje sledované dokumenty."
      - "Vypiš moje automatizace a tu, která běží denně, mi vypni."
      - "Vytvoř agenta, který posoudí zákonnost výpovědi podle zákoníku práce."
  en:
    displayName: "Platform Management"
    summary: "Create and manage custom agents, skills, automations, and plugins directly from chat."
    examplePrompts:
      - "Create an automation that checks my tracked documents every Monday morning."
      - "List my automations and disable the one that runs daily."
      - "Create an agent that reviews whether a dismissal is lawful under the Czech Labour Code."
  sk:
    displayName: "Správa platformy"
    summary: "Vytváranie a správa vlastných agentov, zručností, automatizácií a doplnkov priamo z konverzácie."
    examplePrompts:
      - "Vytvor automatizáciu, ktorá mi každý pondelok ráno skontroluje sledované dokumenty."
      - "Vypíš moje automatizácie a tú, ktorá beží denne, mi vypni."
      - "Vytvor agenta, ktorý posúdi zákonnosť výpovede podľa českého zákonníka práce."
---

# cdxctl - Platform Management CLI

`cdxctl` manages agents, skills, automations, marketplaces, plugins, notifications, outbound email, and tabular extraction via the shell. Output is JSON by default (use `--table` for human-readable).

Always go through `cdxctl` for these platform objects - never hand-edit the underlying files or reach for OS tools (no `notify-send`, `dbus`, `crontab`, manual JSON/markdown edits). The CLI is the single source of truth.

## VM packages

Use `cdxctl package` when a task needs an Ubuntu package on the VM. A change applies to the
whole VM and every chat sandbox of this user. Packages survive reboots but not a replacement of
the VM system image, so install again when a tool is missing.

```bash
cdxctl package show ffmpeg      # installed?, installed and candidate version
cdxctl package install ffmpeg   # apt-get update, then apt-get install
cdxctl package remove ffmpeg
cdxctl package update           # apt-get update only
```

`install`, `remove` and `update` wait for apt to finish and print `{"success", "output"}`;
when apt fails the command exits with code 7 and the apt output on stderr. `show` exits nonzero
when apt does not know the package. Use exact package names; versions, APT flags, local
`.deb` files, repositories, and keys are not accepted.

## App env

A plugin that needs a user-provided secret (an API key for its data source) keeps it in
`~/.cdx/apps/<app>/.env`. The plugin reads that file itself; only `cdxctl appenv` writes it,
because the file sits outside the sandbox. `<app>` is the plugin name (`katastr`, `cdx-sk`).

```bash
cdxctl appenv katastr set API_KEY <value>   # add or replace API_KEY=<value>
cdxctl appenv katastr delete API_KEY        # drop the line; other lines stay
```

Use it when a plugin has no settings command of its own and the user hands you the key, or
when the user asks to remove a stored key. When the plugin ships one
(`katastr-cli settings set <KEY>`), call that instead, because it validates the key before
storing it. Names are env var names
(`API_KEY`, not `api-key`); a value must be one line. The command prints `{"success",
"output"}` and exits 7 with the script output when the write fails, 6 when the daemon has no
such endpoint. Never `cat` or edit the `.env` file by hand.

## Automations

An automation is a schedule plus a prompt (or a shell command) that runs unattended. It runs the way the chat it was created in is set up: same model and its settings, same jurisdiction, same skills, same project and connections. Every run is an isolated chat of its own: the project's files are mounted read-only at `~/project/files`, the connections under `~/<connection name>`, and the files stored on the automation read-only at `~/automation`. So create it from a chat already configured for the job, and pass only the flags you want to change.

```bash
# List all automations
cdxctl automation list

# Create an automation (inherits model + jurisdiction + skills + project + connections from this chat)
cdxctl automation create \
    --title "Daily Report" \
    --repeat DAILY --day-part MORNING \
    --prompt "Generate a daily summary report"

# Override the inherited setup when needed
cdxctl automation create \
    --title "Judikatura watch" \
    --repeat DAILY --day-part MORNING \
    --prompt "Report new Supreme Court rulings on notice periods" \
    --model CLAUDE_SONNET_5 --jurisdiction CZ \
    --skill "<id>"

# Give the runs a project, a connection and input files
cdxctl automation create \
    --title "Contract check" \
    --repeat WEEKLY --day-part MORNING --days-of-week 1 \
    --prompt "Compare the contracts in ~/Smlouvy with the checklist in ~/automation/checklist.md" \
    --project "Audit" \
    --connection "Smlouvy" \
    --file ./checklist.md

# Create a COMMAND automation
cdxctl automation create-command \
    --title "Tracked Documents Check" \
    --repeat WEEKLY --day-part MORNING --days-of-week 1 \
    --command "cdx-sledovane-dokumenty check"

# Update (partial - only the flags you pass change; enabled is untouched)
cdxctl automation update <id> --title "New Title"
cdxctl automation update <id> --repeat WEEKLY --day-part EVENING --days-of-week 1 --days-of-week 4

# Enable / disable (separate from update - see note below)
cdxctl automation enable <id>
cdxctl automation disable <id>

# Delete, or trigger a run now
cdxctl automation delete <id>
cdxctl automation trigger <id>
```

**Model + jurisdiction:** the model, its tuned settings, and the jurisdiction all default to the active chat, so usually you pass nothing. Override the model with `--model` (the enum name, e.g. `CLAUDE_SONNET_5`) and jurisdiction with `--jurisdiction` (`CZ`, `SK`, `EU`, ...; repeatable). On `update` they are kept unless you pass them again.

**Project, connections, files:** the project and the connections also default to the active chat. `--project` takes a project name or id (the run sees its files at `~/project/files`); `--connection` takes a cloud drive or drive name or id, repeatable (mounted at `~/<name>`; on `update` the list replaces the stored one). `--file <path>` uploads a file from this VM to the automation, repeatable; the runs see the stored files read-only at `~/automation/<file name>`, and `update --file` adds more. `automation list` shows `project` and `connections`; the stored files have their own commands.

**Files on an automation:** `cdxctl automation files list <id>` shows what the automation stores, `cdxctl automation files add <id> --file <path>` (repeatable) uploads more from this VM, and `cdxctl automation files remove <id> <name>` deletes one by the name the listing prints. This works on a saved automation; `create --file` / `update --file` are the same store reached while writing the automation itself.

**Enable / disable vs update:** toggling on/off is its own command, not a flag on `update`. `enable`/`disable` work for your own automations and for read-only ones shipped by a plugin. `update` only edits USER automations and never changes the enabled state.

**Disabling a shipped (plugin) automation:** you cannot edit or delete it, but you can disable it - find it with `cdxctl automation list`, then `cdxctl automation disable <id>`. The override is per user and survives plugin updates and daemon restarts.

**Required for create:** `--title`, `--repeat` (+ `--day-part`), `--prompt`. For create-command: the same, with `--command` instead of `--prompt`.

**Writing `--prompt`:** keep it short - one or two imperative sentences naming the task, e.g. `--prompt "Check the documents in /home/codexis/Smlouvy and notify me about issues"`. It is the first message of each run, not a scheduling request (the schedule is `--repeat`/`--day-part`). Never put `cdxctl automation create/update` inside the prompt - this command already creates the automation, so a nested create would just register more automations instead of doing the work.

**IDs:** use the `id` from `automation list`. A raw UUID works too - it is encoded
into the Node ID for you.

More worked calls: `references/automation-examples.md`.

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
  --example-prompt cs:"Posuď zákonnost této výpovědi a navrhni nápravu." \
  --example-prompt cs:"Zkontroluj, zda byla dodržena výpovědní doba." \
  --example-prompt cs:"Shrň rizika této výpovědi do tabulky." \
  --example-prompt en:"Judge whether this dismissal is lawful and propose a fix." \
  --example-prompt en:"Check whether the notice period was observed." \
  --example-prompt en:"Summarize the risks of this dismissal in a table." \
  --instructions-file ./.tmp/koncipient-pp.md

cdxctl agent update <id> --description "New description"
cdxctl agent delete <id>
```

An agent is a reusable persona. It has its own model, tools, and instructions.

Write the system prompt to a temp file under `./.tmp/`. Pass its path with `--instructions-file`.

The model and jurisdictions come from the active chat.

Every new agent needs `--display-name` and `--summary`. Pass each as `lang:text` and repeat it once per language. `--display-name` is what the user sees on the Agents page; `--name` is the technical slug the system uses (lowercase, hyphens).

Give the display name and summary in the prompt's language and in English. A Czech prompt gets Czech and English. Always add English.

Every new agent also gets **three `--example-prompt` per language** - the same `lang:text` shape, repeated once per prompt, in the order they should appear. They are what the user clicks to start a run, so write them concrete and directly runnable: a real sentence that asks this agent for one of its jobs, not a category label like "smlouvy" and not a `«fill this in»` placeholder. Cover three different things the agent does rather than one question reworded, and keep each to one sentence.

`update` keeps every field you do not pass. Pass only the fields you change. List flags (`--skill`, `--tool`, `--disallowed-tool`, `--jurisdiction`) replace the whole stored list - to add one skill, pass the full list including the skills the agent already has. `--display-name`, `--summary` and `--example-prompt` are per-language: the languages you pass are replaced, the rest stay. Repeating `--example-prompt` for one language replaces that language's whole list, so pass all three prompts when you change one.

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

A skill is reusable know-how the AI loads on demand: the method for a task - the steps, which sources to use, how to check them, which tools help. That is different from an agent, which is a persona. If the user did not clearly ask for a skill, ask first.

```bash
cdxctl skill list --editable-only

# Write the skill body to a temp file, then create the skill from flags.
mkdir -p ./.tmp
cat > ./.tmp/vypoved-kontrola.md <<'EOF'
Posuď zákonnost výpovědi ze strany zaměstnavatele.
Na vstupu dostaneš text výpovědi a pracovní smlouvu.
Vycházej ze zákoníku práce (§ 48-62) a judikatury NS.
Výstupem je tabulka: zjištění, ustanovení, riziko, náprava.
EOF
cdxctl skill create --name vypoved-kontrola \
  --description "Use when the user needs a Czech employment dismissal reviewed for lawfulness." \
  --display-name cs:"Kontrola výpovědi" --display-name en:"Dismissal review" \
  --summary cs:"Posoudí zákonnost výpovědi a navrhne nápravu." --summary en:"Judges a dismissal and proposes a fix." \
  --example-prompt cs:"Posuď zákonnost této výpovědi a navrhni nápravu." \
  --example-prompt en:"Judge whether this dismissal is lawful and propose a fix." \
  --jurisdiction CZ \
  --instructions-file ./.tmp/vypoved-kontrola.md

cdxctl skill update <id> --description "New trigger text"
cdxctl skill delete <id>
```

Describe the skill with flags; the whole SKILL.md is built for you.

Write the skill body to a temp file under `./.tmp/` and pass its path with `--instructions-file`.

`--description` is the AI trigger text - write it as "Use when …" with the keywords that route to this skill, and say what it is NOT for when a sibling skill is close. Keep it under 1024 characters.

Every new skill needs `--display-name` and `--summary`. Pass each as `lang:text` and repeat once per language - the prompt's language and English. `--display-name` is what the user sees on the Skills page; `--name` is the technical slug (lowercase, hyphens).

`--example-prompt` takes the same `lang:text` shape and is repeated once per prompt, in the order they should appear. At most three per language. Optional on a skill, which the AI reaches for on its own - unlike an agent, which the user starts by clicking a prompt.

Give the skill `--jurisdiction` when it covers one country's law (e.g. `CZ`), and `--jurisdiction-required` when the AI should reach for it only in that jurisdiction. Leave `--allowed-tool` out so the skill keeps every tool; set it only to restrict on purpose.

`update` keeps every field you do not pass. List flags (`--jurisdiction`, `--allowed-tool`) replace the whole stored list. `--display-name`, `--summary` and `--example-prompt` are per-language: the languages you pass are replaced, the rest stay.

When you confirm the new skill to the user, keep it plain: the display name, what it does, its jurisdiction. Leave out the slug, the command, and other internals.

See `references/skill-examples.md` for worked examples.

### What makes a good skill

A precise trigger `--description` with the keywords and what it is not for; a concrete method (steps, named sources like `§ 52`, the output shape); decision guidance for when to use it over a close sibling; and, for depth, extra files next to the SKILL.md that the body points at. Short sentences, two or three per paragraph.

**IDs:** `cdxctl skill update` and `cdxctl skill delete` accept the GraphQL `id` from `cdxctl skill list`, a base64 Node ID, or a raw skill name.

## Projects

A project is a named container the user starts chats from. It carries instructions every chat
under it follows, and shared folders - the drives and cloud drives mounted into those chats.
Its own files live in the directory `project list` prints as `directoryPath`.

```bash
# List projects with their shared folders
cdxctl project list

# One project in full, by name or ID
cdxctl project get "Audit 2026"

# Create a project with two shared folders
cdxctl project create \
    --name "Audit 2026" \
    --description "Roční audit dodavatelských smluv" \
    --instructions-file ./pokyny.md \
    --drive "Smlouvy" \
    --drive "Judikatura"

# Update (partial - only the flags you pass change)
cdxctl project update "Audit 2026" --instructions "Vycházej jen ze smluv v projektu."

# Replace the whole shared-folder list
cdxctl project update "Audit 2026" --drive "Smlouvy 2026"
```

Name a shared folder in `--drive` by its name, as `cdxctl automation create --connection` does;
a Node ID works too. The candidates are the user's own drives, the drives shared with them, and
their cloud drives. An unknown name fails the command and saves nothing.

`--drive` replaces the whole list rather than adding to it, so pass every folder the project
should keep. Write long instructions to a temp file and pass `--instructions-file`.

Two limits worth knowing before promising anything:

- **Folders from the user's computer are read-only here.** Only the desktop app can register
  one, because it alone knows the device. `project update` carries the ones already stored
  forward untouched; the CLI can neither add nor remove them.
- **Agents and skills do not belong to a project.** They are the user's own and are available in
  every chat, so there is nothing to attach. Use `cdxctl agent` and `cdxctl skill`.

Projects cannot be deleted from the CLI - the daemon has no such mutation. Point the user at the
app.

**IDs:** `cdxctl project get` and `cdxctl project update` accept the project name, the GraphQL
`id` from `cdxctl project list`, or a raw project UUID.

## Tabular Extraction

Extract structured data from files in a folder. Define columns (what to extract), then start extraction - the backend processes each file with AI.

The folder is a path inside one isolation. Without `--project` it is a folder of the current chat's sandbox (`/home/codexis/...` as the chat sees it); `--project <id>` reads the folder in that project's files instead, where `~` is the project's file root. The project id comes from the app's project list or from `cdx-sledovane-dokumenty project available`.

```bash
# Check current extraction state for a folder
cdxctl tabular status ~/invoices

# The same folder inside a project's files
cdxctl tabular status ~/invoices --project 5b1c1c4e-0000-4000-8000-000000000001

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

# Create with extra custom fields
cdxctl notification create \
    --message "New data available" \
    --extra source=pipeline \
    --extra priority=high

# List notifications (newest 50)
cdxctl notification list

# Fetch more, or reach older ones
cdxctl notification list --limit 200
cdxctl notification list --limit 50 --offset 50

# List only unseen notifications
cdxctl notification list --unseen

# Mark a notification as seen
cdxctl notification seen <id>

# Mark a notification as confirmed
cdxctl notification confirm <id>
```

**Paging:** `list` returns the newest 50 unless `--limit` says otherwise. There is
no filter by age. `--unseen` filters what was fetched, so raise `--limit` when the
unread one is older than the newest 50.

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

## Bug reports

When you hit something that is broken in the platform itself - a skill that returns nonsense, a
CLI that crashes or contradicts its own `--help`, documentation that does not match reality - file
a bug report so the team sees it. Do it right after you notice the problem, then carry on with the
user's task.

```bash
cdxctl bug report \
    --title "kataster-sk search returns an empty list" \
    --description "Ran: kataster-sk-cli search --obec Bratislava. Expected a parcel list, got []. The same query works in the web UI, so the skill is not passing --obec through." \
    --severity HIGH
```

Severity is one of `LOW`, `MEDIUM`, `HIGH`, `CRITICAL` (case does not matter). Report when the
platform misbehaves; do not report user mistakes, missing input data, or a one-off network
timeout that succeeded on retry.

Nothing is collected automatically, so put everything a maintainer needs into `--description`:
what you ran, what you expected, what you got, and how to reproduce it. Reports are append-only -
there is no `cdxctl bug list`, `update` or `delete`, and filing the same bug twice creates two
records, so do not resend one you have already filed in this conversation.

## Output

- **Default:** JSON to stdout (machine-parseable)
- **Table:** Add `--table` flag for human-readable columns. A value longer than
  50 characters is truncated with an ellipsis, so take the full text from the
  JSON rather than from a table you are passing on.
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

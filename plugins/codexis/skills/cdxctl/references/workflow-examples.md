# Workflow examples

Worked `cdxctl workflow` runs. The script is a `.js` file in the workspace that exports `meta` and a
default async function; adapt one of the workflow templates the chat offers instead of writing one
from scratch.

## Validate, then run and wait

The usual sequence: check the script parses and shows the phases you expect, then start it with the
shell call blocked for the whole run (`timeoutSeconds: 5400`).

```bash
cdxctl workflow create --file /home/codexis/vypoved-reserse.js
```

```json
{
  "name": "legal-research",
  "description": "Právní rešerše: rámec → rekognoskace → podklady → rešerše → ověření → syntéza.",
  "phases": [
    { "title": "Rámec", "detail": "Kvalifikace, časový řez a sporné otázky" },
    { "title": "Rešerše", "detail": "Předpisy a judikatura ke sporným otázkám" },
    { "title": "Syntéza", "detail": "Stanovisko s odkazy na zdroje" }
  ]
}
```

```bash
cdxctl workflow start /home/codexis/vypoved-reserse.js \
  --args '{"question":"Je výpověď z organizačních důvodů platná?","jurisdiction":"CZ"}' \
  --wait --timeout-secs 5400
```

```
waiting for approval...
running
phase "Rámec" running
phase "Rámec" complete
phase "Rešerše" running
phase "Rešerše" complete
phase "Syntéza" running
phase "Syntéza" complete
complete
{"opinion":"Výpověď je neplatná, protože ...","sources":["§ 52 písm. c) zákoníku práce"]}
```

Exit code 0. The last line is the workflow result; use it as the answer's basis and tell the user
what came out, not that a CLI printed JSON.

## The user asks for changes

The proposal card has a "request changes" action. Taking it ends the run before any phase starts.

```bash
cdxctl workflow start /home/codexis/vypoved-reserse.js --wait --timeout-secs 5400
```

```
waiting for approval...
```

```
Error: user requested changes: Přidej fázi na kontrolu doručení výpovědi.
```

Exit code 7. Edit the script to match the feedback, run `cdxctl workflow create --file` again to
confirm the new phases, and start it again. Do not start the unchanged script a second time.

## Start now, check later

Without `--wait` the command returns as soon as the proposal is on screen, which is what you want
when the answer is not needed in this turn.

```bash
cdxctl workflow start /home/codexis/smlouva-rozbor.js --args '{"path":"/home/codexis/smlouva.pdf"}'
```

```json
{ "workflowRunId": "CFdvcmtmbG93JDkxYzcyN2Nl" }
```

```bash
cdxctl workflow status CFdvcmtmbG93JDkxYzcyN2Nl
```

```json
{
  "id": "CFdvcmtmbG93JDkxYzcyN2Nl",
  "status": "RUNNING",
  "phases": [
    { "title": "Rozklad", "status": "COMPLETE" },
    { "title": "Posouzení", "status": "RUNNING" },
    { "title": "Úpravy", "status": "PLANNED" }
  ],
  "feedback": null,
  "error": null,
  "resultJson": null
}
```

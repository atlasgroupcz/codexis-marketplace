---
uuid: 2b229bd1-c3ed-4aed-aea8-1a1d9a5b953c
name: onepost
icon: icon.svg
description: 'ONEPOST - datové schránky (ISDS) přes ONEPOST API. Use for ONEPOST / ISDS Czech data box work: list datové schránky, folders, messages, attachments, search messages, download attachments, upload attachments, create/update/send concepts, receive new ISDS messages, ZFO, datové zprávy. Triggers on "ONEPOST", "datová schránka", "datové schránky", "ISDS", "ZFO", "zprávy", "příloha", "koncept", "odeslat koncept", "vyhledej zprávu", "stáhni zprávy", "přijmi zprávy".'
version: 0.5.0
jurisdictions: [CZ]
i18n:
  cs:
    displayName: "ONEPOST - datové schránky"
    summary: "Práce s ONEPOST API - schránky, zprávy, přílohy, koncepty, odesílání a stažení zpráv z ISDS."
    examplePrompts:
      - "Ukaž mi nepřečtené zprávy v datové schránce."
      - "Najdi v datové schránce zprávy od finančního úřadu a stáhni přílohy."
      - "Připrav v datové schránce zprávu k odeslání."
  en:
    displayName: "ONEPOST - Data Boxes"
    summary: "Use the ONEPOST API for mailboxes, messages, attachments, concepts, sending, search, and receiving ISDS messages."
    examplePrompts:
      - "Show me the unread messages in my data box."
      - "Find data box messages from the tax office and download their attachments."
      - "Draft a data box message ready to send."
  sk:
    displayName: "ONEPOST - dátové schránky"
    summary: "Práca s ONEPOST API - schránky, správy, prílohy, koncepty, odosielanie a stiahnutie správ z ISDS."
    examplePrompts:
      - "Ukáž mi neprečítané správy v dátovej schránke."
      - "Nájdi v dátovej schránke správy z finančného úradu a stiahni prílohy."
      - "Priprav v dátovej schránke správu na odoslanie."
---

# ONEPOST

Use **`onepost-cli`** for all ONEPOST API access. Do not call `curl` directly.

The API key is not embedded in this plugin. It must be configured by the user:

Prefer the ONEPOST plugin app when the user wants a UI setup flow. The app opens
ONEPOST API key settings and stores the pasted key through the plugin CGI.

```bash
onepost-cli settings show
onepost-cli settings set "<ONEPOST_API_KEY>"
onepost-cli settings test
```

`settings set` validates the key against ONEPOST and stores it through the
platform (`cdxctl appenv onepost set ONEPOST_API_KEY <KEY>`); every command and
the app read it from `~/.cdx/apps/onepost/.env`. Do not write that file by hand
and do not call `cdxctl appenv` for the key directly - `settings set` does the
validation first.

Users generate keys in ONEPOST at `https://app.onepost.cz/profil/codexis-ai-agent`.
`READ` keys can use read commands. `READ_WRITE` keys are required for `receive`,
attachment upload, concept save, and concept send.

## Commands

```bash
onepost-cli mailboxes [--include-archive]
onepost-cli folders <MAILBOX_ID> [--parent-folder-id <FOLDER_ID>]
onepost-cli folder-messages <FOLDER_ID> [--offset 0] [--limit 20] [--order-dir desc]
onepost-cli message <MESSAGE_ID> [--no-attachments]
onepost-cli search "<QUERY>" [--mailbox-id <MAILBOX_ID>] [--folder-id <FOLDER_ID>] [--offset 0] [--limit 20]
onepost-cli attachments <MESSAGE_ID>
onepost-cli download-attachment <ATTACHMENT_ID> --output <PATH>
onepost-cli receive <MAILBOX_ID> [--only-headers]
onepost-cli upload-attachment <PATH> [--file-name NAME] [--mime-type MIME]
onepost-cli concept save --mailbox-id <MAILBOX_ID> [--mailbox-user-id <MAILBOX_USER_ID>] --subject "<SUBJECT>" --message-text "<TEXT>" --recipient <MAILBOX_ID[=NAME]> [--attachment <ATTACHMENT_ID_OR_msg_ID>]
onepost-cli concept save --body @concept.json
onepost-cli concept send <CONCEPT_ID>
onepost-cli api get <PATH> [--param KEY=VALUE]
onepost-cli api post <PATH> [--param KEY=VALUE] [--body '{"json":true}']
```

All normal API commands print JSON to stdout. Parse JSON structurally; do not use
`grep` or string slicing.

## Message Identifiers

A message carries two identifiers, and their shape tells them apart at a glance:

- `isdsMessageId` - the ISDS number, digits only, for example `12054584`. This is
  what the message is called in the data box, and the only identifier a user ever
  sees.
- `id` - the internal ONEPOST identifier, a UUID such as
  `9f1c8f0e-3b7a-4a52-9f4e-2f5b8c7d1a03`. It comes back from `search`,
  `folder-messages` and `message`. Users have no access to it.

All digits means ISDS, a UUID means internal. An identifier a user gives you is
almost always the ISDS one, so assume that and look it up - never ask the user
which kind it is.

When a command returns 404 for one form, fetch the message with
`onepost-cli message <IDENTIFIER>`, which accepts either form and returns both
`id` and `isdsMessageId`, then retry the command with the other form. Report a
message or an attachment as missing only after that.

## Attachments

`message` and `folder-messages` return `attachmentCount`. That field is the
answer to "does this message have attachments":

```bash
onepost-cli message "12054584"      # -> "attachmentCount": 2
onepost-cli attachments "12054584"  # -> the two attachments with their ids
```

When `attachmentCount` is greater than zero, list the attachments and download
what the user asked for in the same turn. Do not tell the user a message has no
attachment on any other basis, and do not wait for a second prompt to download.

## Error Handling

If `onepost-cli` prints `ERROR: ...`, stop and report the message. The client maps
ONEPOST HTTP responses to user-friendly errors:

- missing API key: tell the user to generate a key in ONEPOST and run `onepost-cli settings set`
- HTTP 401: invalid, deleted, or wrong API key
- HTTP 403: wrong key type (`READ` used for write), or missing permission to the resource
- HTTP 429: rate limit, try later
- HTTP 5xx or network error: ONEPOST is temporarily unavailable

Do not expose raw HTML or stack traces to the user.

## Typical Flows

List available mailboxes:

```bash
onepost-cli mailboxes --include-archive
```

Find folders in a mailbox, then list recent messages:

```bash
onepost-cli folders "<MAILBOX_ID>"
onepost-cli folder-messages "<FOLDER_ID>" --limit 10
```

Search messages:

```bash
onepost-cli search "smlouva" --limit 10
```

Download an attachment to a local file:

```bash
onepost-cli download-attachment "<ATTACHMENT_ID>" --output "/tmp/onepost-attachment.bin"
```

Receive new ISDS messages. This requires a `READ_WRITE` key:

```bash
onepost-cli receive "<MAILBOX_ID>" --only-headers
```

Create a concept and send it. This requires a `READ_WRITE` key and `MESSAGES_SEND`
permission in ONEPOST:

```bash
onepost-cli upload-attachment "/tmp/podani.pdf"
onepost-cli concept save \
  --mailbox-id "<SENDER_MAILBOX_ID>" \
  --subject "Podání" \
  --message-text "Dobrý den, zasíláme podání." \
  --recipient "<RECIPIENT_MAILBOX_ID>=Název příjemce" \
  --attachment "<ATTACHMENT_ID>"
onepost-cli concept send "<CONCEPT_ID>"
```

For full ISDS fields, prepare the backend JSON shape and pass it directly:

```bash
onepost-cli concept save --body @concept.json
```

The send response is synchronous JSON with `sentCount`, `errorCount`, and
per-recipient `results`. Treat `LOW_CREDIT` as terminal until the ONEPOST/ISDS
credit issue is resolved; do not blindly retry sends.

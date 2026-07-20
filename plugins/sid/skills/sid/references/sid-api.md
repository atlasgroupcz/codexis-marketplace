# SID API — response shapes and filters

Reference detail for `sid-cli`. The binary calls the endpoints itself; this
page exists so the output can be interpreted correctly and filters composed
without guessing.

## Command → endpoint

| Command | Endpoint |
|---------|----------|
| `search` | `GET /api/search` |
| `docs list` | `GET /api/v1/documents` |
| `docs get <id>` | `GET /api/v1/documents/{id}` |
| `docs legislation <id>` | `GET /api/v1/documents/{id}/legislation` |
| `docs set-validity <id>` | `PATCH /api/v1/documents/{id}` |
| `folders list` | `GET /api/v1/folders` |
| `attributes list` | `GET /api/v1/attributes` |
| `download <fileId>` | `GET /api/files/download/{fileId}` |
| `confirmations list` | `GET /api/v1/confirmations` |
| `notifications` | `GET /api/notifications` |
| `check` | notifications + confirmations in one call each |
| `settings test` | `GET /api/v1/users/me` |

Authentication is always the `x-api-key` header; the key is read from
`~/.cdx/apps/sid/.env` (mode 0600) and never passed as an argument.

The plugin currently targets the **onprem** instance
(`https://sid.onprem.agrp.dev`), declared as `CODEXIS_PLUGIN_SID_API_URL` in
the manifest. Production (`sid.codexis.cz`) does not have headless API-key
auth deployed yet; switching over is a one-line manifest change once it does.
`SID_API_URL` in the env file overrides it per user.

## Links back into SID

Every result naming a document carries a `url`. The route depends on the
signed-in role — SID's `(user)` layout redirects admins to the admin
dashboard, so admins get `/admin/files/<id>` and everyone else `/files/<id>`.
The role is cached as `SID_USER_ROLE` when the key is stored, so building the
link normally costs no extra request.

## `search`

```
sid-cli search "<query>" [--size N] [--from N]
```

Response:

```json
{
  "total": 3,
  "hits": [
    {
      "documentId": "…",
      "documentFileId": "…",
      "pageNumber": 4,
      "title": "Spisový řád",
      "filename": "spisovy-rad.pdf",
      "snippet": "…highlighted excerpt…",
      "score": 8.13
    }
  ],
  "source": "opensearch"
}
```

### Query syntax

The query goes to OpenSearch with Czech stemming, and supports Google-style
operators:

| Input | Meaning |
|-------|---------|
| `slovo` | one word, stemmed — also matches *slovy*, *slovem* |
| `slovo1 slovo2` | AND — both must appear |
| `"přesná fráze"` | exact phrase, adjacent, **not** stemmed |
| `slo*` | prefix |
| `-slovo` | exclude |
| `+slovo` | required |
| `a OR b` | explicit OR |
| `(a OR b) +c` | grouping |

Diacritics are folded, so `pravni` finds `právní`. Fields are weighted:
title ×3, description ×2, content ×1.

Stemming has limits — `smluvní` and `smlouva` are different roots. When a
search returns less than expected, retry with a prefix (`smlou*`) or an
explicit OR rather than concluding nothing exists.

Two traps:

- **`total` is not a total.** The route recomputes it to `hits.length` after
  the ACL filter, so it never exceeds `size`. It cannot drive pagination —
  use `docs list --page` for that.
- **`source: "opensearch_unavailable"` means the index is down**, not "nothing
  found" — HTTP is 200 either way. `sid-cli` adds a `warning` field in that
  case; take it seriously and fall back to `docs list --query`.

## `docs list`

```
sid-cli docs list [--query T] [--folder ID] [--author ID] [--attribute ID]…
                  [--from YYYY-MM-DD] [--to YYYY-MM-DD]
                  [--type pdf|word|excel|image] [--personal]
                  [--sort newest|oldest|title|size] [--page N] [--page-size N]
```

`--attribute` repeats and the API ANDs the values together. `--personal`
restricts the listing to documents with no group assigned. Ids for `--folder`
and `--attribute` come from `folders list` / `attributes list`.

`--page-size` is capped at 100 server-side. Response:

```json
{ "documents": [ … ], "total": 42, "page": 1, "pageSize": 20, "totalPages": 3 }
```

Here `total` **is** the real count — unlike `search`.

## `folders list` and `attributes list`

Both are ACL-scoped: the caller only sees folders they have access to
(ancestors included, so paths stay complete) and attributes reachable through
those folders.

`folders` returns a flat list with `parentId`; the CLI adds `path`
("Směrnice / Bezpečnost") and a `url` into the browser filtered to that
folder. `attributes` hides archived entries — they can no longer be assigned
— unless `--all` is given, and keeps `type` and `options` so a value can be
picked without guessing.

## `confirmations`

`--state pending` returns what the user has not confirmed yet; `confirmed`
returns the settled ones. Confirmation states are `UNSEEN` → `SEEN` →
`CONFIRMED`.

Confirming is deliberately **not** exposed by the CLI. It is a legal
acknowledgement, so it happens in SID with the document on screen rather than
through a chat message. The response carries `confirmInApp`, the URL of the
confirmations page.

## `notifications`

```
sid-cli notifications [--all] [--limit N]
```

Without `--all` only unread ones are returned. `--limit` is capped at 500
server-side.

## `check`

The digest the automation runs (`automations/check-sid.md`, daily at 07:00):

```json
{
  "unreadNotifications": 2,
  "pendingConfirmations": 1,
  "notifications": [ … ],
  "confirmations": [ … ],
  "notified": true
}
```

With `--notify` it also pushes to the daemon — but only when there is
something to report. `notified: false` means nothing was sent (no news, both
channels disabled, or the daemon was unreachable); it is never an error that
would fail the run.

Channels follow the same settings convention as the znamky/katastr plugins:

| File | Role |
|------|------|
| `~/.local/share/sid/settings/defaults.json` | shipped defaults, installed by the hook |
| `~/.cdx/apps/sid/settings.json` | user override, only the changed keys |

```json
{ "version": 1, "notify": { "email": false, "inApp": true, "recipients": [] } }
```

**E-mail is off by default** — a daily automation should not reach someone's
inbox unasked. Both toggles and the extra-recipient list are editable in the
plugin's settings panel; `recipients` are appended to the e-mail's `to` list,
and the signed-in user always receives it regardless.

## Limits

- search 60 requests/min per IP, download 120/min, upload 20/min
- 120 requests/min per key (independent of IP)
- Keys have a 90-day TTL; once expired the API returns 401 (`sid-cli` → exit 3)
- Every request times out after 30 s

## `docs legislation`

What the document tracks, as detected by the Codexis hypertexter or added by
hand. Each entry carries `title`, `reference` ("262/2006 Sb."), optional
`section` and `url`, a `type` (LAW, DECREE, …) and a `status` — `OK`, or a
flag that the source changed, with `changedAt`.

Requires `view` on the document, the same as reading it. Adding or approving
a change is not exposed: that belongs in the app.

## `docs set-validity`

```
sid-cli docs set-validity <id> [--from YYYY-MM-DD] [--to YYYY-MM-DD]
                               [--clear-from] [--clear-to]
```

The one write the CLI performs. Sends only the fields given, so setting the
start does not disturb the end. Needs EDITOR rights on the document — not
admin — and 403 means the caller lacks them.

Dates are validated before the call, so `1.1.2026` fails as misuse (exit 2)
rather than reaching the server. Changing an effective date makes SID notify
the people it concerns (`effectiveness_date_changed`).

## Versions and attachments

`docs get` returns `currentFile` (the current version), plus `fileCount`,
`attachmentCount` and `attachmentsPreview` (id + title). Attachments are
documents in their own right, so `docs get <attachmentId>` works on them.

Older versions are **not reachable**: SID exposes no endpoint listing a
document's files, so only the current one can be downloaded. Someone needing
an earlier version has to open the document in SID.

## Out of scope

These are not reachable over SID's HTTP API at all (they exist only as Next.js
server actions), so `sid-cli` does not expose them and will not grow them
without a server-side change:

- folder permission management,
- group writes,
- access introspection ("what can this user see"),
- **changing** what legislation a document tracks, and the Hypertexter
  detection run. Reading it is available — see `docs legislation` above.

Document upload *is* supported by the API but not wrapped by `sid-cli` — use
the SID web interface for that.

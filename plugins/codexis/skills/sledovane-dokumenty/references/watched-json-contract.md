# `watched.json` - integration contract

When the user watches a folder of a project, `cdx-sledovane-dokumenty` keeps
the watch's `watched.json` in the app's state, **not inside the project's
files**: a chat sees those files read-only, and a hidden file there would show
up in the project's file explorer. The file is a **stable, openly documented contract**:
deterministic scripts, automations and external tools MUST treat it as the
integration boundary. Do not parse the app's other private state instead.

## Location

```
~/.cdx/app-state/sledovane-dokumenty/
  watched-projects.json                 # index: one entry per watched folder
  watched/<watchId>/
    watched.json                        # THIS contract (harvest output)
    tracking-state.json                 # runner bookkeeping (not part of the public contract)
```

`<watchId>` is the `uuid` of the index entry; the entry also carries the
project's UUID (the `id` segment of the daemon's `Project` node id), its name and
the watched folder. The same pair is repeated inside `watched.json`, so the file
stands on its own.

## Schema

```json
{
  "version": 2,
  "projectId": "5b1c1c4e-0000-4000-8000-000000000001",
  "folder": "smlouvy/2026",
  "generatedAt": "2026-06-01T12:00:00Z",
  "documents": [
    {
      "path": "./dogs/buy-dog.docx",
      "sha256": "<document-sha-256>",
      "discoveredAt": "2026-06-01T12:00:00Z",
      "extractedAt": "2026-06-01T12:01:00Z",
      "updatedAt": "2026-06-01T12:01:00Z",
      "legislation": [
        {
          "uri": "https://www.zakonyprolidi.cz/cs/2012-89",
          "text": "Zákon č. 89/2012 Sb., občanský zákoník - § 2079 a násl.",
          "codexisId": "CR26785"
        }
      ]
    }
  ]
}
```

### Top level

| Field | Type | Meaning |
|---|---|---|
| `version` | int | Contract version. Currently `2`. |
| `projectId` | string | UUID of the project the folder belongs to. |
| `folder` | string | The watched folder relative to the project's files, POSIX form without a leading slash; `.` is the project root. |
| `generatedAt` | string | ISO-8601 UTC `Z` of the last write. |
| `documents` | array | One entry per tracked document. |

### `documents[]`

| Field | Type | Meaning |
|---|---|---|
| `path` | string | POSIX path **relative to the watched folder**, prefixed `./`. Example: with `folder` `smlouvy/2026`, a file at `<files>/smlouvy/2026/dogs/buy-dog.docx` is `./dogs/buy-dog.docx`. Inside a chat of the project the same file is `/home/codexis/project/files/smlouvy/2026/dogs/buy-dog.docx`. |
| `sha256` | string | SHA-256 of the raw file bytes. The **only** re-harvest trigger besides an explicit refresh. |
| `discoveredAt` | string | ISO-8601 UTC. When the file was first seen. |
| `extractedAt` | string \| null | ISO-8601 UTC of the last successful AI harvest. `null` means harvest is pending (new file or changed checksum). |
| `updatedAt` | string | ISO-8601 UTC of the last change to this entry. |
| `legislation` | array | Referenced legislation harvested by AI. May be empty. |

### `legislation[]`

| Field | Type | Meaning |
|---|---|---|
| `uri` | string | Canonical, public legislation URI (for Czech acts: `https://www.zakonyprolidi.cz/cs/<year>-<number>`). Suitable for deterministic change tracking. May be empty if unknown. |
| `text` | string | Human-friendly title, including the relevant part where applicable. |
| `codexisId` | string | *Optional.* Base CODEXIS document id (e.g. `CR26785`) when the reference resolved to a Czech act. Present only when resolvable; used by the deterministic runner to detect version changes. |

## Guarantees & rules

- **Paths are always relative to the watched folder**, so the same entry is
  valid wherever the daemon shows the project's files.
- **AI is used once per document** to fill `legislation`, keyed by `path` + `sha256`.
  References are re-harvested only when `sha256` changes or the user explicitly
  refreshes (`cdx-sledovane-dokumenty project refresh <project> --folder <path>`).
- **Change detection is deterministic and AI-free.** The runner
  (`cdx-sledovane-dokumenty project check`) resolves each `codexisId` to its
  current CODEXIS version id and compares it against the baseline stored in
  `tracking-state.json`. No model is invoked.
- The `legislation[].codexisId` field is an **optional extension**: consumers that
  only need to display references can rely on `uri` + `text` alone.
- Writes are atomic (temp file + rename); a reader never sees a partial file.

## `tracking-state.json` (informative, not the contract)

Runner bookkeeping, keyed by `codexisId`: `baselineVersionId`,
`lastKnownVersionId`, `lastCheckAt`, and a `changes[]` log
(`detected_on`, `effective_on`, `old_version_id`, `new_version_id`,
`amendments`, `compare_url`, `confirmed_on`). External tools should not depend on
its shape - read `watched.json` for the list of referenced legislation.

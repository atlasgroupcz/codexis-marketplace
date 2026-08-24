---
name: datify
description: "This skill should be invoked whenever the user needs to read or change data in Datify - listing collections and datasets, reading or paging through records, filtering records by property values, creating or updating records, uploading attachments, listing or downloading a record's attachments, or configuring the Datify API token. Trigger keywords: Datify, datify.cz, kolekce, dataset, datasety, záznam, záznamy, records, property, vlastnost, filtrovat záznamy, přidat záznam, upravit záznam, smazat záznam, příloha do Datify, přílohy záznamu, stáhnout přílohu, otevřít přílohu, co je v příloze, Datify token, Datify API."
version: 1.1.0
icon: icon.svg
i18n:
  cs:
    displayName: "Datify - digitální AI kancelář"
    summary: "Čtení a úpravy kolekcí, datasetů a záznamů v Datify."
    examplePrompts:
      - "Ukaž mi přehled kolekcí a datasetů, které mám v Datify."
      - "Vyfiltruj v Datify záznamy podle hodnoty vlastnosti a vypiš je do tabulky."
      - "Nahraj tuhle smlouvu do Datify a nech z ní vytáhnout údaje do záznamu."
  en:
    displayName: "Datify - Digital AI Office"
    summary: "Read and edit Datify collections, datasets and records."
    examplePrompts:
      - "Show me an overview of the collections and datasets I have in Datify."
      - "Filter Datify records by a property value and list them in a table."
      - "Upload this contract to Datify and have its values extracted into a record."
  sk:
    displayName: "Datify - digitálna AI kancelária"
    summary: "Čítanie a úpravy kolekcií, datasetov a záznamov v Datify."
    examplePrompts:
      - "Ukáž mi prehľad kolekcií a datasetov, ktoré mám v Datify."
      - "Vyfiltruj v Datify záznamy podľa hodnoty vlastnosti a vypíš ich do tabuľky."
      - "Nahraj túto zmluvu do Datify a nechaj z nej vytiahnuť údaje do záznamu."
---

# Datify

`datify-cli` wraps the Datify public REST API (v0). Every command prints
Datify's JSON unchanged.

## Setup

The user generates an API token in Datify and saves it on the plugin's settings
page. Without it every command fails with a message naming that page. To check:

```bash
datify-cli settings show      # configured, masked token, where it came from
datify-cli settings test      # confirm Datify still accepts it
```

## Two things that decide how you use this API

**1. Record values are keyed by property UUID, not by name.** A record comes
back as `{"id": ..., "values": {"<property-uuid>": <value>}}`. To read or write
anything by name you first need the dataset's property list:

```bash
datify-cli dataset <datasetId>     # returns properties: id, name, type, config
```

So the normal flow is **dataset first, records second**. Cache that mapping for
the rest of the conversation instead of re-fetching it per record.

**2. Filters are flat, and always a group.** A filter is `match` + `conditions`,
even for a single condition. Nested groups are not supported, so do not try to
build `(A and B) or C`; run the parts separately and combine the results
yourself.

## Finding data

```bash
datify-cli collections                          # every collection + its datasets
datify-cli dataset <datasetId>                  # properties (the uuid → name map)
datify-cli records list <datasetId> --limit 25  # first page
datify-cli records list <datasetId> --cursor <cursor>   # next page
```

Paging: use the `cursor` from the previous response. A null or absent `cursor`
means there are no more pages. `--sort` accepts `createdAt,asc` or
`createdAt,desc` only.

## Filtering

`--filter` takes JSON, or `-` to read it from stdin. In a condition `property`
may be the property's name or its uuid.

```bash
datify-cli records query <datasetId> --filter '{
  "filter": {
    "match": "and",
    "conditions": [
      {"property": "Title",  "text":   {"contains": "Quarterly"}},
      {"property": "Amount", "number": {"greater_than": 1000}}
    ]
  }
}'
```

One condition uses the same shape, with a single-item `conditions` list:

```json
{"filter": {"match": "and", "conditions": [
  {"property": "Title", "text": {"contains": "Quarterly"}}
]}}
```

The condition key matches the property type, and each type takes its own
operators. Picking the wrong one is the usual reason a filter returns nothing:

| Key | Operators |
| --- | --- |
| `text` | `contains`, `does_not_contain`, `equals`, `not_equals`, `is_empty`, `is_not_empty` |
| `number` | `greater_than`, `greater_than_or_equal`, `less_than`, `less_than_or_equal`, `equals`, `not_equals`, `is_empty`, `is_not_empty` |
| `date` | `equals`, `not_equals`, `before`, `after`, `is_empty`, `is_not_empty` |
| `select` | `is`, `is_not`, `is_any_of`, `is_none_of`, `is_empty`, `is_not_empty` |
| `relation` | `is`, `is_not`, `is_any_of`, `is_none_of`, `is_empty`, `is_not_empty` |
| `user` | `is`, `is_not`, `is_any_of`, `is_none_of`, `is_empty`, `is_not_empty` |
| `checkbox` | `equals` |

Note `select`, `relation` and `user` match with `is`, not `equals` or
`contains`. `select.is` takes an option id or option name; `relation.is` and
`user.is` take a record or user uuid.

**On a multi_select property use `is_any_of`, even for one option.** There
`select.is` means "this is the only option selected", so it silently drops every
record that also has another one - by id or by name. `is_any_of` with the same
single option matches them all. `is` works as written on single_select.

**A `select`, `relation` or `user` condition given a uuid that resolves to
nothing is silently dropped**, and the query answers as if the condition had
never been written - every record in the dataset comes back. It applies to `is`,
`is_not`, `is_any_of` and `is_none_of` alike, with HTTP 200 and no warning.

Only uuids slip through - a *name* that matches nothing is rejected with
`HTTP 400 option/value not found` - so filter by name wherever one exists:

```json
{"property": "Žánr", "select": {"is_any_of": ["Drama"]}}
```

When it has to be a uuid, take it from data you have just read (a dataset's
`config.options`, a record's own values) rather than from the conversation,
check the result count against the unfiltered dataset before acting on it, and
never let such a filter alone decide what gets deleted.

`is_empty` and `is_not_empty` take `true`, not an empty object:

```json
{"property": "Poznámka", "text": {"is_empty": true}}
```

`{}` is rejected with HTTP 400.

Date values are `YYYY-MM-DD`, or one of Datify's relative tokens (`today`,
`yesterday`, `week ago`, `month from now`, `5 days ago`, ...). A full timestamp
is rejected.

`records query` takes `--limit` 1 to 50. `records list` has no such cap.

Relation and rollup values can be longer than a record payload carries, so they
come back paginated. Read one in full with:

```bash
datify-cli property <recordId> <propertyId> --limit 50
```

Use it for relation and rollup properties. On an attachment property Datify
answers HTTP 500 - list the record's files with
`datify-cli records attachments <recordId>` instead.

## Writing

Bodies take JSON or `-` for stdin, and are keyed by **property uuid**. The
wrapper key is `values` - not `properties`, which Datify rejects.

```bash
# one record
datify-cli records create <datasetId> --data '{"values": {"<uuid>": "Q3 report"}}'

# merge into an existing record (PATCH)
datify-cli records update <recordId> --data '{"values": {"<uuid>": 42}}'

# replace the whole record (PUT) - properties you omit are cleared
datify-cli records update <recordId> --data '{"values": {...}}' --replace
```

`records create` answers with an **array**, one entry per record created, even
for a single one - so the new id is `.[0].id`, not `.id`.

**To clear one property, PATCH it to `null`.** Omitting it on a PATCH changes
nothing, and `--replace` would wipe every property you did not resend - so `null`
is the only way to empty one value without risking the rest of the record.

```bash
datify-cli records update <recordId> --data '{"values": {"<uuid>": null}}'
```

**A `percent` value is a whole number: `21` means 21 %.** Writing `0.21` stores
0.21 %. Datify keeps the number exactly as sent and never rescales it.

`single_select` and `multi_select` take an option **name** as well as its id, and
store the id either way - so a write can use the same names a filter does.

### A record you read cannot be written back as it came

Two value types differ between reading and writing, so a body built by reading a
record and changing one field is rejected whole:

- **Computed properties cannot be written.** `formula` and `rollup` are refused
  with `HTTP 422 Invalid value for property of type ...`, even when the value is
  the one Datify itself just returned. `attachment` too - files are managed by
  `upload`, not by record values.
- **`relation` reads as `{"items": [...], "cursor": ...}` but writes as a bare
  array of ids.** Sending back the object fails with
  `HTTP 422 Invalid relation value`.

So drop the computed properties and unwrap every relation before writing:

```bash
# read, keep only what is writable, put it back
datify-cli records get <recordId> | jq '{values: (.values
  | del(.["<formula-uuid>"], .["<rollup-uuid>"])
  | map_values(if type == "object" and has("items") then .items else . end))}' \
  | datify-cli records update <recordId> --data - --replace
```

This bites hardest on `--replace`, which needs the whole record in the body.
Prefer a PATCH carrying only the properties you actually change.

Bulk create takes `records` instead, and its items are **bare uuid → value
maps** - do not wrap each one in `values`:

```bash
datify-cli records create <datasetId> --data '{"records": [
  {"<uuid>": "první"},
  {"<uuid>": "druhá"}
]}'
```

Exactly one of `values` or `records` per create call; sending both or neither is
rejected. Bulk create is capped at ten - eleven fails without creating any.

To leave a property empty, **omit it**. `""` is rejected as an invalid value for
a text property, and in a bulk call that rejection takes the whole batch down
with it.

## Deleting

Deletion is **irreversible and Datify offers no undo**. Confirm the exact
records with the user before running either command, and prefer listing what
you are about to delete first.

```bash
datify-cli records delete <recordId>
datify-cli records delete-many <recordId> <recordId> ...
```

## Attachments

A record does not carry its files. One call lists everything on a record, across
every attachment property it owns:

```bash
datify-cli records attachments <recordId>                # every file on a record
datify-cli attachment <attachmentId>                     # metadata for one
datify-cli download <attachmentId> --output-dir <dir>    # the file itself
```

`records attachments` returns `{"attachments": [...]}`, each entry carrying
`filename`, `contentType`, `size`, `url` and the `propertyId` it hangs under -
the record's own values cannot tell those apart, being bare id arrays per
property.

Attachment properties cannot be filtered on - there is no `attachment` condition
key, and using one fails with `a condition must specify exactly one type-block`.
To find the records that have a file, read them and check the property.

### Reading a file

`download` saves into the directory you name, creating it when missing, and
prints where the file landed:

```bash
datify-cli download <attachmentId> --output-dir /home/codexis/prilohy
# → {"path": "/home/codexis/prilohy/smlouva.pdf", "filename": "smlouva.pdf",
#    "contentType": "application/pdf", "size": 1718462}
```

**Give it a directory, never a file path.** The name comes from Datify, so the
extension matches what the file really is - and `ocr` picks its reader from that
extension. To rename afterwards, use `mv` on the `path` it returned.

Pass an absolute directory; a relative one resolves against whatever directory
the shell happens to be in, which moves during a conversation.

**To answer a question about a file's contents**, download it and read it with
`ocr`, which converts PDF and images to Markdown:

```bash
path=$(datify-cli download <attachmentId> --output-dir /home/codexis | jq -r .path)
ocr "$path"
```

A name already taken in the target directory is not overwritten - the second
file becomes `smlouva (2).pdf`. Two different attachments do share a name in
practice.

### Uploading

The file is read from the VM's own filesystem, so pass an absolute path - a
relative one resolves against whatever directory the shell happens to be in.

```bash
datify-cli upload <path> --record <recordId> --property <propertyId>  # onto a record
datify-cli upload <path> --dataset <datasetId>                        # dataset pool
datify-cli upload <path>                                              # unattached

datify-cli upload /home/codexis/smlouva.pdf --dataset <datasetId> --ai
```

The upload declares the file's media type from its extension, and Datify
reports that back on the attachment.

`--ai` asks Datify to run extraction over the uploaded file (dataset and
unattached uploads only).

**A dataset upload settles tens of seconds after the response returns.** The
reply comes back with `recordId` and `url` both `null`; Datify then creates the
record and, with `--ai`, fills the extracted values in. Twenty seconds is
typical. Poll `datify-cli attachment <attachmentId>` for a minute before
concluding anything, then read the record for what the extraction wrote.

An upload Datify cannot place instead lands in the **Automatic file
classification** queue on the app's home page and waits for a person to confirm
its collection and dataset - `--ai` proposes them, it does not decide them.
Until someone confirms, `recordId` stays `null` and no record exists.

The two are indistinguishable through the API, so give it the minute first. Only
then tell the user to confirm the file in Datify. Prefer `--record` whenever the
upload must complete without anyone opening the app.

**The API has no attachment delete**, and no way to unlink one either:
`attachment` is not among the value types a write accepts, so PATCH and PUT
cannot empty that list of ids. An attachment goes away with the record that
holds it, so deleting the record is the only route - for a dataset upload that
means the record Datify created for it, whose id you have to fetch as above. An
unattached upload never gets one, so nothing in this API can remove it; tell the
user it has to be deleted in the Datify app, and prefer one of the other two
targets when the file is meant to be temporary.

## Users

```bash
datify-cli user <userId>      # name, e-mail
```

There is no endpoint that lists users, so the id has to come from data you have
already read: a `user` property on a record holds the assigned user ids. Resolve
a name from there rather than asking the user for a uuid they have no way to
look up.

## When something fails

The exit code is non-zero and stderr names the cause. The ones worth acting on:

| Message | What to do |
| --- | --- |
| API token není nastaven | Send the user to the settings page |
| odmítlo API token (401) | Token is wrong or revoked - the user regenerates it |
| nepovolilo akci (403) | Rights **or** a wrong id - see below |
| nenašlo objekt (404) | Check the id - list the parent to find the right one |
| odmítlo data (422) | The message names the offending property **type**, not which property it was - match it against the types in the body you sent |
| rate limitu (429) | Wait and retry |
| API vrátilo chybu (500) | Usually a bad cursor - drop it and re-read the first page |

**403 does not mean what it says.** Datify answers 403 for an id that does not
exist as well as for a real permission gap, so check the id first: list the
parent object and confirm it is there before telling the user about rights.

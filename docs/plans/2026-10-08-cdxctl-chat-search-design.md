# cdxctl chat search

## Accepted scope

Expose the daemon's existing chat search to the assistant through the marketplace's
`cdxctl` command. The CLI passes the query, optional project filter and pagination
to the existing GraphQL operation and returns its results. The search backend owns
matching, indexed content and ranking, including its use of chat summaries.

This version adds only `cdxctl chat search`. Chat retrieval, conversation retrieval,
summary generation and date filters are outside this version. A future date filter
must be supported by the backend before the CLI exposes it.

## CLI contract

```text
cdxctl chat search <query> [--project <name-or-id>] [--limit <count>] [--offset <count>] [--table]
```

Examples:

```bash
cdxctl chat search "výpověď nájmu"
cdxctl chat search "výpověď nájmu" --project "Právní agenda"
cdxctl chat search "výpověď nájmu" --project "Právní agenda" --limit 20 --offset 20
```

The query is required and must contain non-whitespace text. Pass it to GraphQL
without adding operators, expanding terms or selecting specific indexed fields.

Omitting `--project` searches all chats available to the authenticated user through
the existing search operation. Do not implicitly restrict the query to the active
chat's project. An explicit project accepts its name, raw UUID or GraphQL Node ID,
consistent with existing project commands. Resolve it using the existing project
listing and Node ID infrastructure. A missing project or ambiguous name is an error;
never silently omit the filter or choose the first ambiguous match.

The default limit is 20 and the default offset is 0. Accept a limit from 1 to 100
and a nonnegative offset. Pass pagination to the backend; do not fetch all results,
reorder them, filter them locally or automatically request additional pages.
The backend's pagination bounds remain authoritative.

## Results

Default output is a JSON array using the existing output infrastructure. Select
these fields for each `ChatSearchHit`:

- `id`
- `title`
- `modifiedAt`
- `snippet`
- `automation`
- `project { id name }`

Preserve the backend's result order, IDs, snippets and nullable values. Do not turn
the snippet into a generated summary or promise that it came from a particular
indexed file. An empty result is `[]`. The existing API supplies neither a total
count nor a next-page indicator; do not invent them.

Support the global `--table` option with a readable project name and the search
result's scalar fields. Keep display formatting separate from the JSON data.

## Data flow and implementation

```text
cdxctl chat search
  -> resolve an explicit project, when supplied
  -> existing GraphQLClient
  -> Query.search(query, projectId, offset, limit)
  -> existing daemon search service
  -> existing search backend
```

Add the `Chat` command group with its single `Search` subcommand to
`plugins/codexis/src/cdxctl/src/main.rs`. Add command handling in
`plugins/codexis/src/cdxctl/src/commands/chat.rs` and register it in
`commands/mod.rs`. Store the operation in `queries/chat_search.graphql` and expose
it through `graphql.rs`, following the current command layout.

Reuse `GraphQLClient` and its existing daemon URL and authentication. Do not connect
directly to OpenSearch or introduce credentials, a second search API or filesystem
scanning. Keep project resolution shared where practical without changing the
behavior of unrelated commands.

Update `plugins/codexis/skills/cdxctl/SKILL.md` so its discovery metadata covers
finding previous chats and its instructions show the new command, project filter,
pagination and output. Explain that search semantics are supplied by the backend.
Do not advertise chat retrieval commands or date filters.

The user explicitly scoped this work to exposing the existing search through
marketplace `cdxctl`. The daemon's search behavior and GraphQL schema remain the
existing contract. Changes to UI, public REST v2, `codexis-agent-cli` and MCP are
outside this delivery; existing differences between those interfaces are not a
parity backfill task.

## Authorization and errors

Use the authenticated user's existing project and search operations. Do not accept
a user identity override or derive a substitute identity.

Reject invalid CLI arguments before executing the search. Preserve the existing
nonzero error handling for project lookup, network and GraphQL errors. An invalid
or missing search response must not be reported as a successful empty result.
When the backend returns an empty list, return it as supplied; the current API does
not expose a separate search-availability state.

## Verification for implementation

Cover the CLI argument contract and exercise the command against the existing
loopback GraphQL test server. Verify the query and pagination forwarded to the
server, omitted and explicit project filters, all supported project identifier
forms, ambiguous or unknown names, empty results, null result fields and error
propagation. Preserve the supplied ordering and snippets in JSON output. Verify
that table output displays the project name.

Run `cargo test -p cdxctl`, the repository's applicable Rust formatting checks and
the skill conformance checks. Use the actual daemon for a smoke test with known
chats in two projects: a global query and the same query restricted to one project
must reflect the existing search behavior. A test term present only in an indexed
summary can demonstrate use of that backend capability without making it CLI logic.

Record command output and distinguish mocked GraphQL checks from an actual search
smoke test. Rebuild the distributed CLI through the repository's normal binary
build workflow when delivering the implementation.

## Repository evidence

The design was checked against marketplace revision `b61ddb05` and daemon revision
`59cd9e1f6f` on 2026-10-08. The relevant daemon contract is in
`cdx-daemon-app/src/main/resources/graphql/chat.graphqls`; its implementation starts
at `search/api/ChatSearchQueryController.java`. The marketplace already has
`GraphQLClient`, Node ID helpers, project lookup and JSON/table output support.

# Global Search (ALL)

Search across all data sources simultaneously. Use this only for orientation when the relevant source is unclear; do not treat it as the final authority for citations or extraction.

## cdx Usage
Use `cdx` for requests. It is opinionated: it runs silently by default, and `-d` implies `POST` plus `Content-Type: application/json` unless you override them.

```bash
cdx "cdx://search/ALL" \
  -d '{"query": "insolvence", "limit": 5}'
```

## Endpoint

```
POST cdx://search/ALL
JSON request body
```

## Request Schema

```json
{
  "query": "string (required) - fulltext search",
  "limit": "integer (1-50, default: 10)",
  "offset": "integer (default: 0)"
}
```

**Note:** Global search has fewer filtering options than source-specific searches and should be followed by a source-specific search before final use.

## Response Schema

```json
{
  "results": [
    {
      "docId": "CR26785_2026_01_01",
      "source": "CR",
      "docUrl": "cdx://doc/CR26785_2026_01_01/text",
      "title": "89/2012 Sb. Zákon občanský zákoník",
      "snippet": "text with <mark>highlights</mark>"
    },
    {
      "docId": "JD252461",
      "source": "JD",
      "docUrl": "cdx://doc/JD252461/text",
      "title": "Nález - Ke stanovení výše náhrady škody...",
      "snippet": "text with <mark>highlights</mark>"
    }
  ],
  "totalResults": 387122,
  "offset": 0,
  "limit": 10
}
```

## Key Fields

| Field | Description |
|-------|-------------|
| `docUrl` | Source-specific document URL |
| `source` | Data source (CR, JD, EU, SK, ES, LT, VS, COMMENT) |

`docUrl` may contain source-specific query hints (for example `?part=...`).  
For deterministic workflows, use `docId` and call explicit endpoints (`/meta`, `/text`, `/toc`, `/related`) as needed.

## Important Limitations

- `ALL` is exploratory, not authoritative resolution.
- Results may include base legislation IDs instead of version IDs.
- Results may surface mixed identifier families such as `LIBERIS...` commentary IDs or `BOOKS...` records.
- `BOOKS...` records may appear in `ALL` even though there is no documented dedicated `search/BOOKS` endpoint and the record may not be directly retrievable via `/doc/...`.
- After using `ALL`, rerun the query in the relevant source (`CR`, `EU`, `JD`, `ES`, `COMMENT`, `LT`, `VS`) before citing, extracting, or linking for the user.

## Examples

### Search All Sources

```bash
cdx "cdx://search/ALL" \
  -d '{
    "query": "odpovědnost za škodu",
    "limit": 20
  }' | jq '.results[] | {source, docId, title}'
```

### Group Results by Source

```bash
cdx "cdx://search/ALL" \
  -d '{
    "query": "ochrana osobních údajů",
    "limit": 50
  }' | jq '.results | group_by(.source) | map({source: .[0].source, count: length})'
```

### Filter Results by Source (Client-Side)

```bash
cdx "cdx://search/ALL" \
  -d '{
    "query": "GDPR",
    "limit": 50
  }' | jq '.results | map(select(.source == "EU" or .source == "ES"))'
```

### Get Top Result from Each Source

```bash
cdx "cdx://search/ALL" \
  -d '{
    "query": "smlouva o dílo",
    "limit": 50
  }' | jq '.results | group_by(.source) | map(.[0])'
```

## When to Use Global Search

**Use ALL search when:**
- Exploring a topic across multiple domains
- Don't know which source is most relevant
- Need to see the distribution across sources
- Doing an initial reconnaissance pass before switching to a specific source

**Use source-specific search when:**
- Know the target source (e.g., only need laws)
- Need advanced filtering (validity dates, facets)
- Need comprehensive results from one source
- Performance is critical (ALL is slower)
- You need a stable document ID for linking or extraction

## Workflow: Topic Research

1. If the source is unknown, start with global search to understand the landscape:
```bash
cdx "cdx://search/ALL" \
  -d '{"query": "insolvence", "limit": 50}' | \
  jq '.results | group_by(.source) | map({source: .[0].source, count: length})'
```

2. Identify the most relevant source or sources from the distribution.

3. Rerun the query in the relevant source and resolve the final document there:
```bash
# If legislation is most relevant
cdx "cdx://search/CR" \
  -d '{"query": "insolvence", "validNow": true, "limit": 20}'
```

4. Only after source-specific resolution should you fetch `/meta`, `/text`, `/toc`, or build user-facing `cdx://doc/...` links.

## Combining with Relations

After finding a promising document family via global search, resolve the final document in its source first, then explore its relations:

```bash
# Get relation counts
DOC_ID="CR26785"
cdx "cdx://doc/${DOC_ID}/related/counts" | jq '.'

# Get specific relations
cdx "cdx://doc/${DOC_ID}/related?type=SOUVISEJICI_JUDIKATURA&limit=10"
```

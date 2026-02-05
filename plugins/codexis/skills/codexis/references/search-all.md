# Global Search (ALL)

Search across all data sources simultaneously.

## cdx Usage
Use `cdx` for requests. It accepts standard curl flags and `cdx://` URLs.

```bash
cdx -s -X POST "cdx://search/ALL" \
  -H 'Content-Type: application/json' \
  -d '{"query": "insolvence", "limit": 5}'
```

## Endpoint

```
POST cdx://search/ALL
Content-Type: application/json
```

## Request Schema

```json
{
  "query": "string (required) - fulltext search",
  "limit": "integer (1-50, default: 10)",
  "offset": "integer (default: 0)"
}
```

**Note:** Global search has fewer filtering options than source-specific searches.

## Response Schema

```json
{
  "results": [
    {
      "docId": "CR26785",
      "source": "CR",
      "title": "89/2012 Sb. Zákon občanský zákoník",
      "snippet": "text with <mark>highlights</mark>",
      "score": 0.95
    },
    {
      "docId": "JD252461",
      "source": "JD",
      "title": "Nález - Ke stanovení výše náhrady škody...",
      "snippet": "text with <mark>highlights</mark>",
      "score": 0.87
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
| `source` | Data source (CR, JD, EU, SK, ES, LT, VS, COMMENT) |
| `score` | Relevance score |

## Examples

### Search All Sources

```bash
cdx -s -X POST "cdx://search/ALL" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "odpovědnost za škodu",
    "limit": 20
  }' | jq '.results[] | {source, docId, title}'
```

### Group Results by Source

```bash
cdx -s -X POST "cdx://search/ALL" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "ochrana osobních údajů",
    "limit": 50
  }' | jq '.results | group_by(.source) | map({source: .[0].source, count: length})'
```

### Filter Results by Source (Client-Side)

```bash
cdx -s -X POST "cdx://search/ALL" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "GDPR",
    "limit": 50
  }' | jq '.results | map(select(.source == "EU" or .source == "ES"))'
```

### Get Top Result from Each Source

```bash
cdx -s -X POST "cdx://search/ALL" \
  -H 'Content-Type: application/json' \
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
- Initial research phase

**Use source-specific search when:**
- Know the target source (e.g., only need laws)
- Need advanced filtering (validity dates, facets)
- Need comprehensive results from one source
- Performance is critical (ALL is slower)

## Workflow: Topic Research

1. Start with global search to understand the landscape:
```bash
cdx -s -X POST "cdx://search/ALL" \
  -H 'Content-Type: application/json' \
  -d '{"query": "insolvence", "limit": 50}' | \
  jq '.results | group_by(.source) | map({source: .[0].source, count: length})'
```

2. Identify most relevant sources from distribution

3. Drill down with source-specific searches:
```bash
# If legislation is most relevant
cdx -s -X POST "cdx://search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "insolvence", "validNow": true, "limit": 20}'
```

## Combining with Relations

After finding a key document via global search, explore its relations:

```bash
# Get relation counts
DOC_ID="CR26785"
cdx -s "cdx://doc/${DOC_ID}/related/counts" | jq '.'

# Get specific relations
cdx -s "cdx://doc/${DOC_ID}/related?type=SOUVISEJICI_JUDIKATURA&limit=10"
```

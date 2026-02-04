---
name: CODEXIS
description: This skill should be invoked whenever user needs law or legal information, czech or european. Provides guidance for querying the CODEXIS legal database API.
version: 1.0.0
---

# CODEXIS Legal Database API

CODEXIS is a comprehensive Czech legal database providing structured access to legislation, case law, EU law, legal literature, and contract templates via a REST API.

## API Configuration

The API base URL is configured via environment variable:

```bash
export CODEXIS_API_URL="https://your-codexis-instance.example.com"
```

All endpoints are relative to `${CODEXIS_API_URL}/rest/cdx-api/`.

## Data Sources

| Code | Name | Description | Has TOC | Has Versions |
|------|------|-------------|---------|--------------|
| `CR` | Czech Legislation | Laws, decrees, regulations, municipal documents | Yes | Yes |
| `SK` | Slovak Legislation | Slovak legal acts | Yes | Yes |
| `JD` | Czech Case Law | Judicial decisions from Czech courts | No | No |
| `ES` | EU Court Decisions | EU Court of Justice rulings | No | No |
| `EU` | EU Legislation | EU regulations and directives | Yes | Yes |
| `LT` | Legal Literature | Legal publications and articles | No | No |
| `VS` | Contract Templates | Contract specimens and templates | No | No |
| `COMMENT` | Legal Commentaries | LIBERIS legal commentary | No | No |
| `ALL` | Global Search | Search across all sources | - | - |

## Core API Operations

### Search Documents

All search endpoints use POST with JSON body:

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/{SOURCE}" \
  -H 'Content-Type: application/json' \
  -d '{"query": "search terms", "limit": 10}'
```

**Common query features:**
- Fulltext with space-separated terms
- Wildcards: `smlouv*` matches smlouva, smlouvy, etc.
- Phrases: `"nájem bytu"` for exact match
- Write Czech characters directly (no Unicode escapes)

### Document Retrieval

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/doc/{docId}/meta` | GET | Document metadata |
| `/doc/{docId}/text` | GET | Full document text with anchors |
| `/doc/{docId}/toc` | GET | Table of contents (CR, SK, EU only) |
| `/doc/{docId}/versions` | GET | Time versions (CR, SK, EU only) |
| `/doc/{docId}/related` | GET | Related documents |
| `/doc/{docId}/related/counts` | GET | Relation type counts |

### Document IDs

Documents use composite IDs with optional version suffix:
- Base ID: `CR26785` (Civil Code)
- Version ID: `CR26785_2026_01_01` (specific time version)

## Working with Results

### Extract Specific Fields with jq

```bash
# Get document IDs and titles
curl -s -X POST ... | jq '.results[] | {docId: .docId, title: .title}'

# Get just the first result
curl -s -X POST ... | jq '.results[0]'

# Count total results
curl -s -X POST ... | jq '.totalResults'
```

### Working with Document Text

Documents with TOC (CR, SK, EU) support line-based extraction:

```bash
# Get full text
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text"

# Get TOC with line numbers
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" | jq '.'

# Extract specific section using line numbers from TOC
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | sed -n '27,50p'
```

## Reference Files

For detailed schemas, examples, and workflows, consult:

### Search Endpoints
- **`references/search-cr.md`** - Czech legislation (laws, decrees, municipal docs)
- **`references/search-jd.md`** - Czech case law (court decisions)
- **`references/search-eu.md`** - EU legislation (regulations, directives)
- **`references/search-sk.md`** - Slovak legislation
- **`references/search-comment.md`** - Legal commentaries (LIBERIS)
- **`references/search-vs.md`** - Contract templates
- **`references/search-lt.md`** - Legal literature
- **`references/search-es.md`** - EU Court decisions
- **`references/search-all.md`** - Global cross-source search

### Document Operations
- **`references/czech-legislature.md`** - Working with CR documents: versions, text, TOC, bash tools
- **`references/eu-legislature.md`** - Working with EU documents: similar patterns
- **`references/relations.md`** - Document relations: view, count, filter

## Quick Examples

### Search Czech Laws

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "občanský zákoník", "limit": 5, "validNow": true}' \
  | jq '.results[] | {docId: .main.docId, title: .main.title}'
```

### Search Case Law

```bash
curl -s -X POST "${CODEXIS_API_URL}/rest/cdx-api/search/JD" \
  -H 'Content-Type: application/json' \
  -d '{"query": "náhrada škody", "soud": ["Ústavní soud"], "limit": 5}' \
  | jq '.results[] | {docId, title, court, ecli}'
```

### Get Related Case Law for a Law

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=10" \
  | jq '.results[] | {docId, title}'
```

### Extract Specific Paragraph from Law

```bash
# 1. Get TOC to find paragraph location
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" \
  | jq '.. | objects | select(.elementId == "paragraf89") | {startLine, endLine}'

# 2. Extract those lines from text
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | sed -n '1842,1860p'
```

## Best Practices

1. **Use specific sources** - Search CR, JD, EU directly rather than ALL when source is known
2. **Filter by validity** - Use `validNow: true` or `validAt: "2024-01-01"` for legislation
3. **Paginate results** - Use offset/limit (max 50) for large result sets
4. **Cache document text** - Full text is large; fetch once and extract sections locally
5. **Use jq for filtering** - Process JSON results with jq rather than multiple API calls

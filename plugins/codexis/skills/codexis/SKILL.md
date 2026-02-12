---
name: codexis
description: This skill should be invoked whenever user needs law or legal information, czech or european. Provides guidance for querying the CODEXIS legal database API.
version: 1.1.2
---

# CODEXIS Legal Database API

CODEXIS is a comprehensive Czech legal database providing structured access to legislation, case law, EU law, legal literature, and contract templates via a REST API.

Use the `cdx` CLI for all requests. It accepts standard curl flags and `cdx://` URLs (for example `cdx://search/CR` or `cdx://doc/CR26785_2026_01_01/text`).

## Data Sources

| Code | Name | Description | Has TOC | Has Versions |
|------|------|-------------|---------|--------------|
| `CR` | Czech Legislation | Laws, decrees, regulations, municipal documents | Yes | Yes |
| `SK` | Slovak Legislation | Slovak legal acts | Yes | Yes |
| `JD` | Czech Case Law | Judicial decisions from Czech courts | Usually yes (generated sections) | Yes (usually single version) |
| `ES` | EU Court Decisions | EU Court of Justice rulings | Usually yes (generated sections) | Yes (usually single version) |
| `EU` | EU Legislation | EU regulations and directives | Yes | Yes |
| `LT` | Legal Literature | Legal publications and articles | Usually yes (generated sections) | Yes (usually single version) |
| `VS` | Contract Templates | Contract specimens and templates | No (`/toc` may return 500) | Yes (usually single version) |
| `COMMENT` | Legal Commentaries | LIBERIS legal commentary | No (`/toc` may return 500) | Yes (usually single version) |
| `ALL` | Global Search | Search across all sources | - | - |

## Core API Operations

### Search Documents

All search endpoints use POST with JSON body:

```bash
cdx -s -X POST "cdx://search/{SOURCE}" \
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
| `/doc/{docId}/toc` | GET | TOC (`CR/SK/EU` reliable, often available for `JD/ES/LT`, avoid for `VS/COMMENT`) |
| `/doc/{docId}/versions` | GET | Versions list (for non-legislation often one entry with null validity bounds) |
| `/doc/{docId}/related` | GET | Related documents |
| `/doc/{docId}/related/counts` | GET | Relation type counts |

### Direct Czech Law Access (CR by law number/year)

If the user provides a Czech law reference in the common "number/year Sb." format (for example `262/2006 Sb.`),
prefer these endpoints to skip the search step. They resolve the law to the current/nearest valid CR time version and
then behave like the corresponding `/doc/{docId}/*` endpoints.

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/cz_law/{lawNumber}/{lawYear}/meta` | GET | Resolve and return metadata (same shape as `/doc/{docId}/meta`) |
| `/cz_law/{lawNumber}/{lawYear}/text` | GET | Resolve and return text (same format as `/doc/{docId}/text`) |
| `/cz_law/{lawNumber}/{lawYear}/toc` | GET | Resolve and return TOC with line numbers (same shape as `/doc/{docId}/toc`) |
| `/cz_law/{lawNumber}/{lawYear}/versions` | GET | Return time versions list for the law |

Notes:
- `lawNumber` is the numeric part before `/` (example: `262`), `lawYear` is the year after `/` (example: `2006`).
- `/cz_law/.../text` supports the same repeated `part` query param as `/doc/{docId}/text` (example: `?part=paragraf1`).

### Document IDs

Documents use composite IDs with optional version suffix:
- Base ID: `CR26785` (Civil Code)
- Version ID: `CR26785_2026_01_01` (specific time version)
- For `CR`/`SK`, use **version IDs** for `/text` and `/toc` (base IDs return HTTP 400).

## Working with Results

### Extract Specific Fields with jq

```bash
# Get document IDs and titles (works for CR/SK/EU, COMMENT, and others)
cdx -s -X POST ... | jq '.results[] | {docId, title}'

# Get just the first result
cdx -s -X POST ... | jq '.results[0]'

# Count total results
cdx -s -X POST ... | jq '.totalResults'
```

Search results are flat objects (`docId`, `title`, ...).  
`main/timecut` structures apply to `/doc/{docId}/meta` payloads (for example under `cr.main`, `cr.timecut`), not to `search/*` responses.

### Working with Document Text

For sources with TOC (always `CR/SK/EU`, often `JD/ES/LT`), extract by section markers first:

```bash
# Get full text (CR/SK require version ID)
DOC_ID="CR26785_2026_01_01"
cdx -s "cdx://doc/${DOC_ID}/text"

# Extract one section by element marker (recommended)
SECTION="paragraf89"
cdx -s "cdx://doc/${DOC_ID}/text" \
  | awk -v section="${SECTION}" '
      $0 == "[?part=" section "]" {capture=1}
      capture {
        if ($0 ~ /^\[\?part=/ && $0 != "[?part=" section "]") exit
        print
      }
    '
```

### CR TOC Line Semantics (Current Backend)

- `startLine` is exact for all CR TOC elements.
- `endLine` is exact for leaf elements (for example `paragraf89`).
- For non-leaf elements (for example `CAST1`, `HLAVA...`), `endLine` is node-local (typically heading/header range), not subtree end.

### `text?part` Semantics

- `CR`: `cdx://doc/<VERSION_ID>/text?part=<ELEMENT_ID>` returns a section-focused preview.
- Other sources: `text?part` is not reliable for section extraction and may return full text.
- For deterministic extraction across sources, prefer TOC + marker-range extraction from `/text`.

### Deterministic Extraction Protocol (Use This First)

1. Resolve the source (`CR`, `SK`, `EU`, `JD`, etc.) and narrow search (`validNow`, `typ`, low `limit`).
2. Get `docId` and verify title before extraction. For `CR`/`SK`, ensure this is a version ID (`..._YYYY_MM_DD`) before `/text` or `/toc`.
3. For structured documents (`CR`, `SK`, `EU`, and often `JD/ES/LT`), resolve `elementId` from TOC when needed:
   - `cdx -s "cdx://doc/<DOC_ID>/toc" | jq '.. | objects | select(.elementId? == "paragraf140")'`
   - For CR leaf sections, line extraction via `startLine,endLine` is acceptable.
4. Fetch document text once and extract by marker range:
   - Start when line equals `[?part=<ELEMENT_ID>]`.
   - Stop on the next line that starts with `[?part=`.
5. Validate extraction result:
   - First lines include expected heading (for example `§ 140` / `Článek 5`).
   - If validation fails, retry with corrected `elementId` (do not trust line numbers blindly).

### Known Pitfalls (Avoid)

- Do **not** assume `cdx://doc/<DOC_ID>/text?part=<ELEMENT_ID>` behaves consistently across sources.
- Do **not** use `cdx://doc/<DOC_ID>?part=<ELEMENT_ID>` (invalid path for API resource).
- Do **not** use base `CR`/`SK` IDs on `/text` or `/toc`; use full version IDs (`..._YYYY_MM_DD`).
- Do **not** open bare `cdx://doc/<DOC_ID>` links directly; use `cdx://doc/<DOC_ID>/text` (or `/meta`) because bare doc paths return 404.
- Do **not** assume TOC is always an object; it may be a top-level array.
- Do **not** call `/toc` for `VS` and `COMMENT`; current backend returns HTTP 500.
- Do **not** assume `endLine` on non-leaf CR nodes is subtree end; for full subtree extraction, use descendants or marker boundaries.
- Do **not** inspect huge TOC JSON via `head` on a one-line payload; use `jq` filters first.

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
cdx -s -X POST "cdx://search/CR" \
  -H 'Content-Type: application/json' \
  -d '{"query": "občanský zákoník", "limit": 5, "validNow": true}' \
  | jq '.results[] | {docId, title}'
```

### Search Case Law

```bash
cdx -s -X POST "cdx://search/JD" \
  -H 'Content-Type: application/json' \
  -d '{"query": "náhrada škody", "soud": ["Ústavní soud"], "limit": 5}' \
  | jq '.results[] | {docId, title, court, ecli}'
```

### Get Related Case Law for a Law

```bash
cdx -s "cdx://doc/CR26785_2026_01_01/related?type=SOUVISEJICI_JUDIKATURA&limit=10" \
  | jq '.results[] | {docId, title}'
```

### Extract Specific Paragraph from Law

```bash
# 1. Resolve section in TOC
SECTION="paragraf89"
DOC_ID="CR26785_2026_01_01"
cdx -s "cdx://doc/${DOC_ID}/toc" \
  | jq ".. | objects | select(.elementId? == \"${SECTION}\") | {title, elementId}"

# 2. Extract section from text by marker boundaries
cdx -s "cdx://doc/${DOC_ID}/text" \
  | awk -v section="${SECTION}" '
      $0 == "[?part=" section "]" {capture=1}
      capture {
        if ($0 ~ /^\[\?part=/ && $0 != "[?part=" section "]") exit
        print
      }
    '
```

### Fetch § Directly by Law Number/Year

If you know the law reference (for example `262/2006 Sb.`), you can fetch a specific paragraph directly:

```bash
# Zákoník práce, § 1
cdx -s "cdx://cz_law/262/2006/text?part=paragraf1"

# Metadata (shows the resolved timecut docId)
cdx -s "cdx://cz_law/262/2006/meta" | jq '{docId, title: .cr.main.title, docNumber: .cr.main.docNumber}'
```

## Best Practices

1. **Use specific sources** - Search CR, JD, EU directly rather than ALL when source is known.
2. **Filter by validity** - Use `validNow: true` or `validAt: "2024-01-01"` for legislation.
3. **Constrain search early** - Add `typ` and lower `limit` to avoid wrong top hits.
4. **Extract by markers first** - Use `[?part=elementId]` marker ranges; treat TOC lines as secondary.
5. **Validate section identity** - Confirm extracted heading matches requested section.
6. **Cache document text** - Full text is large; fetch once and extract sections locally.
7. **Use jq for filtering** - Process JSON results with jq rather than multiple API calls.

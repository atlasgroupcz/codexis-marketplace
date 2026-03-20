# Austrian Case Law Search (ATJD — Judikatur)

Court decisions from 7 Austrian courts: VfGH (Constitutional), VwGH (Administrative), Justiz (Civil/Criminal), BVwG (Federal Administrative), LVwG (State Administrative), Dok (Disciplinary), Umse (Environmental Senate).

## Search Request

```bash
cdx-at -s -X POST "cdx-at://search/ATJD" \
  -H 'Content-Type: application/json' \
  -d '{
    "query": "Meinungsfreiheit",
    "application": "Vfgh",
    "dateFrom": "2024-01-01",
    "limit": 10
  }'
```

### Request Fields

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `query` | string | Full-text search | `"Grundrecht"` |
| `application` | string | Court code | `"Vfgh"`, `"Vwgh"`, `"Justiz"`, `"Bvwg"`, `"Lvwg"`, `"Dok"`, `"Umse"` |
| `documentType` | string | Document type | `"Entscheidungstext"`, `"Rechtssatz"` |
| `decisionType` | string | Decision form | `"Erkenntnis"`, `"Beschluss"` |
| `caseNumber` | string | Case file number | `"G 37/2024"` |
| `ecli` | string | ECLI identifier | `"ECLI:AT:VFGH:2024:G37.2024"` |
| `state` | string | Austrian state (LVwG only) | `"Wien"`, `"Steiermark"` |
| `dateFrom` | date | Decision date from (YYYY-MM-DD) | `"2024-01-01"` |
| `dateTo` | date | Decision date to (YYYY-MM-DD) | `"2025-12-31"` |
| `offset` | int | Pagination offset (default 0) | `0` |
| `limit` | int | Results per page (max 100, default 20) | `10` |

### Query Parameters

| Param | Values | Description |
|-------|--------|-------------|
| `sort` | `relevance`, `title`, `date` | Sort field (default: relevance) |
| `order` | `asc`, `desc` | Sort direction |

## Response Fields

| Field | Description |
|-------|-------------|
| `docId` | Display ID (e.g., ATJD1234) — internal only |
| `title` | Decision title/headline |
| `court` | Court name |
| `application` | Court application code |
| `documentType` | Entscheidungstext or Rechtssatz |
| `decisionType` | Erkenntnis, Beschluss, etc. |
| `caseNumbers` | Array of case file numbers |
| `decisionDate` | Decision date (YYYY-MM-DD) |
| `ecli` | European Case Law Identifier |
| `headnote` | Legal headnote summary |
| `legalNorms` | Referenced legal norms |
| `state` | Austrian state (for LVwG) |

## Examples

### Search Constitutional Court Decisions

```bash
cdx-at -s -X POST "cdx-at://search/ATJD" \
  -H 'Content-Type: application/json' \
  -d '{"query": "Gleichheitsgrundsatz", "application": "Vfgh", "limit": 5}' \
  | jq '.results[] | {docId, court, caseNumbers, decisionDate}'
```

### Search by Case Number

```bash
cdx-at -s -X POST "cdx-at://search/ATJD" \
  -H 'Content-Type: application/json' \
  -d '{"caseNumber": "G 37/2024", "limit": 5}' \
  | jq '.results[] | {docId, title, court}'
```

### Search by ECLI

```bash
cdx-at -s -X POST "cdx-at://search/ATJD" \
  -H 'Content-Type: application/json' \
  -d '{"ecli": "ECLI:AT:VFGH:2024:G37.2024", "limit": 1}' \
  | jq '.results[0]'
```

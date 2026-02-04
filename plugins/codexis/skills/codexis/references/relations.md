# Working with Document Relations

Documents in CODEXIS are interconnected through various relation types. This guide covers how to explore, count, and filter these relationships.

## Relation Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/doc/{docId}/related/counts` | GET | Get counts for all relation types |
| `/doc/{docId}/related` | GET | Get paginated list of related documents |

## Getting Relation Counts

First, understand what relations exist for a document:

```bash
DOC_ID="CR26785"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related/counts" | jq '.'
```

Response:
```json
{
  "docId": "CR26785",
  "counts": [
    {"type": "SOUVISEJICI_JUDIKATURA", "name": "Související judikatura ČR", "count": 47736},
    {"type": "COMMENT", "name": "Komentáře LIBERIS", "count": 9690},
    {"type": "SOUVISEJICI_LITERATURA", "name": "Související literatura", "count": 6107},
    {"type": "SOUVISEJICI_PREDPISY_ESD_ESLP", "name": "Související judikatura ESD/ESLP", "count": 146},
    {"type": "AKTIVNI_NOVELA", "name": "Aktivní derogace - mění", "count": 112},
    {"type": "AKTIVNI_DEROGACE", "name": "Aktivní derogace - ruší", "count": 72},
    {"type": "SOUVISEJICI_LEGISLATIVA_CR", "name": "Související legislativa ČR", "count": 57},
    {"type": "SOUVISEJICI_PREDPISY_EU", "name": "Související předpisy EU", "count": 48},
    {"type": "PASIVNI_NOVELA", "name": "Pasivní derogace - změněno", "count": 21},
    {"type": "DUVODOVA_ZPRAVA", "name": "Důvodové zprávy", "count": 19},
    {"type": "PROVADECI_PREDPIS", "name": "Prováděcí předpisy", "count": 7},
    {"type": "PREKLAD", "name": "Překlad", "count": 3},
    {"type": "ALL", "name": "Všechny souvislosti", "count": 42038}
  ]
}
```

## Relation Types

| Type | Description | Typical Sources |
|------|-------------|-----------------|
| `SOUVISEJICI_JUDIKATURA` | Related Czech case law | CR, EU |
| `SOUVISEJICI_PREDPISY_ESD_ESLP` | EU/ECHR court decisions | CR, EU |
| `SOUVISEJICI_LITERATURA` | Legal literature | CR, EU |
| `SOUVISEJICI_LEGISLATIVA_CR` | Related Czech legislation | All |
| `SOUVISEJICI_PREDPISY_EU` | Related EU legislation | CR, JD |
| `AKTIVNI_NOVELA` | Active derogation - amends | CR, SK |
| `AKTIVNI_DEROGACE` | Active derogation - repeals | CR, SK |
| `PASIVNI_NOVELA` | Passive derogation - was amended | CR, SK |
| `PROVADECI_PREDPIS` | Implementing regulations | CR, EU |
| `DUVODOVA_ZPRAVA` | Explanatory reports | CR |
| `COMMENT` | LIBERIS commentaries | CR |
| `PREKLAD` | Translations | CR, EU |
| `ALL` | All relations | All |

## Getting Related Documents

### Basic Query

```bash
DOC_ID="CR26785"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related" | jq '.'
```

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `type` | string | ALL | Filter by relation type |
| `part` | string | - | Filter by document part (paragraph) |
| `offset` | int | 0 | Pagination offset |
| `limit` | int | 20 | Results per page (max 100) |
| `sort` | string | relevance | Sort: relevance, title, date |
| `order` | string | desc | Order: asc, desc |

### Response Schema

```json
{
  "docId": "CR26785",
  "relationType": "SOUVISEJICI_JUDIKATURA",
  "totalResults": 47736,
  "offset": 0,
  "limit": 10,
  "results": [
    {
      "docId": "JD1543780",
      "source": "JD",
      "title": "Rozsudek - Rozsudek Krajského soudu...",
      "relationType": "SOUVISEJICI_JUDIKATURA",
      "validFrom": "2019-01-16",
      "validTo": null
    }
  ]
}
```

## Filtering by Relation Type

### Get Related Case Law

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=10" | \
  jq '.results[] | {docId, title, date: .validFrom}'
```

### Get Related EU Legislation

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_PREDPISY_EU&limit=10" | \
  jq '.results[] | {docId, title}'
```

### Get Implementing Regulations

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=PROVADECI_PREDPIS&limit=10" | \
  jq '.results[] | {docId, title}'
```

### Get Commentaries

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=COMMENT&limit=10" | \
  jq '.results[] | {docId, title}'
```

## Filtering by Document Part

Get relations for a specific paragraph/section:

```bash
# Relations for paragraph 89 of Civil Code
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&part=paragraf89&limit=10" | \
  jq '.results[] | {docId, title}'
```

This is useful for finding case law interpreting a specific provision.

## Pagination

### Navigate Through Results

```bash
# First page
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=20&offset=0"

# Second page
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=20&offset=20"

# Third page
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&limit=20&offset=40"
```

### Get All Related Documents (Scripted)

```bash
DOC_ID="CR26785"
TYPE="SOUVISEJICI_JUDIKATURA"
LIMIT=100
OFFSET=0

# Get total count first
TOTAL=$(curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related/counts" | \
  jq -r ".counts[] | select(.type == \"${TYPE}\") | .count")

echo "Total: $TOTAL"

# Iterate through pages
while [ $OFFSET -lt $TOTAL ]; do
  curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related?type=${TYPE}&limit=${LIMIT}&offset=${OFFSET}" | \
    jq -r '.results[] | "\(.docId)\t\(.title)"'
  OFFSET=$((OFFSET + LIMIT))
done
```

## Sorting

### Sort by Date (Most Recent First)

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_JUDIKATURA&sort=date&order=desc&limit=10" | \
  jq '.results[] | {docId, title, date: .validFrom}'
```

### Sort by Title (Alphabetically)

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/related?type=SOUVISEJICI_LEGISLATIVA_CR&sort=title&order=asc&limit=10" | \
  jq '.results[] | {docId, title}'
```

## Practical Workflows

### Workflow 1: Research a Law's Impact

```bash
DOC_ID="CR26785"

# 1. Get overview of all relations
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related/counts" | \
  jq '.counts[] | "\(.name): \(.count)"' -r

# 2. Get recent case law
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related?type=SOUVISEJICI_JUDIKATURA&sort=date&limit=5" | \
  jq '.results[] | {docId, title, date: .validFrom}'

# 3. Get implementing regulations
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related?type=PROVADECI_PREDPIS" | \
  jq '.results[] | {docId, title}'
```

### Workflow 2: Trace Amendment History

```bash
DOC_ID="CR26785"

# Laws that amended this one
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related?type=PASIVNI_NOVELA&sort=date" | \
  jq '.results[] | {docId, title, date: .validFrom}'

# Laws this one amends
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related?type=AKTIVNI_NOVELA" | \
  jq '.results[] | {docId, title}'
```

### Workflow 3: Find Paragraph-Specific Case Law

```bash
DOC_ID="CR26785"
PARAGRAPH="paragraf89"

# Get case law for specific paragraph
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related?type=SOUVISEJICI_JUDIKATURA&part=${PARAGRAPH}&limit=10" | \
  jq '.results[] | {docId, title}'
```

### Workflow 4: Cross-Reference EU and Czech Law

```bash
# Find Czech law
CR_DOC="CR26785"

# Get related EU legislation
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${CR_DOC}/related?type=SOUVISEJICI_PREDPISY_EU" | \
  jq '.results[] | {docId, title}'

# For each EU doc, find other Czech implementations
EU_DOC="EU213382"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${EU_DOC}/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

### Workflow 5: Export Relations to CSV

```bash
DOC_ID="CR26785"
TYPE="SOUVISEJICI_JUDIKATURA"

echo "docId,title,date" > relations.csv
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/related?type=${TYPE}&limit=100" | \
  jq -r '.results[] | [.docId, .title, .validFrom] | @csv' >> relations.csv
```

## Summary of Common Queries

| Use Case | Query |
|----------|-------|
| All relation counts | `GET /doc/{id}/related/counts` |
| Case law for a law | `GET /doc/{id}/related?type=SOUVISEJICI_JUDIKATURA` |
| Case law for specific § | `GET /doc/{id}/related?type=SOUVISEJICI_JUDIKATURA&part=paragraf89` |
| Implementing regulations | `GET /doc/{id}/related?type=PROVADECI_PREDPIS` |
| Related EU law | `GET /doc/{id}/related?type=SOUVISEJICI_PREDPISY_EU` |
| Amendment history | `GET /doc/{id}/related?type=PASIVNI_NOVELA` |
| Commentaries | `GET /doc/{id}/related?type=COMMENT` |
| Recent relations | `GET /doc/{id}/related?sort=date&order=desc` |

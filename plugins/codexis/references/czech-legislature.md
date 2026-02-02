# Working with Czech Legislature (CR)

Czech legislation documents have rich structure: time versions, table of contents, and full text with anchors. This guide covers efficient retrieval and extraction techniques.

## Document Structure

Czech laws consist of:
- **Base document** (`CR26785`) - the law itself
- **Time versions** (`CR26785_2026_01_01`) - law as valid at specific date
- **Text with anchors** - structured content with element IDs
- **Table of contents** - hierarchical structure with line numbers

## Retrieving Document Metadata

```bash
DOC_ID="CR26785"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/meta" | jq '.'
```

Response includes:
- `main` - base document info (title, type, dates)
- `timecut` - current version info

## Working with Time Versions

### List All Versions

```bash
DOC_ID="CR26785"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/versions" | jq '.'
```

Response:
```json
[
  {
    "versionId": "CR26785_2026_01_01",
    "validFrom": "2026-01-01",
    "validTo": null,
    "amendmentDocIds": ["CR156140_2026_01_01", ...]
  },
  {
    "versionId": "CR26785_2025_07_01",
    "validFrom": "2025-07-01",
    "validTo": "2025-12-31",
    "amendmentDocIds": [...]
  }
]
```

### Get Version Valid at Specific Date

```bash
# Find version valid on 2020-01-01
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/versions" | \
  jq '.[] | select(.validFrom <= "2020-01-01" and (.validTo == null or .validTo >= "2020-01-01"))'
```

### Get Text for Specific Version

Use the `versionId` (timecutId) to get text as it was at that time:

```bash
VERSION_ID="CR26785_2020_01_01"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${VERSION_ID}/text"
```

## Table of Contents (TOC)

The TOC provides hierarchical structure with line numbers for text extraction.

### Get TOC

```bash
DOC_ID="CR26785"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/toc" | jq '.'
```

### TOC Structure

```json
{
  "title": "ČÁST PRVNÍ - Obecná část",
  "subtitle": "§ 1 - § 654",
  "level": 1,
  "elementId": "CAST1",
  "startLine": 12,
  "endLine": 5129,
  "children": [
    {
      "title": "HLAVA I - Předmět úpravy...",
      "level": 2,
      "elementId": "HLAVA1",
      "startLine": 17,
      "endLine": 142,
      "children": [...]
    }
  ]
}
```

### Find Specific Section in TOC

```bash
# Find paragraph 89
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" | \
  jq '.. | objects | select(.elementId == "paragraf89")'
```

### List All Paragraphs

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" | \
  jq '.. | objects | select(.elementId | startswith("paragraf")) | {title, elementId, startLine, endLine}'
```

### Find Section by Title

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" | \
  jq '.. | objects | select(.title | contains("Smlouvy"))'
```

## Document Text

### Get Full Text

```bash
DOC_ID="CR26785"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text"
```

### Text Format

The text includes special markers:
- `[id: #unit-N]` - Unit anchors
- `[?part=elementId]` - Section markers (match TOC elementId)
- `[Title](cdx://doc/DOCID)` - Links to other documents

Example:
```
[id: #unit-26]
[?part=paragraf1]
§ 1
[id: #unit-27]
(1) Ustanovení právního řádu upravující...
```

## Extracting Specific Sections

### Using Line Numbers from TOC

1. Get line numbers:
```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" | \
  jq '.. | objects | select(.elementId == "paragraf89") | {startLine, endLine}'
# Output: {"startLine": 1842, "endLine": 1860}
```

2. Extract those lines:
```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | sed -n '1842,1860p'
```

### Using grep to Find Content

```bash
# Find all paragraphs mentioning "smlouva"
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | grep -i "smlouva"

# Find with context
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | grep -B2 -A5 "§ 89"
```

### Using head/tail for Ranges

```bash
# Get first 100 lines
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | head -100

# Get lines 500-600
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | sed -n '500,600p'

# Get last 50 lines
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | tail -50
```

### Extract Section by Element ID

```bash
# Get text from section marker to next section
DOC_ID="CR26785"
SECTION="paragraf89"

curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text" | \
  sed -n "/\[?part=${SECTION}\]/,/\[?part=paragraf[0-9]/p" | head -n -1
```

## Practical Workflows

### Workflow 1: Get Specific Paragraph

```bash
DOC_ID="CR26785"
PARA_NUM="89"

# Step 1: Find line numbers
LINES=$(curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/toc" | \
  jq -r ".. | objects | select(.elementId == \"paragraf${PARA_NUM}\") | \"\(.startLine),\(.endLine)\"")

# Step 2: Extract text
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text" | sed -n "${LINES}p"
```

### Workflow 2: Compare Versions

```bash
DOC_ID="CR26785"

# Get current version
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text" > /tmp/current.txt

# Get old version
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}_2020_01_01/text" > /tmp/old.txt

# Compare
diff /tmp/old.txt /tmp/current.txt | head -50
```

### Workflow 3: Extract All Paragraphs to JSON

```bash
DOC_ID="CR26785"

# Get TOC with paragraph info
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/toc" | \
  jq '[.. | objects | select(.elementId | startswith("paragraf")) | {
    paragraph: .title,
    elementId: .elementId,
    startLine: .startLine,
    endLine: .endLine
  }]'
```

### Workflow 4: Find Where a Term is Defined

```bash
DOC_ID="CR26785"

# Search for definition patterns
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text" | \
  grep -n "se rozumí\|znamená\|je definován"
```

### Workflow 5: Cache Text Locally

For repeated operations, cache the full text:

```bash
DOC_ID="CR26785"
CACHE_FILE="/tmp/codexis_${DOC_ID}.txt"

# Download once
if [ ! -f "$CACHE_FILE" ]; then
  curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/${DOC_ID}/text" > "$CACHE_FILE"
fi

# Use cached file
grep "smlouva" "$CACHE_FILE"
sed -n '100,200p' "$CACHE_FILE"
```

## Processing Tips

### Clean Text Output

Remove anchors for cleaner reading:

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | \
  sed 's/\[id: #[^]]*\]//g' | \
  sed 's/\[?part=[^]]*\]//g'
```

### Convert Links to Plain Text

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/text" | \
  sed 's/\[\([^]]*\)\](cdx:\/\/[^)]*)/\1/g'
```

### Count Paragraphs

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" | \
  jq '[.. | objects | select(.elementId | startswith("paragraf"))] | length'
```

### Get TOC as Flat List

```bash
curl -s "${CODEXIS_API_URL}/rest/cdx-api/doc/CR26785/toc" | \
  jq '[.. | objects | select(.title) | {title, level, elementId}]'
```

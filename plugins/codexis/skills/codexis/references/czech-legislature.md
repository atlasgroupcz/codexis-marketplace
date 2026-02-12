# Working with Czech Legislature (CR)

Czech legislation documents have rich structure: time versions, table of contents, and full text with anchors. This guide covers efficient retrieval and extraction techniques.

## cdx Usage
Use `cdx` for requests. It accepts standard curl flags and `cdx://` URLs.

```bash
BASE_DOC_ID="CR26785"
DOC_ID="CR26785_2026_01_01"  # full version ID (required for /text and /toc)
cdx -s "cdx://doc/${BASE_DOC_ID}/meta"
```

For CR documents, `/text` and `/toc` require a full version ID (for example `CR26785_2026_01_01`).  
Use `BASE_DOC_ID` for `/versions` lookups and `DOC_ID` (version ID) for `/text` and `/toc`.

## Direct Access by Law Number/Year (Skip Search)

If the user provides a law reference like `262/2006 Sb.`, you can directly resolve and fetch data without first
finding a `CR...` id via search:

```bash
# Resolve and get metadata (returns resolved timecut under .docId)
cdx -s "cdx://cz_law/262/2006/meta" | jq '.'

# Fetch a specific paragraph directly
cdx -s "cdx://cz_law/262/2006/text?part=paragraf1"

# ToC and versions
cdx -s "cdx://cz_law/262/2006/toc" | jq '.'
cdx -s "cdx://cz_law/262/2006/versions" | jq '.'
```

Notes:
- `cz_law/.../text` supports repeated `part` query params (same as `/doc/{docId}/text`).
- The resolved `.docId` is a timecut id (for example `CR13986_2026_01_01`) and can be used with `cdx://doc/<docId>/text`.

## Document Structure

Czech laws consist of:
- **Base document** (`CR26785`) - the law itself
- **Time versions** (`CR26785_2026_01_01`) - law as valid at specific date
- **Text with anchors** - structured content with element IDs
- **Table of contents** - hierarchical structure with line numbers

## Retrieving Document Metadata

```bash
BASE_DOC_ID="CR26785"
cdx -s "cdx://doc/${BASE_DOC_ID}/meta" | jq '.'
```

Response includes:
- `cr.main` - base document info (title, type, dates)
- `cr.timecut` - current version info

## Working with Time Versions

### List All Versions

```bash
BASE_DOC_ID="CR26785"
cdx -s "cdx://doc/${BASE_DOC_ID}/versions" | jq '.'
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
cdx -s "cdx://doc/CR26785/versions" | \
  jq '.[] | select(.validFrom <= "2020-01-01" and (.validTo == null or .validTo >= "2020-01-01"))'
```

### Get Text for Specific Version

Use `versionId` to get text as it was at that time:

```bash
VERSION_ID="CR26785_2020_01_01"
cdx -s "cdx://doc/${VERSION_ID}/text"
```

## Table of Contents (TOC)

The TOC provides hierarchical structure with line numbers.

Current CR behavior:
- `startLine` is exact for all TOC elements.
- `endLine` is exact for leaf elements (for example paragraphs).
- For non-leaf elements, `endLine` is node-local (typically heading/header range), not subtree end.

### Get TOC

```bash
DOC_ID="CR26785_2026_01_01"
cdx -s "cdx://doc/${DOC_ID}/toc" | jq '.'
```

### TOC Structure

TOC can be returned as a top-level array:

```json
[
  {
    "title": "ČÁST PRVNÍ - Obecná část",
    "subtitle": "§ 1 - § 654",
    "level": 1,
    "elementId": "CAST1",
    "startLine": 7,
    "endLine": 9,
    "children": [
      {
        "title": "HLAVA I - Předmět úpravy...",
        "level": 2,
        "elementId": "HLAVA1",
        "startLine": 10,
        "endLine": 12,
        "children": []
      }
    ]
  }
]
```

### Find Specific Section in TOC

```bash
# Find paragraph 89
cdx -s "cdx://doc/CR26785_2026_01_01/toc" | \
  jq '.. | objects | select(.elementId? == "paragraf89")'
```

### List All Paragraphs

```bash
cdx -s "cdx://doc/CR26785_2026_01_01/toc" | \
  jq '.. | objects | select(.elementId? | startswith("paragraf")) | {title, elementId, startLine, endLine}'
```

### Find Section by Title

```bash
cdx -s "cdx://doc/CR26785_2026_01_01/toc" | \
  jq '.. | objects | select(.title? | contains("Smlouvy"))'
```

## Document Text

### Get Full Text

```bash
DOC_ID="CR26785_2026_01_01"
cdx -s "cdx://doc/${DOC_ID}/text"
```

### Text Format

The text includes special markers:
- `[id: #unit-N]` - Unit anchors
- `[?part=elementId]` - Section markers (match TOC elementId)
- `[Title](cdx://doc/DOCID)` - Links to other documents (open as `cdx://doc/DOCID/text` or `/meta`; bare `cdx://doc/DOCID` returns 404)

Example:
```
[id: #unit-26]
[?part=paragraf1]
§ 1
[id: #unit-27]
(1) Ustanovení právního řádu upravující...
```

## Extracting Specific Sections

### Using Section Markers (Recommended)

```bash
DOC_ID="CR26785_2026_01_01"
SECTION="paragraf89"

cdx -s "cdx://doc/${DOC_ID}/text" | \
  awk -v section="${SECTION}" '
    $0 == "[?part=" section "]" {capture=1}
    capture {
      if ($0 ~ /^\[\?part=/ && $0 != "[?part=" section "]") exit
      print
    }
  '
```

### Quick Section Preview (`text?part`)

For CR, this endpoint currently returns a section-focused preview:

```bash
DOC_ID="CR26785_2026_01_01"
SECTION="paragraf89"
cdx -s "cdx://doc/${DOC_ID}/text?part=${SECTION}"
```

### Using Line Numbers from TOC

Recommended for leaf elements (`paragraf...`). Validate heading after extraction.

```bash
DOC_ID="CR26785_2026_01_01"
SECTION="paragraf89"

LINES=$(cdx -s "cdx://doc/${DOC_ID}/toc" | \
  jq -r ".. | objects | select(.elementId? == \"${SECTION}\") | \"\(.startLine),\(.endLine)\"")

cdx -s "cdx://doc/${DOC_ID}/text" | sed -n "${LINES}p"
```

### Using grep to Find Content

```bash
# Find all paragraphs mentioning "smlouva"
cdx -s "cdx://doc/CR26785_2026_01_01/text" | grep -i "smlouva"

# Find with context
cdx -s "cdx://doc/CR26785_2026_01_01/text" | grep -B2 -A5 "§ 89"
```

### Using head/tail for Ranges

```bash
# Get first 100 lines
cdx -s "cdx://doc/CR26785_2026_01_01/text" | head -100

# Get lines 500-600
cdx -s "cdx://doc/CR26785_2026_01_01/text" | sed -n '500,600p'

# Get last 50 lines
cdx -s "cdx://doc/CR26785_2026_01_01/text" | tail -50
```

### Extract Section by Element ID

```bash
# Get text from section marker to next section
DOC_ID="CR26785_2026_01_01"
SECTION="paragraf89"

cdx -s "cdx://doc/${DOC_ID}/text" | \
  awk -v section="${SECTION}" '
    $0 == "[?part=" section "]" {capture=1}
    capture {
      if ($0 ~ /^\[\?part=/ && $0 != "[?part=" section "]") exit
      print
    }
  '
```

## Known Pitfalls

- `cdx://doc/<DOC_ID>/text?part=<SECTION>` works for CR previews, but keep marker extraction as the deterministic method.
- `cdx://doc/<DOC_ID>?part=<SECTION>` is an invalid resource path.
- `cdx://doc/<DOC_ID>` (without endpoint suffix) returns 404; append `/text` or `/meta`.
- For CR documents, base IDs (for example `CR26785`) return 400 on `/text` and `/toc`; use version IDs (`CR26785_YYYY_MM_DD`).
- TOC may be an array; avoid assumptions such as `.toc` object wrapping.
- For non-leaf TOC elements, `endLine` is node-local, not subtree end.

## Practical Workflows

### Workflow 1: Get Specific Paragraph

```bash
DOC_ID="CR26785_2026_01_01"
PARA_NUM="89"
SECTION="paragraf${PARA_NUM}"

# Step 1: Resolve section in TOC
cdx -s "cdx://doc/${DOC_ID}/toc" | \
  jq ".. | objects | select(.elementId? == \"${SECTION}\") | {title, elementId}"

# Step 2: Extract by marker range
cdx -s "cdx://doc/${DOC_ID}/text" | \
  awk -v section="${SECTION}" '
    $0 == "[?part=" section "]" {capture=1}
    capture {
      if ($0 ~ /^\[\?part=/ && $0 != "[?part=" section "]") exit
      print
    }
  '

# Step 3: Validate heading
# expected first heading line contains "§ ${PARA_NUM}"
```

### Workflow 2: Compare Versions

```bash
BASE_DOC_ID="CR26785"
DOC_ID="CR26785_2026_01_01"

# Get current version
cdx -s "cdx://doc/${DOC_ID}/text" > /tmp/current.txt

# Get old version
cdx -s "cdx://doc/${BASE_DOC_ID}_2020_01_01/text" > /tmp/old.txt

# Compare
diff /tmp/old.txt /tmp/current.txt | head -50
```

### Workflow 3: Extract All Paragraphs to JSON

```bash
DOC_ID="CR26785_2026_01_01"

# Get TOC with paragraph info
cdx -s "cdx://doc/${DOC_ID}/toc" | \
  jq '[.. | objects | select(.elementId? | startswith("paragraf")) | {
    paragraph: .title,
    elementId: .elementId,
    startLine: .startLine,
    endLine: .endLine
  }]'
```

### Workflow 4: Find Where a Term is Defined

```bash
DOC_ID="CR26785_2026_01_01"

# Search for definition patterns
cdx -s "cdx://doc/${DOC_ID}/text" | \
  grep -n "se rozumí\|znamená\|je definován"
```

### Workflow 5: Cache Text Locally

For repeated operations, cache the full text:

```bash
DOC_ID="CR26785_2026_01_01"
CACHE_FILE="/tmp/codexis_${DOC_ID}.txt"

# Download once
if [ ! -f "$CACHE_FILE" ]; then
  cdx -s "cdx://doc/${DOC_ID}/text" > "$CACHE_FILE"
fi

# Use cached file
grep "smlouva" "$CACHE_FILE"
sed -n '100,200p' "$CACHE_FILE"
```

## Processing Tips

### Clean Text Output

Remove anchors for cleaner reading:

```bash
cdx -s "cdx://doc/CR26785_2026_01_01/text" | \
  sed 's/\[id: #[^]]*\]//g' | \
  sed 's/\[?part=[^]]*\]//g'
```

### Convert Links to Plain Text

```bash
cdx -s "cdx://doc/CR26785_2026_01_01/text" | \
  sed 's/\[\([^]]*\)\](cdx:\/\/[^)]*)/\1/g'
```

### Count Paragraphs

```bash
cdx -s "cdx://doc/CR26785_2026_01_01/toc" | \
  jq '[.. | objects | select(.elementId? | startswith("paragraf"))] | length'
```

### Get TOC as Flat List

```bash
cdx -s "cdx://doc/CR26785_2026_01_01/toc" | \
  jq '[.. | objects | select(.title) | {title, level, elementId}]'
```

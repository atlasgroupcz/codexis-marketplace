# Working with EU Legislature (EU)

EU legislation documents share similar structure with Czech legislation: time versions, table of contents, and full text. This guide covers EU-specific patterns.

## cdx Usage
Use `cdx` for requests. It accepts standard curl flags and `cdx://` URLs.

```bash
DOC_ID="EU213382"
cdx -s "cdx://doc/${DOC_ID}/meta"
```

## Document Structure

EU documents consist of:
- **Base document** (`EU213382`) - the regulation/directive
- **Time versions** - document as valid at specific date
- **Text with anchors** - structured content
- **Table of contents** - hierarchical structure with line numbers

## Retrieving Document Metadata

```bash
DOC_ID="EU213382"
cdx -s "cdx://doc/${DOC_ID}/meta" | jq '.'
```

Key fields:
- `celex` - CELEX number (e.g., `32016R0679`)
- `docType` - Nařízení, Směrnice, Rozhodnutí
- `validFrom` / `validTo` - validity period

## Working with Time Versions

### List All Versions

```bash
DOC_ID="EU213382"
cdx -s "cdx://doc/${DOC_ID}/versions" | jq '.'
```

### Get Specific Version

```bash
VERSION_ID="EU213382_2024_01_01"
cdx -s "cdx://doc/${VERSION_ID}/text"
```

## Table of Contents (TOC)

EU documents have hierarchical TOC similar to Czech laws.

### Get TOC

```bash
DOC_ID="EU213382"
cdx -s "cdx://doc/${DOC_ID}/toc" | jq '.'
```

### EU Document Structure

EU regulations typically follow this hierarchy:
- Kapitoly (Chapters)
- Oddíly (Sections)
- Články (Articles)
- Odstavce (Paragraphs)

### Find Specific Article

```bash
# Find Article 5
cdx -s "cdx://doc/EU213382/toc" | \
  jq '.. | objects | select(.title | contains("Článek 5"))'
```

### List All Articles

```bash
cdx -s "cdx://doc/EU213382/toc" | \
  jq '.. | objects | select(.title | startswith("Článek")) | {title, startLine, endLine}'
```

## Document Text

### Get Full Text

```bash
DOC_ID="EU213382"
cdx -s "cdx://doc/${DOC_ID}/text"
```

### EU Text Format

EU document text follows similar patterns to Czech laws:
- `[id: #unit-N]` - Unit anchors
- `[?part=elementId]` - Section markers
- Links to other EU documents

## Extracting Specific Sections

### Using Line Numbers from TOC

```bash
# 1. Find article line numbers
cdx -s "cdx://doc/EU213382/toc" | \
  jq '.. | objects | select(.title | contains("Článek 5")) | {startLine, endLine}'

# 2. Extract those lines
cdx -s "cdx://doc/EU213382/text" | sed -n 'START,ENDp'
```

### Search Within Text

```bash
# Find all references to "osobní údaje"
cdx -s "cdx://doc/EU213382/text" | grep -i "osobní údaje"

# Find with context
cdx -s "cdx://doc/EU213382/text" | grep -B2 -A5 "Článek 5"
```

## Practical Workflows

### Workflow 1: Get GDPR Article

```bash
# GDPR is typically EU document with CELEX 32016R0679
# First find the docId
cdx -s -X POST "cdx://search/EU" \
  -H 'Content-Type: application/json' \
  -d '{"query": "32016R0679", "limit": 1}' | jq -r '.results[0].docId'

# Then get specific article
DOC_ID="EU_GDPR_DOC_ID"
cdx -s "cdx://doc/${DOC_ID}/toc" | \
  jq '.. | objects | select(.title | contains("Článek 17"))'
```

### Workflow 2: Find Czech Implementation

```bash
# Find EU directive
cdx -s -X POST "cdx://search/EU" \
  -H 'Content-Type: application/json' \
  -d '{"query": "směrnice digitální služby", "typ": ["Směrnice"], "limit": 3}'

# Get related Czech legislation
DOC_ID="EU_DIRECTIVE_ID"
cdx -s "cdx://doc/${DOC_ID}/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

### Workflow 3: Compare EU and Czech Text

```bash
# Get EU regulation text
cdx -s "cdx://doc/EU_DOC_ID/text" > /tmp/eu_text.txt

# Get Czech implementing law
cdx -s "cdx://doc/CR_DOC_ID/text" > /tmp/cr_text.txt

# Search for common terms
grep -i "sankce" /tmp/eu_text.txt
grep -i "sankce" /tmp/cr_text.txt
```

### Workflow 4: Get Recitals (Důvody)

EU regulations have recitals before the main articles:

```bash
# Recitals are typically before "Článek 1"
cdx -s "cdx://doc/EU213382/toc" | \
  jq '.. | objects | select(.title | contains("Článek 1")) | .startLine'

# Get text before that line
START_LINE=<from_above>
cdx -s "cdx://doc/EU213382/text" | head -$((START_LINE - 1))
```

## EU-Specific Considerations

### Document Languages

EU documents in CODEXIS are in Czech translation. For official EU languages, refer to EUR-Lex directly.

### CELEX Number Lookup

| Prefix | Document Type |
|--------|---------------|
| `3` | Legislation |
| `6` | Case law |
| `C` | Preparatory acts |
| `E` | EFTA documents |

Document type codes:
- `R` = Regulation (Nařízení)
- `L` = Directive (Směrnice)
- `D` = Decision (Rozhodnutí)

### Finding Related Czech Transposition

For directives (Směrnice), find implementing Czech laws:

```bash
cdx -s "cdx://doc/EU_DIRECTIVE_ID/related?type=SOUVISEJICI_LEGISLATIVA_CR" | \
  jq '.results[] | {docId, title}'
```

### Finding EU Court Interpretation

```bash
cdx -s "cdx://doc/EU_DOC_ID/related?type=SOUVISEJICI_PREDPISY_ESD_ESLP" | \
  jq '.results[] | {docId, title}'
```

## Processing Tips

### Clean Text

```bash
cdx -s "cdx://doc/EU213382/text" | \
  sed 's/\[id: #[^]]*\]//g' | \
  sed 's/\[?part=[^]]*\]//g'
```

### Extract Chapter Structure

```bash
cdx -s "cdx://doc/EU213382/toc" | \
  jq '[.. | objects | select(.level <= 2) | {title, level}]'
```

### Count Articles

```bash
cdx -s "cdx://doc/EU213382/toc" | \
  jq '[.. | objects | select(.title | startswith("Článek"))] | length'
```

# Getting and Assessing Changes in Czech Law

Use this guide when the task is not only to find a Czech amendment, but to determine:

- whether a legal-text change exists on a concrete date,
- which version boundary proves it,
- which amending act caused it,
- which provisions actually changed,
- what the change means for a named actor, clause, internal rule, or compliance workflow.

## Operating Rules

- Start with the first task-serving `cdx-cli` request. Do not run `which cdx-cli`, inspect env vars, or call bare `cdx-cli` as a preflight step.
- If the law number and year are known, skip search and go straight to `cdx://cz_law/{lawNumber}/{lawYear}/...`.
- For change questions, start with `/versions`, not `/text`, not broad keyword search, and not web lookup.
- Use `cdx://cz_law/{lawNumber}/{lawYear}/meta` to obtain both the base CR id (`.cr.main.docId`) and the currently resolved version id (`.docId`).
- Use the base CR id for `/related` and `/related/counts`.
- Use version ids for `/text` and `/toc`.
- If `/versions` shows no boundary at the target date, stop and answer that directly.
- Use `amendmentDocIds` from `/versions` before falling back to relation-based provenance.

## Quick Route by Question

- `kdy se naposledy novelizoval ...`, `jaká je poslední novela ...` -> resolve the law, inspect `/versions`, then use `amendmentDocIds`.
- `co se mění od 1.1.2027`, `jaké změny budou od ...` -> prove a boundary with `/versions`; if none exists, say so.
- `porovnej znění ... před/po datu` -> resolve exact old and new version ids from `/versions`, then diff.
- `opravdu novela X změnila § Y`, `ověř to`, `zkontroluj` -> reproduce the version boundary first, then inspect the concrete section text.
- `co to znamená pro zaměstnavatele / obec / školu / fond / úřad` -> detect the concrete change first, then explain impact from the changed provisions only.

## 1. Resolve the Target Law

### If the law number and year are known

Go directly to the Czech-law endpoint:

```bash
LAW_NUM=89
LAW_YEAR=2012

cdx-cli get cdx://cz_law/${LAW_NUM}/${LAW_YEAR}/meta | jq .
```

Useful fields:

```bash
cdx-cli get cdx://cz_law/${LAW_NUM}/${LAW_YEAR}/meta | jq '{baseDocId: .cr.main.docId, currentVersionId: .docId, title: .cr.main.title, effectiveFrom: .cr.main.ucinnyOd}'
```

For `89/2012 Sb.` this currently resolves to:

- base CR id: `CR26785`
- resolved version id: `CR26785_2026_01_01`

### If only a title or topic is known

Use a narrow legislation search, then inspect the candidate title and number:

```bash
cdx-cli search CR --query "občanský zákoník" --type Zákon --current --limit 10 | jq '.results[] | {docId, title, docNumber, docType, ucinnyod}'
```

If the query is noisy, search by number/year wording instead:

```bash
cdx-cli search CR --query "89/2012" --limit 10 | jq '.results[] | {docId, title, docNumber, docType}'
```

Do not trust broad keyword search if the law number is already known.

## 2. Prove the Version Boundary

Fetch the version list first:

```bash
LAW_NUM=89
LAW_YEAR=2012

VERSIONS=$(cdx-cli get cdx://cz_law/${LAW_NUM}/${LAW_YEAR}/versions)
echo "$VERSIONS" | jq '.[] | {versionId, validFrom, validTo, amendmentDocIds}'
```

### Check whether anything changes on a concrete date

```bash
TARGET=2026-01-01

echo "$VERSIONS" | jq --arg d "$TARGET" '.[] | select(.validFrom == $d) | {versionId, validFrom, validTo, amendmentDocIds}'
```

If this returns nothing, there is no proved version boundary on that date. Stop there unless the user asked for a broader date range.

### Resolve the version immediately before and at the target date

```bash
TARGET=2026-01-01

OLD=$(echo "$VERSIONS" | jq -r --arg d "$TARGET" '
  map(select(.validTo != null and .validTo < $d))
  | sort_by(.validTo)
  | last
  | .versionId // empty
')

NEW=$(echo "$VERSIONS" | jq -r --arg d "$TARGET" '
  map(select(.validFrom <= $d and (.validTo == null or .validTo >= $d)))
  | sort_by(.validFrom)
  | last
  | .versionId // empty
')

printf 'OLD=%s\nNEW=%s\n' "$OLD" "$NEW"
```

For "what changes from `2026-01-01`", the decisive proof is a version entry whose `validFrom` equals `2026-01-01`.

## 3. Identify Which Amending Act Caused the Change

Use `amendmentDocIds` from the boundary version first:

```bash
TARGET=2026-01-01

echo "$VERSIONS" | jq -r --arg d "$TARGET" '
  .[]
  | select(.validFrom == $d)
  | .amendmentDocIds[]?
'
```

Resolve each amending act to metadata:

```bash
for DOC in $(echo "$VERSIONS" | jq -r --arg d "$TARGET" '.[] | select(.validFrom == $d) | .amendmentDocIds[]?'); do
  cdx-cli get cdx://doc/${DOC}/meta | jq '{docId: .docId, title: .cr.main.title, docNumber: .cr.main.docNumber, effectiveFrom: .cr.main.ucinnyOd}'
done
```

If `amendmentDocIds` are empty or incomplete, inspect change-related relations on the base document:

```bash
BASE_DOC_ID=$(cdx-cli get cdx://cz_law/${LAW_NUM}/${LAW_YEAR}/meta | jq -r '.cr.main.docId')

cdx-cli get cdx://doc/${BASE_DOC_ID}/related/counts | jq '.counts[] | select(.type | test("NOVELA|DEROGACE"))'
```

Then fetch concrete amending relations:

```bash
cdx-cli get "cdx://doc/${BASE_DOC_ID}/related?type=PASIVNI_NOVELA&sort=date&order=desc&limit=10" | jq '.results[] | {docId, title, validFrom}'
```

Use relation-based provenance as a fallback or cross-check, not as the first proof of the date boundary.

## 4. Extract the Changed Text

### Whole-law comparison

Fetch the exact versions first:

```bash
cdx-cli get cdx://doc/${OLD}/text > /tmp/old-law.md
cdx-cli get cdx://doc/${NEW}/text > /tmp/new-law.md
diff -u /tmp/old-law.md /tmp/new-law.md
```

For large laws, whole-document diff is only orientation. Use it to find candidate sections, then narrow down.

### Section comparison when the part id is known

```bash
SECTION=paragraf3028

cdx-cli get "cdx://doc/${OLD}/text?part=${SECTION}" > /tmp/old-section.md
cdx-cli get "cdx://doc/${NEW}/text?part=${SECTION}" > /tmp/new-section.md
diff -u /tmp/old-section.md /tmp/new-section.md
```

### Section comparison when the part id is not known yet

Inspect the table of contents:

```bash
cdx-cli get cdx://doc/${NEW}/toc | jq .
```

Then fetch only the relevant part:

```bash
cdx-cli get "cdx://doc/${NEW}/text?part=${SECTION}"
```

Use `/toc` to resolve part ids accurately before relying on partial text extraction.

## 5. Assess the Practical Impact

For impact questions, do not explain from the amendment title alone. First prove:

1. that a new version starts on the target date,
2. which provision changed,
3. what the old and new wording actually say.

Only then explain what changes for the named actor.

Suggested sequence:

```bash
# 1. prove the version boundary
echo "$VERSIONS" | jq --arg d "$TARGET" '.[] | select(.validFrom == $d)'

# 2. fetch the changed section
cdx-cli get "cdx://doc/${NEW}/text?part=${SECTION}"

# 3. if needed, compare with the previous version
cdx-cli get "cdx://doc/${OLD}/text?part=${SECTION}"
```

Keep the explanation tied to the changed provision, not to generic summaries of the amending act.

## 6. Verify a Claimed Amendment

When the user says "did amendment X really change `§ Y`?", verify in this order:

1. resolve the law and the target date,
2. prove the version boundary from `/versions`,
3. identify the amending act from `amendmentDocIds`,
4. fetch the section text before and after the boundary,
5. answer only from the proved delta.

Do not answer only from the title of the amending act or from a relation list.

## 7. Useful `cdx-cli` Introspection Commands

These are useful when you need to confirm endpoint behavior without leaving the terminal:

```bash
cdx-cli search CR --help
cdx-cli schema meta CR
cdx-cli schema versions
cdx-cli schema text
cdx-cli schema related
cdx-cli schema related/counts
```

## Common Errors to Avoid

- Starting with broad search when the law number is already known.
- Reading `/text` before proving the relevant boundary with `/versions`.
- Using the resolved current version id for relation queries instead of the base CR id.
- Using only relation lists when `amendmentDocIds` already identify the amending acts directly.
- Explaining impact from an amending act title without comparing the actual provision text.
- Continuing to diff when `/versions` already proves that no change exists on the target date.

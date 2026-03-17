# Tracking Amendments, Effective Dates, and Legal Changes

Operating note:
- Start with the first task-serving `cdx` request.
- Do not run `which cdx`, inspect env vars, call bare `cdx`, or call `cdx --help` as a preflight step.
- If the first real `cdx` request fails, then diagnostics are allowed.

Use this workflow for high-frequency prompts about:
- when a law was last amended,
- whether a change is already effective,
- what changes from a target date,
- how wording differs before and after a date,
- what a change means for a concrete actor,
- whether a claimed amendment or prior answer is correct,
- whether there is only an enacted change or also a prepared bill.

## Quick Routing by Prompt Type

- `kdy se naposledy novelizoval ...`, `jaká je poslední novela ...` -> `/versions` first, then `amendmentDocIds`, then `related` only if needed.
- `co se mění od 1.1.2026`, `jaké změny budou od ...`, `chystají se změny ...` -> `/versions` first; stop early if no boundary exists.
- `porovnej znění ... před/po datu` -> `/versions`, then normalized diff.
- `co to znamená pro zaměstnavatele / obec / školu / fond / úřad` -> detect the change first, then explain the practical effect.
- `uprav směrnici / smlouvu / vzor podle aktuální legislativy` -> verify the current wording first, then rewrite.
- `opravdu novela X změnila § Y`, `ověř to`, `zkontroluj` -> reproduce the version boundary or lack of it before answering.

## Workflow 1: When Was This Last Amended / Is It Effective?

If the law number is known, skip search:

```bash
LAW_NUM="262"
LAW_YEAR="2006"
TODAY="2026-03-17"

cdx "cdx://cz_law/${LAW_NUM}/${LAW_YEAR}/versions" | jq '.'
```

Get the current version and the nearest future boundary:

```bash
VERSIONS=$(cdx "cdx://cz_law/${LAW_NUM}/${LAW_YEAR}/versions")

printf '%s' "$VERSIONS" | jq -r --arg d "$TODAY" '
  {
    currentVersion: ([.[] | select(.validFrom <= $d and (.validTo == null or .validTo >= $d))] | first),
    nextEffectiveVersion: ([.[] | select(.validFrom > $d)] | last)
  }'
```

Use the current version row for:
- the currently effective text,
- the most recent already-effective amendment boundary (`validFrom`),
- the first provenance shortcut via `amendmentDocIds`.

Use `nextEffectiveVersion` to answer:
- whether a future enacted change already exists,
- from which date it becomes effective.

If `amendmentDocIds` are present, use them before broader relation queries.

## Workflow 2: What Changes From Date X?

Detect the boundary first. Do not fetch `/text` until a boundary is confirmed.

```bash
LAW_NUM="586"
LAW_YEAR="1992"
TARGET_DATE="2026-04-01"

cdx "cdx://cz_law/${LAW_NUM}/${LAW_YEAR}/versions" | \
  jq -r --arg d "$TARGET_DATE" '
    to_entries as $rows
    | $rows[]
    | select(.value.validFrom == $d)
    | {
        newVersionId: .value.versionId,
        oldVersionId: ($rows[.key + 1].value.versionId // empty),
        amendmentDocIds: .value.amendmentDocIds
      }'
```

If nothing matches `validFrom == TARGET_DATE`, there is no legal-text change starting on that date. Stop and answer that directly.

If a boundary exists:
1. Use `amendmentDocIds` as the first provenance shortcut.
2. Then call `/related/counts`.
3. Then call `/related?type=PASIVNI_NOVELA&sort=date&order=desc&limit=10` if you need confirmation or broader amendment history.

## Workflow 3: Compare Wording Before and After Date X

Once `oldVersionId` and `newVersionId` are known:

```bash
OLD="CR10_2026_01_01"
NEW="CR10_2026_04_01"

cdx "cdx://doc/${OLD}/text" > /tmp/old.txt
cdx "cdx://doc/${NEW}/text" > /tmp/new.txt

sed -E 's#cdx://doc/[A-Z0-9_]+/text\\?part=[A-Za-z0-9_]+#INTERNAL_LINK#g' /tmp/old.txt > /tmp/old.norm
sed -E 's#cdx://doc/[A-Z0-9_]+/text\\?part=[A-Za-z0-9_]+#INTERNAL_LINK#g' /tmp/new.txt > /tmp/new.norm

diff -u /tmp/old.norm /tmp/new.norm | sed -n '1,200p'
```

Interpret the diff only after self-link normalization. Otherwise internal anchors create noisy false positives.

## Workflow 4: Re-Diff Only the Changed Section

If the full diff is too noisy, narrow it to the changed `part`.

For a quick CR preview:

```bash
SECTION="paragraf38h"
cdx "cdx://doc/${OLD}/text?part=${SECTION}" > /tmp/section-old.txt
cdx "cdx://doc/${NEW}/text?part=${SECTION}" > /tmp/section-new.txt
diff -u /tmp/section-old.txt /tmp/section-new.txt
```

For deterministic extraction, use marker-based extraction from the full text:

```bash
cdx "cdx://doc/${NEW}/text" | \
  awk -v section="${SECTION}" '
    $0 == "[?part=" section "]" {capture=1}
    capture {
      if ($0 ~ /^\[\?part=/ && $0 != "[?part=" section "]") exit
      print
    }
  '
```

Summarize from the narrowed section diff, not from the raw full diff.

## Workflow 5: Explain the Practical Impact for an Actor

Prompt shapes:
- `co to znamená pro zaměstnavatele`
- `jaký dopad to má na obec / školu / fond / úřad`

Default order:
1. Detect the change with `/versions`.
2. Identify the amending act from `amendmentDocIds` or `PASIVNI_NOVELA`.
3. Diff the affected version or provision.
4. Explain the effect for the named actor.

Answer structure:
1. What changed.
2. From when.
3. Which provisions changed.
4. What the actor must do differently, if anything.
5. Whether action is needed now or only from a future date.

## Workflow 6: Update a Clause, Template, or Internal Rule

Prompt shapes:
- `uprav směrnici / smlouvu / vzor podle aktuální legislativy`
- `zaktualizuj text, aby byl v souladu s legislativou od 1.1.2025`

Default order:
1. Resolve the cited law and target date.
2. Verify the currently effective provision with `/versions` and `/text`.
3. If the user text appears stale, compare old/new wording.
4. Rewrite against the current law, not against memory.

Do not draft first and verify later.

## Workflow 7: Verify a Claimed Amendment or Prior Answer

Prompt shapes:
- `opravdu novela X změnila § Y`
- `ověř to`
- `neexistuje žádná novelizace tohoto zákona?`

Default order:
1. Restate the claim being checked.
2. Reproduce the version boundary from `/versions`.
3. Confirm or refute provenance from `amendmentDocIds` and `PASIVNI_NOVELA`.
4. Quote or summarize the changed provision only after the boundary is proven.

Use this workflow to verify previous assistant output too.

## Workflow 8: Prepared Bill or Only Enacted Change?

First distinguish enacted text from legislative preparation.

1. Check `/versions` first.
2. If `/versions` already shows a future effective version, answer that an enacted change exists and state its effective date.
3. If the user asks whether something is only prepared, use CODEXIS sources for draft-related materials next:
   - `DUVODOVA_ZPRAVA` on a known amendment document,
   - targeted search for `návrh zákona`, `připravovaná novela`, or the bill topic.
4. Only if live legislative-process status is still unclear, use official Parliament / government sources as a fallback.

Do not confuse:
- currently effective law,
- enacted but future-effective amendment,
- merely prepared bill.

## Domain Starter Sets for Broad “What Changes” Queries

Use starter sets only for broad domain sweeps. If the user names a statute, go directly to that statute.

Labour / employment / social:
- `262/2006`
- `435/2004`
- `582/1991`
- `589/1992`
- `187/2006`
- `309/2006`

Tax / DPH / ZDP:
- `586/1992`
- `235/2004`
- `280/2009`
- `338/1992`
- `353/2003`
- `16/1993`
- `593/1992`
- `565/1990`

Construction / planning:
- `283/2021`
- `183/2006`
- `500/2006`
- `501/2006`

Civil / property / contracts:
- `89/2012`
- `99/1963`
- `292/2013`

Public administration / procurement:
- `134/2016`
- `500/2004`
- `128/2000`

For domain sweeps:
1. Run `/versions` across the starter set.
2. Keep only laws with a target-date boundary.
3. Run `relations` and `diff` only on positive hits.

## Pitfalls

- Do not start change questions with broad keyword search when the law is already known.
- Do not fetch `/text` before `/versions` proves a boundary.
- Do not guess version IDs; always read them from the API.
- Use base IDs for `versions` and amendment provenance; use version IDs for `text` and `toc`.
- Normalize self-links before interpreting a diff.
- Distinguish enacted future changes from merely prepared bills.

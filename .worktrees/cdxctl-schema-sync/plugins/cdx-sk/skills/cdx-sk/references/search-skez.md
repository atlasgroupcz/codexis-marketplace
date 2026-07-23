# SKEZ Search Filter Values

Slovak legislation from e-Zbierka (Zbierka zakonov SR). ~26,000 regulations with multiple timecut versions.

## query
Free-text search across paragraph text, titles, and headings. Optional.

## docNumber
Exact match on document number. Include suffix.
- Examples: `40/1964 Zb.`, `311/2001 Z. z.`

## Citation link text
User-facing links use the compact law citation `ČÍSLO NÁZOV` built from `docNumber` + `title`
(e.g. `311/2001 Z. z. Zákonník práce`); when citing a specific version, append
`, v znení účinnom od DD.MM.RRRR` from the result's `validFrom`. The collection suffix in
`docNumber` is authoritative (`Zb.` pre-1993, `Z. z.` post-1993) — never hardcode it. The API's
resolved source titles already use this format — reuse them verbatim where shown. See
SKILL.md → Document Titles.

## typ
Document type (exact match, case-sensitive with diacritics):
- `Zakon`
- `Vyhlaska`
- `Oznamenie`
- `Opatrenie`
- `Nariadenie vlady`
- `Ustavny zakon`

## validAt
Date (YYYY-MM-DD). Filters to versions valid at this date (validFrom <= date AND validTo >= date).

## issuedFrom / issuedTo
Date range (YYYY-MM-DD). Filters on declared date (inclusive).

## sort (query param)
- `relevance` (default)
- `title`
- `date`

## order (query param)
- `asc`
- `desc` (default)

## offset
Integer, 0-based. Default: 0.

## limit
Integer, 1-100. Default: 20.

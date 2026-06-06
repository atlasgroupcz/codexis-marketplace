## Judikatura Workflow

For case-law or judikatura research, prefer a dedicated decision-finding workflow over generic legal search.
Some phrases meaning case law research - `najdi judikaturu`, `najdi rozsudek`, `existuje rozhodnutí k ...`, `jak rozhodl Nejvyšší soud / Ústavní soud / vrchní soud`, atp.

1. Start in `JD`, not `ALL`, when the user is asking for Czech court decisions.
2. Use `cdx-cli search JD --help` if you need to confirm available filters.
3. When doing case law research ALWAYS delegate JD candidate discovery to subagent if available. Instruct subagent to deliver meaningful snippets and `https://` document links (the resolved URL from the source block).
4. Wait for agents first, then verify. Do not run your own search in parallel with the agent; it will likely be redundant and less effective than the agent's targeted search.
5. After collection promising candidates use subagent to verify the actual content of the decision text and the metadata. Pay attention to:
   - what conduct actually was at issue in the case,
   - whether the cited statement is ratio, context, or only quoted argument,
   - whether the higher court affirmed, reversed, remanded, or decided only a procedural issue.
6. Do not confuse engagement metrics, quoted third-party comments, or procedural background with the punishable conduct or the holding.
7. If `JD` does not surface an exact match, use `ALL`, `COMMENT`, or `LT` only to identify candidate references, then verify back in `JD` before citing.
8. If no exact higher-court decision is verified, say so explicitly and provide the closest verified cases instead of overstating the result.

### Extracting information from specific case law document

To get specific information from a concrete case law use subagent instead of retrieving the whole text. Instruct subagent to retrieve full text of the decision and extract the relevant information from it by first reading the metadata and then reading the text as a whole. Only when court decission text is too large for agent to process let it use tools like `rg` to find relevant passages. 

## Citation format

Every visible court-decision citation/link MUST be a compact legal citation a lawyer can read at a glance — court abbreviation FIRST. Never use a truncated generic title (e.g. "Usnesení - Usnesení Ústavního soudu ze…") as the primary link text.

```
COURT - ČÍSLO JEDNACÍ / SPISOVÁ ZNAČKA - DD.MM.YYYY
```

Build it from JD metadata (search result or `cdx-cli get cdx://doc/<id>/meta`):

| Part | Field | Rule |
|------|-------|------|
| Court | `court` (full name) → abbreviation | mapping below; unknown court → keep its official short name verbatim |
| Reference | `cislaJednaci[0]`, else `spZns[0]` | číslo jednací preferred over spisová značka; if neither, use `ecli` then `sbirkoveCislo` |
| Date | `releaseDate` → `DD.MM.YYYY` | decision date; omit the ` - DD.MM.YYYY` segment entirely if absent — never invent or zero-pad |

Secondary metadata (`docType`, original `title`, `legalSentence`) may appear as surrounding prose or snippet, but never as the primary clickable label.

### Court-name → abbreviation

| `court` value | Abbrev |
|---------------|--------|
| Ústavní soud | ÚS |
| Nejvyšší soud | NS |
| Nejvyšší správní soud | NSS |
| Vrchní soud v Praze / v Olomouci | VS Praha / VS Olomouc |
| Krajský soud v `<městě>` | KS `<město>` (KS Praha, KS Brno, KS Ostrava, …) |
| Městský soud v Praze | MS Praha |
| Okresní soud v/ve `<městě>` | OS `<město>` (OS Brno, OS Ostrava, …) |
| Obvodní soud pro Prahu `N` | OS Praha `N` |

City is taken from the court name (or the `city` field).

For `ES` (EU case law) the same shape applies with EU court abbreviations: the Court of Justice of the EU is `SDEU` (Soudní dvůr EU) and the European Court of Human Rights is `ESLP`. The reference is the case number, falling back to `ecli` then `celex` — e.g. `SDEU - C-123/24 - 04.10.2024`.

### Worked examples

- `ÚS - Pl. ÚS 1/26 - 20.05.2026`
- `NSS - 1 As 123/2026 - 13.05.2026`
- `NS - 25 Cdo 1234/2026 - 01.06.2026`
- `KS Praha - 12 Co 45/2026 - 01.06.2026`
- `OS Brno - 10 C 123/2026 - 01.06.2026`

As a Markdown link the citation is the link text over the resolved `https://` URL, e.g. `[ÚS - Pl. ÚS 1/26 - 20.05.2026](https://…)`.

### Graceful degradation

- Court abbreviation is always present and first; never dropped.
- Missing date → omit only the date segment, keep `COURT - REF`.
- Missing sp. zn. and č. j. → use `ecli` or `sbirkoveCislo` as the reference.
- Only if all structured metadata is missing, fall back to `title` → `docNumber` → short descriptive fallback. Never hide court/date/reference data that is actually available.

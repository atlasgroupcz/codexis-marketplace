---
uuid: 272cbab6-4deb-4b1b-8595-279b6c6923d9
name: codexis
description: This skill should be invoked whenever user needs Czech or European legal research, especially legislation changes, case law or judikatura lookup, legal verification, or linked legal analysis. Use CODEXIS as the primary research surface and add complementary official sources only for targeted institutional context when needed. Invoke `cdx-cli` command in parallel (it exists) to see up-to-date CLI interface.
version: 2.1.0
i18n:
  cs:
    displayName: "CODEXIS — Legislativa ČR"
    summary: "Vyhledávání v české a evropské legislativě s propojenou judikaturou a odbornou literaturou."
  en:
    displayName: "CODEXIS — Czech Legislation"
    summary: "Search Czech and EU legislation with linked case law and legal literature."
  sk:
    displayName: "CODEXIS — Česká legislatíva"
    summary: "Vyhľadávanie v českej a európskej legislatíve s prepojenou judikatúrou a odbornou literatúrou."
---

# CODEXIS Legal Database CLI

Use CODEXIS first for Czech and EU legal work. Route fast with `cdx-cli`.

## Operating Assumptions

- Use `cdx-cli` for all CODEXIS requests.
- Canonical forms:
  - search: `cdx-cli search JD --query "náhrada škody" --court "Nejvyšší soud" --limit 5`
  - fetch: `cdx-cli get cdx://doc/JD402788/text`
  - schema: `cdx-cli schema versions` or `cdx-cli schema meta CR`
- Assume `cdx-cli` is installed, authenticated, and operational.
- Do **not** run `which cdx-cli`, inspect environment variables, or call bare `cdx-cli` as a preflight step.
- Start with the first real request. Diagnose only if it fails or the user asks for diagnostics.
- Prefer flag-based `search` commands. The optional `[JSON_PAYLOAD]` argument is a fallback, not the default.
- Use only the documented `search`, `get`, and `schema` subcommands.
- document retrieval: `cdx-cli get cdx://...`

## Research Strategy

1. Start in CODEXIS.
2. Prefer source-specific search (`CR`, `EU`, `JD`, `ES`, `COMMENT`, `LT`, `VS`) over `ALL` whenever the likely source is known.
3. Use `ALL` only to orient source selection, then rerun in the specific source before citing or extracting.
4. Add non-CODEXIS sources for targeted official context.

### External Sources

Use external sources only for official institutional material, current operational info, or original-language / multilingual EU text. Use only official institutional sources. Do not use blogs, law firms, commercial explainers, forums, or generic search hits when CODEXIS can answer.

## Data Sources

Brief source glossary, aligned with `cdx-cli --help`:
All CODEXIS documents are in Czech.

| Code | Name | What you find there |
|------|------|---------------------|
| `CR` | Czech Legislation | Czech laws, decrees, regulations, and municipal documents |
| `SK` | Slovak Legislation | Slovak laws and regulations |
| `JD` | Czech Case Law | Judicial decisions from Czech courts |
| `ES` | EU Court Decisions | EU Court of Justice and ECHR rulings |
| `EU` | EU Legislation | EU regulations, directives, and decisions |
| `LT` | Legal Literature | Legal publications and articles |
| `VS` | Contract Templates | Contract specimens and templates |
| `COMMENT` | Legal Commentaries | LIBERIS legal commentaries on Czech legislation |
| `ALL` | Global Search | Exploratory search across all sources; use only for orientation |

## User-Facing Output Rules

- Use only `cdx://doc/` links: `[title](cdx://doc/{docId})` or `[title](cdx://doc/{docId}#elementId)`. Never expose raw IDs, source codes, relation enums, resolved HTTP URLs, or API suffixes like `/text`, `/meta`, `/toc` as user has no way to access it.
- Use the search-result title as link text, stripping `<mark>` tags. For `JD`, include the court name in the link text. If title is unavailable, use `docNumber` or a short descriptive fallback, raw ID as last resort.
- For relation labels, use the `name` field from `/related/counts`, never the `type` enum.
- When citing a legal section, make the section itself the clickable reference instead of splitting title and link.

## Common workflows
- deep-dive Czech LAW change assessment workflow: `references/czech-law-change-assessment.md`
- for caselaw/judikatura research see `references/judikatura.md`

## Governing Regime First

For questions that can plausibly fall under multiple legal regimes, determine the primary regime before broad extraction.

- Start with the rule that could make a special regime applicable and only then pull general or subsidiary rules.
- If the user names a statute, paragraph, or regime, treat it as the first routing candidate, not as a side note.
- For liability questions, do not default to general Civil Code delict provisions until you have ruled out a more specific regime in sector legislation, the Labour Code, procedural law, or another special statute.
- If multiple regimes remain plausible after a narrow first pass, state which regime is primary and why, and treat the others as subsidiary.
- If the facts are insufficient to choose with confidence, ask one narrow clarifying question before running a wide search.

## Hard Rules

- If the law number is known, do not start with broad keyword search.
- For Czech laws named by number and year, prefer `cdx-cli get cdx://cz_law/{lawNumber}/{lawYear}/...` and go straight to `/versions`, `/meta`, `/text`, or `/toc` as needed.
- For change questions, start with `/versions`, not `/text`, broad search, or web lookup.
- Stop early if `/versions` shows no boundary at the target date. Answer that directly.
- Use base IDs for `/versions` and amendment provenance. Use version IDs for `/text` and `/toc`.
- For broad domain sweeps, use a curated statute list plus `/versions`; do not diff every law in the set.
- Do not run broad fallback searches after a decisive `/versions` sweep unless ambiguity remains.
- Do not jump to the web when CODEXIS already answers the legal question.
- Treat `ALL` as source-selection help, not as a final citation target.
- For source-specific search, use `cdx-cli search <SOURCE>` with explicit flags first. Use raw JSON only when a needed filter is unavailable as a flag or when piping a saved payload via stdin.
- When filters are unclear, use `cdx-cli search <SOURCE> --help` or `--with-facets`; do not rely on separate datasource cheat sheets.
- For section extraction, prefer marker-based extraction from `/text`; use TOC to resolve the correct element when needed.
- For specific paragraphs, fetch `/toc`, resolve the paragraph `elementId` (for example `paragraf19c`), then use `/text?part=<elementId>`; repeat `part=` when needed.
- Do not guess `docId` values. Extract them from API responses.

## Proactive Legal Reference Enrichment

Whenever legal references appear in user text, tool output, or extracts, resolve them in CODEXIS and present them with `cdx://doc/` links.

- Do not present raw legal references without attempting lookup first.
- If a reference cannot be found in CODEXIS, keep it as plain text instead of omitting it.
- When listing laws with paragraphs, use one bullet per law and keep paragraph links inline, comma-separated.
- Collapse truly consecutive paragraphs into a single range, for example `§ 312–332`.

## Short Best Practices

- Base conclusions primarily on legislation and case law; use commentary and literature secondarily.
- Filter early with `--current`, `--valid-at`, `--type`, source-specific date flags, `--with-facets` when discovery is needed, and low `--limit` values.
- Fetch large text once, extract locally, and validate the heading before relying on it.
- Use `jq` to filter and shape responses instead of extra requests.

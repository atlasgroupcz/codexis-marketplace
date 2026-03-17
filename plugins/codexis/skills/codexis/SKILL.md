---
name: codexis
description: This skill should be invoked whenever user needs law or legal information, czech or european. Use CODEXIS as the primary research surface and add complementary official sources only for targeted institutional context when needed.
version: 1.6.0
---

# CODEXIS Legal Database API

Use CODEXIS as the default research surface for Czech and EU legal work. Keep this file short in practice: route first, then open only the specific reference file you need.

## Operating Assumptions

- Use the `cdx` CLI for all CODEXIS requests. It is opinionated: it runs silently by default, and `-d` implies `POST` plus `Content-Type: application/json` unless you override them. Use `cdx://` URLs such as `cdx://search/CR` and `cdx://doc/CR26785_2026_01_01/text`.
- Assume `cdx` is installed, available on `PATH`, authenticated, and operational.
- Do **not** run `which cdx`, inspect environment variables, call bare `cdx`, or call `cdx --help` as a preflight step. The CLI may not expose a help screen, and help output is not a capability check.
- Start with the first task-serving `cdx` request. Do diagnostics only if that real request fails or the user explicitly asks for diagnostics.

## Research Priority

1. Start in CODEXIS and exhaust the relevant source there first.
2. Prefer source-specific searches (`CR`, `EU`, `JD`, `ES`, `COMMENT`, `LT`, `VS`) over `ALL` whenever the likely source is known.
3. Use `ALL` only for orientation when the source is genuinely unclear, then rerun the search in the specific source before citing or extracting.
4. Add non-CODEXIS sources only for targeted official context.

### External Source Restrictions

Use complementary official sources only when at least one of these is true:

- the task calls for an official institutional page, form, press release, registry, or operational guidance,
- the user explicitly asks for an official institutional source,
- the answer depends on very current administrative or operational information,
- an official multilingual or original-language EU text is needed in addition to the CODEXIS Czech version.

If you need external sources, use only official ones: statutes and court collections, ministries and regulators, tax and customs authorities, courts, Parliament, government portals, EUR-Lex, Curia, europa.eu, and other official EU institution sites.

Do not use blogs, law-firm articles, commercial explainers, discussion boards, or generic search hits when CODEXIS can answer the question.

## User-Facing Output Rules

- Use markdown links with the `cdx://doc/` scheme only: `[title](cdx://doc/{docId})` or `[title](cdx://doc/{docId}#elementId)`.
- Never expose raw document IDs, source codes, relation enums, or resolved HTTP URLs in user-visible text.
- Never use API paths in user-facing links. `/text`, `/meta`, `/toc`, and similar suffixes are for API calls only.
- Use the title as link text. For `CR` and `SK`, use `title` from search results and strip `<mark>` tags. For `JD`, include the court name inside the link text. If title is unavailable, use `docNumber` or a short descriptive fallback, never the raw ID.
- For relation labels, use the `name` field from `/related/counts`, never the `type` enum.
- When citing a legal section, make the section itself the clickable reference instead of splitting title and link.

## Core Routing

Use the smallest reference file that matches the task.

- Changes, effective dates, amendment tracking, wording diffs, practical impact, or verification of a claimed amendment:
  Read `references/law-changes.md`.
- Czech legislation structure, `cz_law` endpoints, `CR` versions, TOC, or section extraction:
  Read `references/czech-legislature.md`.
- EU legislation structure and extraction:
  Read `references/eu-legislature.md`.
- Related documents, relation counts, or relation filtering:
  Read `references/relations.md`.
- Source-specific search syntax and filters:
  Read the matching `references/search-*.md`.

High-frequency prompt routing:

- `kdy se naposledy novelizoval ...`, `jaká je poslední novela ...`
  Start with `/versions`; identify the relevant boundary and use `amendmentDocIds` first.
- `co se mění od 1.1.2026`, `jaké změny budou od ...`, `chystají se změny ...`
  Start with `/versions`; only diff laws that actually have a target-date boundary.
- `porovnej znění ... před/po datu`
  Use `/versions` to get exact old and new version IDs, then diff.
- `co to znamená pro zaměstnavatele / obec / školu / fond / úřad`
  Detect the change first, then extract only the changed provisions and explain the practical effect.
- `uprav text / směrnici / smlouvu / vzor podle aktuální legislativy`
  Verify the current wording first, then draft against the current law.
- `opravdu novela X změnila § Y`, `ověř to`, `zkontroluj`
  Reproduce the version boundary or the lack of it before answering.

## Hard Rules

- If the law number is known, do not start with broad keyword search.
- For Czech laws named by number and year, prefer `cdx://cz_law/{lawNumber}/{lawYear}/...` and go straight to `/versions`, `/meta`, `/text`, or `/toc` as appropriate.
- For change questions, start with `/versions`. Do not start amendment tracking with `/text`, broad search, or web lookup.
- Stop early if `/versions` shows no boundary at the target date. Answer that directly.
- Use base IDs for `/versions` and amendment provenance. Use version IDs for `/text` and `/toc`.
- For broad domain sweeps, use a curated statute list plus `/versions`; do not diff every law in the set.
- Do not run broad fallback searches after a decisive starter-set `/versions` sweep unless the result is still ambiguous.
- Do not jump to the web when CODEXIS already answers the legal question.
- Treat `search/ALL` results as hints for source selection, not as final citation targets.
- For section extraction, prefer marker-based extraction from `/text`; use TOC to resolve the correct element when needed.
- Do not guess `docId` values. Extract them from API responses.

## Proactive Legal Reference Enrichment

Whenever legal references appear in any context, including tool output, user-pasted text, or extracted documents, resolve them in CODEXIS and present them with `cdx://doc/` links.

- Do not present raw legal references without attempting lookup first.
- If a reference cannot be found in CODEXIS, keep it as plain text instead of omitting it.
- When listing laws with paragraphs, use one bullet per law and keep paragraph links inline, comma-separated.
- Collapse truly consecutive paragraphs into a single range, for example `§ 312–332`.

## Reference Files

- `references/law-changes.md` - amendment tracking, effective dates, version boundaries, diffs, practical impact, verification
- `references/czech-legislature.md` - Czech legislation structure, versions, TOC, extraction, `cz_law` shortcuts
- `references/eu-legislature.md` - EU legislation structure and extraction
- `references/relations.md` - related documents and relation counts
- `references/search-cr.md` - Czech legislation search
- `references/search-jd.md` - Czech case law search
- `references/search-eu.md` - EU legislation search
- `references/search-sk.md` - Slovak legislation search
- `references/search-comment.md` - legal commentary search
- `references/search-vs.md` - contract template search
- `references/search-lt.md` - legal literature search
- `references/search-es.md` - EU Court decision search
- `references/search-all.md` - exploratory cross-source search only

## Short Best Practices

- Base conclusions primarily on legislation and case law; use commentary and literature as secondary support.
- Filter early with `validNow`, `validAt`, `typ`, and low `limit` values where supported.
- Fetch large text once and extract locally instead of repeating API calls.
- Validate that an extracted section matches the expected heading before relying on it.
- Use `jq` for filtering and shaping API responses instead of extra requests.

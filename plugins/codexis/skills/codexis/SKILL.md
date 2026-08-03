---
uuid: 272cbab6-4deb-4b1b-8595-279b6c6923d9
name: codexis
icon: icon.svg
description: This skill should be invoked whenever user needs Czech or European legal research - legislation changes, case law / judikatura lookup, legal commentaries, professional literature, contract templates, or linked legal analysis. Use CODEXIS as the primary research surface and add complementary official sources only for targeted institutional context when needed. Invoke `cdx-cli` command in parallel (it exists) to see up-to-date CLI interface.
version: 2.3.0
jurisdictions: [CZ, EU]
jurisdiction-required: true
i18n:
  cs:
    displayName: "CODEXIS - Právní databáze"
    summary: "Česká, slovenská a evropská legislativa, česká a evropská judikatura, odborné komentáře, právní literatura a vzory smluv."
    examplePrompts:
      - "Najdi judikaturu Nejvyššího soudu k náhradě škody a shrň závěry."
      - "Ukaž, jak se v čase měnil § 52 zákoníku práce a od kdy platí aktuální znění."
      - "Najdi vzor smlouvy o dílo a shrň, na co si při jejím uzavírání dát pozor."
  en:
    displayName: "CODEXIS - Legal Database"
    summary: "Czech, Slovak and EU legislation, Czech and European case law, legal commentaries, literature and contract templates."
    examplePrompts:
      - "Find Supreme Court case law on damages and summarize the conclusions."
      - "Show how § 52 of the Czech Labour Code has changed over time and since when the current wording applies."
      - "Find a contract-for-work template and summarize what to watch out for when signing it."
  sk:
    displayName: "CODEXIS - Právna databáza"
    summary: "Česká, slovenská a európska legislatíva, česká a európska judikatúra, odborné komentáre, právna literatúra a vzory zmlúv."
    examplePrompts:
      - "Nájdi judikatúru českého Najvyššieho súdu k náhrade škody a zhrň závery."
      - "Ukáž, ako sa v čase menil § 52 českého zákonníka práce a odkedy platí aktuálne znenie."
      - "Nájdi vzor zmluvy o dielo a zhrň, na čo si pri jej uzatváraní dať pozor."
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
4. Decide HOW to run the retrieval before you run it. A real legal question is almost never one
   query: the same rule hides behind several formulations, and the answer needs the best few
   documents out of dozens. Whenever the honest plan is three or more searches, or the same fetch
   repeated over a list of documents, run it as ONE `script` program - not as a chain of `shell`
   calls. Do this even when the user asked a plain question and said nothing about how to search;
   deciding the retrieval shape is your job, not theirs. See "Broad retrieval goes through
   `script`" below.
5. Add non-CODEXIS sources for targeted official context.

### External Sources

Use external sources only for official institutional material, current operational info, or original-language / multilingual EU text. Use only official institutional sources. Do not use blogs, law firms, commercial explainers, forums, or generic search hits when CODEXIS can answer.

### Broad retrieval goes through `script`

Run wide retrieval as ONE `script` call, not as a sequence of `shell` calls and not as one
subagent per query. The program fans the searches out, merges and dedupes them, opens only the
best candidates, and returns an evidence package; the raw hits never reach the context.

Reach for `script` when any of these holds:

- three or more queries are needed (reworded phrasings, or several of CR/JD/ES/EU/COMMENT/LT),
- results from several queries must be merged and deduped by `docId`,
- full text is needed for a handful of candidates out of many,
- relations must be chased across several documents (`/related`),
- a provision's wording at a date must be checked across several acts (`/versions`),
- one act's `/versions`, `/toc` and several `/text?part=` are all wanted — fetch them together, not one turn each.

One query, one `get`, one check stays a plain `shell` call.

Rules for a retrieval script:

1. Searches under `parallel`, document opening under `pipeline` - one failure then costs one
   branch, not the run.
2. `--limit` at most 50 (the CLI rejects more). No `ALL`: send more queries into specific sources.
3. Dedupe by `docId`; rank by how many queries found the same document.
4. Take only `docId` from a search hit. Every citation field comes from `/meta`.
5. Open full text only for the top candidates, write it to a file and read it with `node:fs` - a
   decision easily exceeds the shell tool's output cap.
6. Return per document: `docId`, court, case number, date, one to three verbatim passages, and
   the query that found it. Never return whole texts or whole API responses.
7. The script does no legal assessment. Attribution - what the court held versus what a party
   argued, see `references/judikatura.md` - is judged by you or by a verifying subagent, over the
   passages the script returned.

Worked programs for all six shapes: `references/script-research.md`.

Build user-facing links from the `【src_xxx】 url: https://…` blocks attached to the tool call,
never from `docId`.

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

- User-facing document links MUST use the resolved `https://` URL from the source block. Each cited document is appended to tool output as a `【src_xxx】 url: https://…` block - use that `https://` URL as the link target: `[title](https://…)` or `[title](https://…#elementId)` (append the same `#elementId` anchor you would have used). The `cdx://` scheme is ONLY an addressing scheme for `cdx-cli get cdx://...` fetch arguments - never put a `cdx://` link in output shown to the user. Never expose raw IDs, source codes, relation enums, or API suffixes like `/text`, `/meta`, `/toc` as user has no way to access it.
- Use the search-result title as link text, stripping `<mark>` tags. If title is unavailable, use `docNumber` or a short descriptive fallback, raw ID as last resort.
- For court decisions (`JD`, `ES`), the visible link text MUST be a compact legal citation, not the raw or generic title: `COURT - Č. J. / SP. ZN. - DD.MM.YYYY` (e.g. `ÚS - Pl. ÚS 1/26 - 20.05.2026`, `NSS - 1 As 123/2026 - 13.05.2026`). Compose it from metadata (search result or `/meta`): map `court` to its abbreviation per the table in `references/judikatura.md` (Czech courts plus the EU `ES` sources SDEU/ESLP), take the reference from `cislaJednaci[0]` (else `spZns[0]`, then `ecli`/`sbirkoveCislo`), and append ` - DD.MM.YYYY` from `releaseDate` only when a date exists - never invent one. The generic title (`docType` + long court name) may appear only as secondary text, never as the primary label; for an unrecognised court keep its official short name verbatim.
- For relation labels, use the `name` field from `/related/counts`, never the `type` enum.
- When citing a legal section, make the section itself the clickable reference instead of splitting title and link.

## Common workflows
- deep-dive Czech LAW change assessment workflow: `references/czech-law-change-assessment.md`
- for caselaw/judikatura research see `references/judikatura.md`
- for retrieval programs driving `cdx-cli` see `references/script-research.md`

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
- For EU acts named by type, year and number, prefer `cdx-cli get cdx://eu_law/{R|L|D}/{year}/{number}/...` (`/meta`, `/versions`) over `search EU`; the type letter is `R` regulation, `L` directive, `D` decision. Read the year and number off the citation - do not compute a CELEX, CODEXIS composes it. Mind only which side of the pair is the year: EU numbering flipped in 2015, so Nařízení (EU) 2016/679 is `R/2016/679` while Nařízení (ES) č. 988/2009 is `R/2009/988`. With a CELEX already in hand - a EUR-Lex link the user pasted - use `cdx://eu_law/by-celex/{celex}/...` instead. A `search EU` hit needs neither route: it already carries `docId`, so fetch `cdx://doc/{docId}/...` directly. Fall back to `search EU` only when no citation coordinates are available or the lookup returns 404.
- For change questions, start with `/versions`, not `/text`, broad search, or web lookup.
- Stop early if `/versions` shows no boundary at the target date. Answer that directly.
- Use base IDs for `/versions` and amendment provenance. Use version IDs for `/text` and `/toc`.
- For broad domain sweeps, use a curated statute list plus `/versions`; do not diff every law in the set.
- Never issue the same kind of `cdx-cli` call three or more times in a row from the chat. That is a `script` program: it runs them in parallel, deduplicates, and returns only the passages. Reading the raw hits yourself spends the context the answer needs.
- This covers opening documents, not just searching. Once you have several `docId`s worth reading, fetch all of their `/text` in one `script` run and return the matching passages - a decision is long, and pulling four of them into the chat one `get` at a time is the single most expensive thing you can do with the context.
- Do not run broad fallback searches after a decisive `/versions` sweep unless ambiguity remains.
- Do not jump to the web when CODEXIS already answers the legal question.
- Treat `ALL` as source-selection help, not as a final citation target.
- For source-specific search, use `cdx-cli search <SOURCE>` with explicit flags first. Use raw JSON only when a needed filter is unavailable as a flag or when piping a saved payload via stdin.
- When filters are unclear, use `cdx-cli search <SOURCE> --help` or `--with-facets`; do not rely on separate datasource cheat sheets.
- For section extraction, prefer marker-based extraction from `/text`; use TOC to resolve the correct element when needed.
- For specific paragraphs, fetch `/toc`, resolve the paragraph `elementId` (for example `paragraf19c`), then use `/text?part=<elementId>`; repeat `part=` when needed.
- Do not guess `docId` values. Extract them from API responses.

## Proactive Legal Reference Enrichment

Whenever legal references appear in user text, tool output, or extracts, resolve them in CODEXIS and present them with `https://` document links (the resolved URL from the source block).

- Do not present raw legal references without attempting lookup first.
- If a reference cannot be found in CODEXIS, keep it as plain text instead of omitting it.
- When listing laws with paragraphs, use one bullet per law and keep paragraph links inline, comma-separated.
- Collapse truly consecutive paragraphs into a single range, for example `§ 312-332`.

## Short Best Practices

- Base conclusions primarily on legislation and case law; use commentary and literature secondarily.
- Filter early with `--current`, `--valid-at`, `--type`, source-specific date flags, `--with-facets` when discovery is needed, and low `--limit` values.
- Fetch large text once, extract locally, and validate the heading before relying on it.
- Use `jq` to filter and shape responses instead of extra requests.
- Retrieval spanning three or more queries goes through `script`, not a chain of `shell` calls.

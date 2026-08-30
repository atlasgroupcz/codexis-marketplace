# Retrieval scripts for CODEXIS

Five programs for the `script` tool. Each one is complete: fan out, dedupe, open only what
survives, and return an evidence package instead of the haystack.

## When retrieval belongs in a program

Run wide retrieval as ONE `script` call, not as a sequence of `shell` calls and not as one
subagent per query. The program fans the searches out, merges and dedupes them, opens only the
best candidates, and returns an evidence package; the raw hits never reach the context.

Reach for `script` when any of these holds:

- three or more queries are needed (reworded phrasings, or several of CR/JD/ES/EU/COMMENT/LT),
- results from several queries must be merged and deduped by `docId`,
- full text is needed for a handful of candidates out of many,
- relations must be chased across several documents (`/related`),
- one act's `/versions`, `/toc` and several `/text?part=` are all wanted - fetch them together, not one turn each.

One query, one `get`, one check stays a plain `shell` call.

## Ground rules these examples follow

- **Search gives you `docId`. Everything you cite comes from `/meta`.** A search item is
  `{docId, …}` inside `{totalResults, results, availableFilters?}`; `court`, `spZns`,
  `cislaJednaci`, `releaseDate`, `ecli`, `sbirkoveCislo` and `derogated` are guaranteed on
  `cdx://doc/<id>/meta` under the source key (`jd`, `cr`, …). Never invent a citation from a hit.
- **Values go in `args`, not into a command string.** No shell means no quoting, no word
  splitting, and no `2>/dev/null` - stderr is not merged into what you parse.
- **A decision text is long.** Fetching four of them one `get` at a time from the chat is the most
  expensive thing you can do with the context; fetch them inside one program and return only the
  passages. If a single text is too big to hold, write it out with `node:fs` and grep it there.
- **`--limit` is 1..50.** The CLI rejects more. Send more queries instead of a bigger one.
- **Filters differ per source.** JD has `--court`, `--city`, `--type` (all repeatable). ES has
  `--type` and `--issued-from/--issued-to` but **no `--court`**. CR has `--type`, `--author`,
  `--current`, `--valid-at`. `ALL` rejects `--with-facets`.
- **Report the misses.** A candidate whose text carries no matching passage is a fact about the
  query, not something to hide.
- **Searches under `parallel`, document opening under `pipeline`.** One failure then costs one
  branch, not the run.
- **Rank by agreement.** Dedupe by `docId` and rank a document by how many queries found it.
- **Return the evidence, not the haystack.** Per document: `docId`, court, case number, date, one
  to three verbatim passages, and the query that found it. Never whole texts or whole API responses.
- **The program does no legal assessment.** Attribution - what the court held versus what a party
  argued, see `judikatura.md` - is judged over the passages the program returned, not inside it.

**Pass values as `args`, never as a quoted command string.** `shell` takes an optional `args`
array; when it is present the program is run directly, with no shell in between, so a query with
spaces, apostrophes, `$` or a semicolon arrives exactly as written and there is nothing to escape.
Interpolating a Czech legal phrase into a command string is the usual way these programs break.

A shared preamble, assumed by every example below:

    const cdx = async (...args) => {
      const out = await shell({ command: 'cdx-cli', args, timeoutSeconds: 180 })
      return out.trim()
    }
    const json = async (...args) => JSON.parse(await cdx(...args))

Only reach for a command string when you genuinely need the shell - a pipe, a redirect, `&&`.
Those are the only places `2>/dev/null` belongs; with `args` there is no shell to merge streams.

## 1. Parallel multi-source sweep

The workhorse: many phrasings, merged, ranked by agreement, opened only at the top.

    export default async function () {
      const NS = ['--court', 'Nejvyšší soud']
      const QUERIES = [
        ["JD", "moderační právo soudu náhrada škody zaměstnance", NS],
        ["JD", "snížení náhrady škody zaměstnancem zákoník práce", []],
        ["JD", "odpovědnost zaměstnance za schodek na svěřených hodnotách", NS],
        ["JD", "přiměřené snížení náhrady škody pracovněprávní vztah", ['--type', 'Rozsudek']],
        ["ES", "odpovědnost zaměstnance za škodu způsobenou zaměstnavateli", []]
      ]
      const PASSAGE = /moderac|moderač|snížen[íi] náhrady|přiměřen[éeě] snížen/i
      const TOP = 20

      const found = await parallel(QUERIES.map(([source, query, filters]) => async () => {
        const body = await json('search', source, '--query', query, ...filters, '--limit', '50')
        return { source, query, results: body.results || [] }
      }))

      const byId = new Map()
      for (const hit of found.filter(Boolean)) {
        for (const item of hit.results) {
          if (!item.docId) continue
          const seen = byId.get(item.docId) ?? { docId: item.docId, source: hit.source, queries: [] }
          seen.queries.push(hit.query)
          byId.set(item.docId, seen)
        }
      }
      const candidates = [...byId.values()]
        .sort((a, b) => b.queries.length - a.queries.length)
        .slice(0, TOP)

      const documents = await pipeline(candidates,
        async (candidate) => {
          const meta = await json('get', `cdx://doc/${candidate.docId}/meta`)
          const facts = meta.jd ?? meta.es ?? {}
          return {
            ...candidate,
            court: facts.court,
            reference: (facts.cislaJednaci ?? [])[0] ?? (facts.spZns ?? [])[0]
                       ?? facts.ecli ?? facts.sbirkoveCislo,
            date: facts.releaseDate,
            docType: facts.docType,
            derogated: facts.derogated
          }
        },
        async (doc) => {
          const text = await cdx('get', `cdx://doc/${doc.docId}/text`)
          const passages = text.split(/\n\s*\n/)
            .filter((block) => PASSAGE.test(block))
            .slice(0, 3)
            .map((block) => block.replace(/\s+/g, " ").trim().slice(0, 700))
          return passages.length ? { ...doc, passages } : null
        })

      const kept = documents.filter(Boolean)
      return {
        queries: QUERIES.map(([source, query]) => `${source}: ${query}`),
        deduped: byId.size,
        opened: candidates.length,
        documents: kept.map((doc) => ({
          docId: doc.docId, court: doc.court, reference: doc.reference, date: doc.date,
          docType: doc.docType, derogated: doc.derogated, foundBy: doc.queries, passages: doc.passages
        })),
        withoutPassage: candidates
          .filter((c) => !kept.some((k) => k.docId === c.docId))
          .map((c) => c.docId)
      }
    }

Roughly 250 raw hits are read; about twenty documents and sixty passages come back.

## 2. Citation chasing

Walk the relation graph out from decisions you already trust, and keep only what is new.

    export default async function () {
      const SEEDS = ["JD123456", "JD654321"]
      const RELATION = "SOUVISEJICI_JUDIKATURA"

      const rings = await parallel(SEEDS.map((docId) => async () => {
        const body = await json('get', `cdx://doc/${docId}/related?type=${RELATION}&limit=50`)
        return { seed: docId, related: body.results ?? [] }
      }))

      const seeds = new Set(SEEDS)
      const byId = new Map()
      for (const ring of rings.filter(Boolean)) {
        for (const item of ring.related) {
          if (!item.docId || seeds.has(item.docId)) continue
          const seen = byId.get(item.docId)
            ?? { docId: item.docId, source: item.source, title: item.title, citedBy: [] }
          seen.citedBy.push(ring.seed)
          byId.set(item.docId, seen)
        }
      }

      const shared = [...byId.values()].sort((a, b) => b.citedBy.length - a.citedBy.length).slice(0, 25)
      const enriched = await pipeline(shared, async (candidate) => {
        const meta = await json('get', `cdx://doc/${candidate.docId}/meta`)
        const facts = meta.jd ?? meta.es ?? {}
        return {
          ...candidate,
          court: facts.court,
          reference: (facts.cislaJednaci ?? [])[0] ?? (facts.spZns ?? [])[0] ?? facts.ecli,
          date: facts.releaseDate
        }
      })

      return { seeds: SEEDS, relation: RELATION, related: enriched.filter(Boolean) }
    }

Anything cited by more than one seed is the line of case law worth reading; a single citation
is a lead, not authority.

## 3. Contrarian search

Two query sets, two buckets. Never hand the synthesis only the confirming half.

    export default async function () {
      const SUPPORTING = [
        "jednatel odpovídá za neprověření obchodního partnera",
        "porušení péče řádného hospodáře bonita smluvního partnera"
      ]
      const OPPOSING = [
        "jednatel neodpovídá úpadek obchodního partnera",
        "podnikatelské riziko bez odpovědnosti jednatele",
        "pravidlo podnikatelského úsudku insolvence partnera",
        "nepředvídatelný úpadek smluvního partnera"
      ]

      const sweep = (queries) => parallel(queries.map((query) => async () => {
        const body = await json('search', 'JD', '--query', query, '--court', 'Nejvyšší soud', '--limit', '50')
        return { query, ids: (body.results ?? []).map((r) => r.docId).filter(Boolean) }
      }))

      const [supporting, opposing] = await Promise.all([sweep(SUPPORTING), sweep(OPPOSING)])

      const collect = (batches) => {
        const byId = new Map()
        for (const batch of batches.filter(Boolean)) {
          for (const docId of batch.ids) {
            const seen = byId.get(docId) ?? { docId, queries: [] }
            seen.queries.push(batch.query)
            byId.set(docId, seen)
          }
        }
        return [...byId.values()].sort((a, b) => b.queries.length - a.queries.length).slice(0, 12)
      }

      const forClaim = collect(supporting)
      const againstClaim = collect(opposing)
      const bothSides = forClaim.filter((doc) => againstClaim.some((other) => other.docId === doc.docId))

      return { supporting: forClaim, opposing: againstClaim,
        appearsInBoth: bothSides.map((doc) => doc.docId) }
    }

A decision that answers both query sets is usually the one that draws the line - read it first.
Finding no opposing authority is not proof there is none; say so in the opinion.

## 4. One statute, everything about it at once

Versions, table of contents and the provisions you care about are four independent fetches. Doing
them in sequence from the chat costs four turns and pulls every byte into the context; one program
issues them together and returns the wording plus where it sits.

    export default async function () {
      const ACT = { num: "89", year: "2012" }
      const SECTIONS = ["paragraf159", "paragraf2909", "paragraf2913"]
      const base = `cdx://cz_law/${ACT.num}/${ACT.year}`

      const [meta, versions, toc] = await parallel([
        () => json('get', `${base}/meta`),
        () => json('get', `${base}/versions`),
        () => json('get', `${base}/toc`)
      ])
      if (!meta || !versions) {
        return { error: `${ACT.num}/${ACT.year} Sb. se nepodařilo načíst.` }
      }

      const current = versions.find((v) => !v.validTo) ?? versions[versions.length - 1]
      const texts = await parallel(SECTIONS.map((part) => async () => ({
        part,
        text: (await cdx('get', `${base}/text?part=${part}`)).slice(0, 4000)
      })))

      const headings = (toc?.items ?? toc ?? [])
      return {
        act: `${ACT.num}/${ACT.year} Sb.`,
        versionId: meta.docId,
        inForceFrom: current?.validFrom ?? null,
        laterBoundaries: versions.filter((v) => current && v.validFrom > current.validFrom)
          .map((v) => v.validFrom),
        provisions: texts.filter(Boolean),
        tocEntries: Array.isArray(headings) ? headings.length : null
      }
    }

The same shape works across acts: put several `base` values in the outer `parallel` and you have the
whole regime - wording, dates and structure - in one call instead of a dozen.

## 5. Commentary sweep

Commentary tied to the provisions an earlier phase actually identified.

    export default async function () {
      const PROVISIONS = [
        { label: "§ 159 odst. 1 o. z.", docId: "CR17044", query: "péče řádného hospodáře" },
        { label: "§ 51 z. o. k.", docId: "CR17045", query: "pravidlo podnikatelského úsudku" }
      ]

      const swept = await parallel(PROVISIONS.map((provision) => async () => {
        const body = await json('search', 'COMMENT', '--query', provision.query,
          '--related-doc', provision.docId, '--limit', '20')
        return {
          provision: provision.label,
          relatedTo: provision.docId,
          totalResults: body.totalResults ?? 0,
          commentary: (body.results ?? []).slice(0, 5).map((r) => r.docId)
        }
      }))

      return { provisions: swept.filter(Boolean) }
    }

Commentary is secondary. Keep it in its own bucket so the opinion never rests a proposition on
it - it points at primary sources, it does not replace them.

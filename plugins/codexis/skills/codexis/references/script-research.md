# Retrieval scripts for CODEXIS

Five programs for the `script` tool. Each one is complete: fan out, dedupe, open only what
survives, and return an evidence package instead of the haystack.

## Ground rules these examples follow

- **Search gives you `docId`. Everything you cite comes from `/meta`.** A search item is
  `{docId, …}` inside `{totalResults, results, availableFilters?}`; `court`, `spZns`,
  `cislaJednaci`, `releaseDate`, `ecli`, `sbirkoveCislo` and `derogated` are guaranteed on
  `cdx://doc/<id>/meta` under the source key (`jd`, `cr`, …). Never invent a citation from a hit.
- **`2>/dev/null` on every command.** The shell tool merges stdout and stderr, so one warning
  line from the CLI breaks `JSON.parse`.
- **Full text goes to a file first.** `cdx://doc/<id>/text` is `text/plain` and a decision easily
  exceeds the shell tool's output cap, which would hand the program a silently truncated string.
  Write it out, read it with `node:fs`.
- **`--limit` is 1..50.** The CLI rejects more. Send more queries instead of a bigger one.
- **Filters differ per source.** JD has `--court`, `--city`, `--type` (all repeatable). ES has
  `--type` and `--issued-from/--issued-to` but **no `--court`**. CR has `--type`, `--author`,
  `--current`, `--valid-at`. `ALL` rejects `--with-facets`.
- **Report the misses.** A candidate whose text carries no matching passage is a fact about the
  query, not something to hide.

A shared preamble, assumed by every example below:

    const sh = async (command) => (await shell({ command, timeoutSeconds: 180 })).trim()
    const json = async (command) => JSON.parse(await sh(`${command} 2>/dev/null`))
    const quote = (value) => `'${String(value).replace(/'/g, `'\\''`)}'`

## 1. Parallel multi-source sweep

The workhorse: many phrasings, merged, ranked by agreement, opened only at the top.

    export default async function () {
      const QUERIES = [
        ["JD", "moderační právo soudu náhrada škody zaměstnance", `--court ${quote("Nejvyšší soud")}`],
        ["JD", "snížení náhrady škody zaměstnancem zákoník práce", ""],
        ["JD", "odpovědnost zaměstnance za schodek na svěřených hodnotách", `--court ${quote("Nejvyšší soud")}`],
        ["JD", "přiměřené snížení náhrady škody pracovněprávní vztah", "--type Rozsudek"],
        ["ES", "odpovědnost zaměstnance za škodu způsobenou zaměstnavateli", ""]
      ]
      const PASSAGE = /moderac|moderač|snížen[íi] náhrady|přiměřen[éeě] snížen/i
      const TOP = 20

      const found = await parallel(QUERIES.map(([source, query, filters]) => async () => {
        const body = await json(
          `cdx-cli search ${source} --query ${quote(query)} ${filters} --limit 50`)
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

      const { readFile } = await import("node:fs/promises")

      const documents = await pipeline(candidates,
        async (candidate) => {
          const meta = await json(`cdx-cli get cdx://doc/${candidate.docId}/meta`)
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
          const file = `.tmp/${doc.docId}.txt`
          await sh(`mkdir -p .tmp && cdx-cli get cdx://doc/${doc.docId}/text > ${file} 2>/dev/null`)
          const text = await readFile(file, "utf8")
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
        const body = await json(
          `cdx-cli get ${quote(`cdx://doc/${docId}/related?type=${RELATION}&limit=50`)}`)
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
        const meta = await json(`cdx-cli get cdx://doc/${candidate.docId}/meta`)
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

## 3. Wording at a date

Which of these statutes actually changed around the day that matters.

    export default async function () {
      const EVENT_DATE = "2018-06-15"
      const ACTS = [["89", "2012"], ["262", "2006"], ["90", "2012"]]

      const checked = await parallel(ACTS.map(([num, year]) => async () => {
        const versions = await json(`cdx-cli get cdx://cz_law/${num}/${year}/versions`)
        const inForce = versions.find((v) =>
          v.validFrom <= EVENT_DATE && (!v.validTo || v.validTo >= EVENT_DATE))
        const later = versions.filter((v) => v.validFrom > EVENT_DATE)
        return {
          act: `${num}/${year} Sb.`,
          versionAtEvent: inForce?.versionId ?? null,
          validFrom: inForce?.validFrom ?? null,
          amendedSince: later.length,
          nextBoundary: later[0]?.validFrom ?? null,
          amendmentDocIds: (later[0]?.amendmentDocIds ?? []).slice(0, 5)
        }
      }))

      const changed = checked.filter(Boolean).filter((act) => act.amendedSince > 0)
      return { eventDate: EVENT_DATE, changed, unchanged: checked.filter(Boolean)
        .filter((act) => act.amendedSince === 0).map((act) => act.act) }
    }

Only the acts in `changed` need the historical wording opened; hand them to
`references/czech-law-change-assessment.md`.

## 4. Contrarian search

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
        const body = await json(
          `cdx-cli search JD --query ${quote(query)} --court ${quote("Nejvyšší soud")} --limit 50`)
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

A decision that answers both query sets is usually the one that draws the line — read it first.
Finding no opposing authority is not proof there is none; say so in the opinion.

## 5. Commentary sweep

Commentary tied to the provisions an earlier phase actually identified.

    export default async function () {
      const PROVISIONS = [
        { label: "§ 159 odst. 1 o. z.", docId: "CR17044", query: "péče řádného hospodáře" },
        { label: "§ 51 z. o. k.", docId: "CR17045", query: "pravidlo podnikatelského úsudku" }
      ]

      const swept = await parallel(PROVISIONS.map((provision) => async () => {
        const body = await json(
          `cdx-cli search COMMENT --query ${quote(provision.query)}`
          + ` --related-doc ${quote(provision.docId)} --limit 20`)
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
it — it points at primary sources, it does not replace them.

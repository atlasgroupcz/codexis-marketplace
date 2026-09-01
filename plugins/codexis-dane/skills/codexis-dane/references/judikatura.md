## Judikatura Workflow

For case-law or judikatura research, prefer a dedicated decision-finding workflow over generic legal search.
Some phrases meaning case law research - `najdi judikaturu`, `najdi rozsudek`, `existuje rozhodnutí k ...`, `jak rozhodl Nejvyšší soud / Ústavní soud / vrchní soud`, atp.

1. Start in `JD`, not `ALL`, when the user is asking for Czech court decisions.
2. Use `cdx-cli search JD --help` if you need to confirm available filters.
3. When doing case law research ALWAYS delegate JD candidate discovery to a subagent if available. Write the brief so the agent never has to come back and ask:
   - the user's question in their own words, and which legal concepts you have already ruled in or out,
   - three to eight rewordings to try - one phrasing finds one line of case law,
   - the source (`JD`) and any filter you already know holds (court, type, date range),
   - what to bring back: `docId`, the citation fields read from `/meta`, one to three verbatim passages per decision, and the `https://` document link (the resolved URL from the source block),
   - what not to do: no attribution calls, no summarising, and never a citation taken from a search hit.
   Tell it to run the rewordings together rather than one after another, dedupe by `docId`, and rank a decision by how many rewordings found it - surfacing under three phrasings makes it the line of case law, a single hit is only a lead.
4. Wait for agents first, then verify. Do not run your own search in parallel with the agent; it will likely be redundant and less effective than the agent's targeted search.
5. After collecting promising candidates use a subagent to verify the actual content of the decision text and the metadata. Hand it the passages discovery returned. Pay attention to:
   - what conduct actually was at issue in the case,
   - whether the cited statement is ratio, context, or only quoted argument,
   - whether the higher court affirmed, reversed, remanded, or decided only a procedural issue.
6. Do not confuse engagement metrics, quoted third-party comments, or procedural background with the punishable conduct or the holding. In particular, separate what the **court held** from what a **party argued** - see "Attribution: what the COURT held vs. what a PARTY argued" below.
7. If `JD` does not surface an exact match, use `ALL`, `COMMENT`, or `LT` only to identify candidate references, then verify back in `JD` before citing.
8. If no exact higher-court decision is verified, say so explicitly and provide the closest verified cases instead of overstating the result.

### Extracting information from specific case law document

To get specific information from a concrete case law use a subagent instead of retrieving the whole text into this chat. Instruct it to read the metadata first, then the decision text, and to return only the passages that answer the question. A decision easily exceeds the shell tool's output cap, so tell it to write the text to a file and search it there - parsing it straight out of a command's output hands it a silently truncated string.

## Running the discovery as a program

Three or more rewordings, or the same fetch repeated over a list of decisions, is one
`script` program - the hits then cost neither your context nor the agent's. Shape rules for
any such program are in `SKILL.md`; what follows is what case law needs.

Values go into `args`, never into a command string, and the two helpers below are assumed:

    const cdx = async (...args) => {
      const out = await shell({ command: 'cdx-cli', args, timeoutSeconds: 180 })
      return out.trim()
    }
    const json = async (...args) => JSON.parse(await cdx(...args))

`JD` has `--court`, `--city` and `--type`; `ES` has `--type` and `--issued-from/--issued-to` but
**no** `--court`. `--limit` tops out at 50 - send more queries rather than a bigger one.

### 1. Parallel multi-source sweep

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

### 2. Citation chasing

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

### 3. Contrarian search

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

## Attribution: what the COURT held vs. what a PARTY argued

**The rule:** treat as the holding only the `výrok` and those parts of the `odůvodnění` where the **deciding court itself adopts** a factual or legal conclusion. Recapitulated party arguments, a lower court's conclusions the deciding court did NOT adopt, and dissenting opinions are always a *foreign voice* - never present them as the court's conclusion. This is the single most common extraction error - guard against it explicitly.

A Czech decision is layered into a binding `výrok` and an `odůvodnění`. The `odůvodnění` typically restates, in reported speech, what the lower courts decided and what each party submitted, and gives the court's **own legal assessment** - but the form varies. It is NOT reliably "recapitulation first, assessment second": brief decisions skip the recapitulation, some interleave restatement with assessment, and some expressly adopt parts of the lower court's reasoning. So attribute by the *voice of each passage* (see signal phrases below), not by its position in the text.

Anatomy of a decision, by attribution layer:

| Layer | What it is | Whose voice |
|-------|-----------|-------------|
| `výrok` ("soud rozhodl takto") | The operative ruling - who won, what was annulled/awarded/remanded | **Court (binding)** |
| Rekapitulace řízení / vyjádření účastníků | Restatement of party submissions and lower-court conclusions | **Parties / lower court (reported, not endorsed)** |
| Vlastní posouzení soudu | The deciding court's legal reasoning → the ratio | **Court (this is the holding)** |
| Lower-court reasoning the higher court **expressly adopts** | Becomes the deciding court's own reasoning *to that extent* | **Court (holding, in that scope)** |
| Lower-court reasoning the higher court **reverses / rejects / overrules** | Superseded - must NOT be shown as the higher court's conclusion | **Lower court (not the holding)** |
| `právní věta` | A short summary of the ratio - useful ratio-decidendi signal, but verify against the `odůvodnění`; in commercial databases it may be the **publisher's editorial headnote**, not the court's text | Ratio indicator (court's at NS Sbírka; possibly editorial elsewhere) |
| Odlišné stanovisko (dissent) | Minority opinion | **Individual judge - NOT the holding** |

Signal phrases - read the verb before attributing any statement:

- **Party submission (do NOT attribute to the court):** "žalobce / žalovaný namítá / tvrdí / uvádí / dovozuje / má za to / navrhuje", "stěžovatel namítá / spatřuje porušení", "dovolatel dovozuje / vytýká", "podle navrhovatele / odpůrce", "státní zástupce dovozoval", "obviněný / obžalovaný uvedl", "podle dovolání".
- **Court's own holding:** "soud dospěl k závěru", "soud má za to", "Nejvyšší soud / Ústavní soud uzavírá", "dovolací soud konstatuje", "podle názoru soudu", "soud proto rozhodl / zrušil / zamítl", "námitku soud neshledal důvodnou".

When the court **quotes an argument only to reject it** ("k námitce, že …, soud uvádí, že je nedůvodná"), the rejected proposition is the OPPOSITE of the holding. Do not lift the quoted argument as the court's position.

In any output - summary, report `summary`/`legal_sentence`, comparison - attribute every legal proposition to its source: say "soud dovodil, že …" vs. "žalobce namítal, že …". If you cannot tell from the text which layer a statement belongs to, label it as uncertain rather than guessing; re-read the surrounding paragraph or fetch more context before asserting it as the holding.

### Worked examples of misattribution (avoid these)

- Decision text: *"Stěžovatel namítal, že výpověď je neplatná pro absenci projednání s odborovou organizací. Ústavní soud však uzavřel, že projednání proběhlo a ústavní stížnost zamítl."*
  - ❌ Wrong: "ÚS dovodil, že výpověď je neplatná pro absenci projednání s odborovou organizací."
  - ✅ Right: "Stěžovatel tvrdil neplatnost výpovědi; ÚS stížnost **zamítl** se závěrem, že projednání s odborovou organizací proběhlo."
- Decision text: *"Krajský soud žalobě vyhověl. Nejvyšší soud rozsudek krajského soudu zrušil."*
  - ❌ Wrong: presenting the krajský soud's pro-žalobce conclusion as the final outcome.
  - ✅ Right: "Krajský soud žalobě vyhověl, **NS jeho rozsudek zrušil** - rozhodující je závěr NS, nikoli překonaný závěr krajského soudu."

## Citation format

Every visible court-decision citation/link MUST be a compact legal citation a lawyer can read at a glance - court abbreviation FIRST. Never use a truncated generic title (e.g. "Usnesení - Usnesení Ústavního soudu ze…") as the primary link text.

```
COURT - ČÍSLO JEDNACÍ / SPISOVÁ ZNAČKA - DD.MM.YYYY
```

Build it from JD metadata (search result or `cdx-cli get cdx://doc/<id>/meta`):

| Part | Field | Rule |
|------|-------|------|
| Court | `court` (full name) → abbreviation | mapping below; unknown court → keep its official short name verbatim |
| Reference | `cislaJednaci[0]`, else `spZns[0]` | číslo jednací preferred over spisová značka; if neither, use `ecli` then `sbirkoveCislo` |
| Date | `releaseDate` → `DD.MM.YYYY` | decision date; omit the ` - DD.MM.YYYY` segment entirely if absent - never invent or zero-pad |

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

For `ES` (EU case law) the same shape applies with EU court abbreviations: the Court of Justice of the EU is `SDEU` (Soudní dvůr EU) and the European Court of Human Rights is `ESLP`. The reference is the case number, falling back to `ecli` then `celex` - e.g. `SDEU - C-123/24 - 04.10.2024`.

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

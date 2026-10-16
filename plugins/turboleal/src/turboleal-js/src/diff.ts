export type DiffItem =
  | { kind: "deleted"; where: string | null; text: string }
  | { kind: "inserted"; where: string | null; text: string }
  | { kind: "changed"; where: string | null; before: string; after: string; referencesOnly: boolean }
  | { kind: "renumbered"; before: string | null; after: string | null; text: string }

export type DiffOverview = { items: DiffItem[]; renumberingOnly: boolean }

export type CompactChange =
  | { kind: "deleted" | "inserted"; where: string | null; text: string }
  | { kind: "changed"; where: string | null; before: string; after: string }

export type CompactOverview = {
  changes: CompactChange[]
  more: number
  renumbered: number
  renumberingOnly: boolean
}

const COMPACT_CHANGES = 6
const COMPACT_CHARS = 200

type Entry = { block: number; text: string; enumerator: string | null; body: string; where: string | null }

const INSERTED = /\{\+\+([\s\S]*?)\+\+\}/g
const DELETED = /\{--([\s\S]*?)--\}/g
const ENUMERATOR = /^(\(\d+[a-z]*\)|[a-z]{1,2}\)|\d+[a-z]*\.)\s*/
const REFERENCE =
  /(§|odst\.|odstavc[eiůích]*|odstavec|písm\.|písmen[eaou]?|bod[uůy]?|čl\.|článk[uůy])\s*(\d+[a-z]*|[a-z]{1,2}\))/g
const SIMILAR = 0.6

/**
 * Reads a CriticMarkup diff paragraph by paragraph: realigns paragraphs that only
 * moved, so a deleted paragraph, an inserted one, a rewritten one and a mere
 * renumbering each come out as their own item.
 */
export function diffOverview(diffMarkdown: string | null | undefined): DiffOverview | null {
  if (!diffMarkdown) return null
  const blocks = diffMarkdown.split(/\n\s*\n/).filter((block) => !block.trimStart().startsWith("#"))
  const marked = blocks.map((block) => block.includes("{--") || block.includes("{++"))
  const before = entries(blocks.map((block) => clean(block.replace(INSERTED, "").replace(DELETED, "$1"))))
  const after = entries(blocks.map((block) => clean(block.replace(DELETED, "").replace(INSERTED, "$1"))))
  const items: DiffItem[] = []
  const usedBefore = new Set<Entry>()
  const usedAfter = new Set<Entry>()
  const pair = (old: Entry, fresh: Entry) => {
    usedBefore.add(old)
    usedAfter.add(fresh)
    if (old.enumerator !== fresh.enumerator && (marked[old.block] || marked[fresh.block])) {
      items.push({ kind: "renumbered", before: old.where, after: fresh.where, text: fresh.text })
    }
  }
  for (const sameBlock of [true, false]) {
    for (const old of before) {
      if (usedBefore.has(old)) continue
      const fresh = after.find(
        (candidate) => !usedAfter.has(candidate) && candidate.body === old.body && (!sameBlock || candidate.block === old.block),
      )
      if (fresh) pair(old, fresh)
    }
  }
  for (const old of before) {
    if (usedBefore.has(old)) continue
    const fresh = after.find((candidate) => !usedAfter.has(candidate) && candidate.block === old.block)
    if (fresh && similarity(old.body, fresh.body) >= SIMILAR) {
      usedBefore.add(old)
      usedAfter.add(fresh)
      items.push({
        kind: "changed",
        where: fresh.where,
        before: old.text,
        after: fresh.text,
        referencesOnly: neutral(old.text) === neutral(fresh.text),
      })
    }
  }
  for (const old of before) if (!usedBefore.has(old)) items.push({ kind: "deleted", where: old.where, text: old.text })
  for (const fresh of after) if (!usedAfter.has(fresh)) items.push({ kind: "inserted", where: fresh.where, text: fresh.text })
  items.sort((left, right) => blockOf(left, before, after) - blockOf(right, before, after))
  const renumberingOnly =
    items.length > 0 &&
    items.every((item) => item.kind === "renumbered" || (item.kind === "changed" && item.referencesOnly))
  return { items, renumberingOnly }
}

function blockOf(item: DiffItem, before: Entry[], after: Entry[]): number {
  const text = item.kind === "changed" ? item.after : item.text
  return (after.find((entry) => entry.text === text) ?? before.find((entry) => entry.text === text))?.block ?? 0
}

/** One entry per non-empty paragraph, labelled with the paragraph, letter and point it sits in. */
function entries(texts: string[]): Entry[] {
  let paragraph: string | null = null
  let letter: string | null = null
  let point: string | null = null
  const result: Entry[] = []
  texts.forEach((text, block) => {
    if (!text) return
    const footnote = /^(\d+)\)\s*/.exec(text)
    if (footnote) {
      result.push({ block, text, enumerator: null, body: text.slice(footnote[0].length), where: `poznámka ${footnote[1]})` })
      return
    }
    const enumerator = ENUMERATOR.exec(text)?.[1] ?? null
    if (enumerator?.startsWith("(")) {
      paragraph = enumerator.slice(1, -1)
      letter = null
      point = null
    } else if (enumerator?.endsWith(")")) {
      letter = enumerator
      point = null
    } else if (enumerator) {
      point = enumerator.slice(0, -1)
    }
    const where = [paragraph && `odst. ${paragraph}`, letter && `písm. ${letter}`, point && `bod ${point}`]
      .filter(Boolean)
      .join(" ")
    result.push({ block, text, enumerator, body: text.replace(ENUMERATOR, ""), where: where || null })
  })
  return result
}

function clean(text: string): string {
  return text.replace(/\\([\\`*_{}[\]()#+\-.!])/g, "$1").replace(/\s+/g, " ").trim()
}

/** The text with paragraph numbers and cross-references blanked out. */
function neutral(text: string): string {
  return text.replace(ENUMERATOR, "#").replace(REFERENCE, "$1 #").replace(/\b[a-z]{1,2}\)/g, "#)")
}

function similarity(left: string, right: string): number {
  const a = new Set(left.toLowerCase().split(/\s+/))
  const b = new Set(right.toLowerCase().split(/\s+/))
  const shared = [...a].filter((word) => b.has(word)).length
  return shared / Math.max(1, Math.min(a.size, b.size))
}

/**
 * The substantive changes only, each passage cut to about 200 characters, at
 * most six of them; renumbering and reference-only edits are just counted.
 */
export function compactOverview(overview: DiffOverview | null): CompactOverview | null {
  if (!overview) return null
  const changes: CompactChange[] = []
  let renumbered = 0
  for (const item of overview.items) {
    if (item.kind === "renumbered" || (item.kind === "changed" && item.referencesOnly)) renumbered++
    else if (item.kind === "changed") {
      changes.push({ kind: "changed", where: item.where, before: clip(item.before), after: clip(item.after) })
    } else changes.push({ kind: item.kind, where: item.where, text: clip(item.text) })
  }
  return {
    changes: changes.slice(0, COMPACT_CHANGES),
    more: Math.max(0, changes.length - COMPACT_CHANGES),
    renumbered,
    renumberingOnly: overview.renumberingOnly,
  }
}

function clip(text: string): string {
  const characters = [...text]
  return characters.length <= COMPACT_CHARS ? text : `${characters.slice(0, COMPACT_CHARS - 1).join("")}…`
}

import { describe, expect, it } from "bun:test"
import { compactOverview, diffOverview } from "./diff"

describe("diffOverview", () => {
  it("reports inserted paragraphs as their own items", () => {
    const overview = diffOverview("### § 38g (CHANGED)\n\n(5) Beze změny.\n\n{++(6) Nově podá přiznání každý.++}\n\n{++(7) A také paušalista.++}")

    expect(overview).toEqual({
      items: [
        { kind: "inserted", where: "odst. 6", text: "(6) Nově podá přiznání každý." },
        { kind: "inserted", where: "odst. 7", text: "(7) A také paušalista." },
      ],
      renumberingOnly: false,
    })
  })

  it("realigns a word-level diff into a deleted paragraph and a renumbered one", () => {
    const overview = diffOverview(
      [
        "(2) Daní na vstupu se rozumí daň.",
        "(3) {--U--}{++Nárok++} {--pořízení automobilu se za daň považuje částka 420 000 Kč.--}{++vzniká okamžikem plnění.++}",
        "{--(4) Nárok vzniká okamžikem plnění.--}",
      ].join("\n\n"),
    )

    expect(overview?.items).toHaveLength(2)
    expect(overview?.items).toContainEqual({
      kind: "deleted",
      where: "odst. 3",
      text: "(3) U pořízení automobilu se za daň považuje částka 420 000 Kč.",
    })
    expect(overview?.items).toContainEqual({
      kind: "renumbered",
      before: "odst. 4",
      after: "odst. 3",
      text: "(3) Nárok vzniká okamžikem plnění.",
    })
    expect(overview?.renumberingOnly).toBe(false)
  })

  it("flags a change of letters and references alone as renumbering", () => {
    const overview = diffOverview("d) plnění v § 54 odst. 1 písm. a) až {--j),--}{++i),++} {--l)--}{++k)++} až {--t)--}{++r)++}, nebo")

    expect(overview?.items).toHaveLength(1)
    expect(overview?.items[0]).toMatchObject({ kind: "changed", where: "písm. d)", referencesOnly: true })
    expect(overview?.renumberingOnly).toBe(true)
  })

  it("keeps a changed amount as a substantive change", () => {
    const overview = diffOverview("(1) Přiznání podá, kdo měl příjmy nad {--50--}{++100++} 000 Kč.")

    expect(overview).toEqual({
      items: [
        {
          kind: "changed",
          where: "odst. 1",
          before: "(1) Přiznání podá, kdo měl příjmy nad 50 000 Kč.",
          after: "(1) Přiznání podá, kdo měl příjmy nad 100 000 Kč.",
          referencesOnly: false,
        },
      ],
      renumberingOnly: false,
    })
  })

  it("has nothing to say without a diff", () => {
    expect(diffOverview(null)).toBeNull()
  })
})

describe("compactOverview", () => {
  it("keeps substantive changes cut short and only counts renumbering", () => {
    const long = "slovo ".repeat(60).trim()
    const overview = diffOverview(
      [
        "(2) Daní na vstupu se rozumí daň.",
        "(3) {--U--}{++Nárok++} {--pořízení automobilu se za daň považuje částka 420 000 Kč.--}{++vzniká okamžikem plnění.++}",
        "{--(4) Nárok vzniká okamžikem plnění.--}",
        `{++(5) ${long}++}`,
      ].join("\n\n"),
    )

    const compact = compactOverview(overview)

    expect(compact?.renumbered).toBe(1)
    expect(compact?.more).toBe(0)
    expect(compact?.changes.map((change) => change.kind)).toEqual(["deleted", "inserted"])
    const inserted = compact?.changes[1]
    expect(inserted?.kind === "inserted" && [...inserted.text].length).toBe(200)
    expect(inserted?.kind === "inserted" && inserted.text.endsWith("…")).toBe(true)
  })

  it("shows at most six changes and says how many more there are", () => {
    const paragraphs = Array.from({ length: 8 }, (_, index) => `{++(${index + 1}) Nový odstavec číslo ${index + 1}.++}`)

    const compact = compactOverview(diffOverview(paragraphs.join("\n\n")))

    expect(compact?.changes).toHaveLength(6)
    expect(compact?.more).toBe(2)
  })

  it("has nothing to say without a diff", () => {
    expect(compactOverview(null)).toBeNull()
  })
})

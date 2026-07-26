// @template master-file
// @use-when Transfer pricing documentation — assembling or reviewing the Master File for a group:
//   work out which chapters the rules require, collect what the group already has, and draft the
//   chapters from those documents rather than from memory.
// @jurisdictions CZ SK EU
// @args group (required) — name of the group the Master File is assembled for
// @args year — the tax period; defaults to the last closed one
//
// The chapter list comes from the rules themselves (OECD Chapter V as implemented locally), not from
// a hardcoded outline, so the template survives a change in the requirements. Missing inputs are
// requested from the user once, and every chapter is drafted only from material handed to it.

export const meta = {
  name: "master-file",
  description: "Master File k převodním cenám: požadavky → podklady → kapitoly → kontrola úplnosti.",
  phases: [
    { title: "Požadavky", detail: "Zjistit z předpisů a pokynů, co má Master File obsahovat" },
    { title: "Podklady", detail: "Inventura dokumentů skupiny a dožádání chybějících" },
    { title: "Kapitoly", detail: "Zpracování jednotlivých kapitol z doložených podkladů" },
    { title: "Kontrola", detail: "Kontrola úplnosti proti požadavkům a soupis mezer" }
  ]
}

const REQUIREMENTS = { type: "object", required: ["chapters"], properties: {
  regime: { type: "string" },
  chapters: { type: "array", items: { type: "object", required: ["chapter", "requires", "source"],
    properties: { chapter: { type: "string" }, requires: { type: "string" }, source: { type: "string" },
      quote: { type: "string" },
      inputs: { type: "array", items: { type: "string" } } } } } } }

const INVENTORY = { type: "object", required: ["present", "missing"], properties: {
  present: { type: "array", items: { type: "object", required: ["input", "path"], properties: {
    input: { type: "string" }, path: { type: "string" }, note: { type: "string" } } } },
  missing: { type: "array", items: { type: "string" } } } }

const CHAPTER = { type: "object", required: ["chapter", "content", "basedOn"], properties: {
  chapter: { type: "string" }, content: { type: "string" },
  basedOn: { type: "array", items: { type: "string" } },
  gaps: { type: "array", items: { type: "string" } } } }

const REVIEW = { type: "object", required: ["summary", "chapters", "gaps"], properties: {
  summary: { type: "string" },
  chapters: { type: "array", items: { type: "object", required: ["chapter", "status"], properties: {
    chapter: { type: "string" }, status: { enum: ["úplná", "částečná", "chybí"] },
    note: { type: "string" } } } },
  gaps: { type: "array", items: { type: "string" } }, caveats: { type: "string" } } }

const SKILLS = ["codexis"]

export default async function (args) {
  const GROUP = (typeof args === "object" && args && typeof args.group === "string" && args.group.trim()) || ""
  if (!GROUP) {
    return { error: "Chybí skupina. Předej ji jako args: { group: '<název skupiny>' }." }
  }
  const YEAR = (args && args.year) || "poslední ukončené zdaňovací období"

  phase("Požadavky")
  const requirements = await agent(
    "Zjisti, co musí obsahovat Master File dokumentace k převodním cenám pro skupinu působící v ČR, " +
    "za období: " + YEAR + ".\n\n" +
    "Vyjdi z rozhodné úpravy a z pokynů daňové správy (zdroje CR, u unijního rámce EU); OECD Kapitolu V " +
    "ber jako výkladový rámec, ne jako náhradu za místní úpravu. Vrať seznam kapitol tak, jak je " +
    "úprava vyžaduje — neopisuj obecnou osnovu zpaměti. U každé kapitoly uveď, co přesně má obsahovat, " +
    "zdroj s doslovnou citací, a jaké konkrétní podklady od skupiny jsou k jejímu zpracování potřeba " +
    "(např. schéma vlastnické struktury, seznam nehmotných aktiv, vnitroskupinové smlouvy, " +
    "konsolidovaná účetní závěrka). Structured output only.",
    { label: "požadavky na Master File", skills: SKILLS, schema: REQUIREMENTS })

  const BRIEF =
    "Master File k převodním cenám" +
    "\nSkupina: " + GROUP +
    "\nObdobí: " + YEAR +
    (requirements.regime ? "\nRozhodná úprava: " + requirements.regime : "") +
    "\n(Piš česky, věcně, ve stylu daňové dokumentace. Vše musí být doloženo podkladem skupiny — " +
    "co doložené není, výslovně označ jako mezeru. Nikdy si údaje nevymýšlej.)"

  log("Kapitoly: " + requirements.chapters.map(c => c.chapter).join(", "))

  phase("Podklady")
  const needed = requirements.chapters
    .flatMap(c => (c.inputs || []).map(input => "- " + input + " (pro kapitolu: " + c.chapter + ")"))
    .join("\n")
  const inventory = await agent(
    BRIEF + "\n\nProjdi pracovní adresář a zjisti, které z níže uvedených podkladů skupiny už tam jsou. " +
    "U každého nalezeného uveď přesnou cestu a jednou větou, co obsahuje. Co nenajdeš, vypiš do " +
    "'missing' přesně tak, jak je to pojmenováno níže.\n\nPotřebné podklady:\n" + needed + "\n\n" +
    "Structured output only.",
    { label: "inventura podkladů", schema: INVENTORY })

  let supplied = inventory.present
  if (inventory.missing.length > 0) {
    const answer = await ask(
      "Pro sestavení Master File potřebuji ještě tyto podklady:\n" +
      inventory.missing.map(m => "• " + m).join("\n") +
      "\n\nNahrajte je prosím. Co nedodáte, zůstane v dokumentaci označené jako mezera.",
      { files: true })
    if (answer.answered && answer.files.length > 0) {
      supplied = supplied.concat(answer.files.map(path => ({ input: "doplněno uživatelem", path })))
      log("Uživatel doplnil " + answer.files.length + " soubor(ů)")
    }
  }

  const documents = supplied.length > 0
    ? supplied.map(d => "- " + d.input + ": " + d.path + (d.note ? " — " + d.note : "")).join("\n")
    : "(žádné podklady skupiny nejsou k dispozici)"

  phase("Kapitoly")
  const chapters = await parallel(requirements.chapters.map(chapter => () =>
    agent(
      BRIEF + "\n\nZpracuj TUTO kapitolu Master File a žádnou jinou.\n\n" +
      "Kapitola: " + chapter.chapter +
      "\nCo má obsahovat: " + chapter.requires +
      "\nZdroj požadavku: " + chapter.source +
      (chapter.quote ? "\nCitace požadavku: \"" + chapter.quote + "\"" : "") + "\n\n" +
      "Dostupné podklady skupiny (čti z nich, nevymýšlej si obsah):\n" + documents + "\n\n" +
      "Napiš text kapitoly tak, jak má stát v dokumentaci. Do 'basedOn' uveď cesty podkladů, ze " +
      "kterých jsi čerpal. Co podklady nepokrývají, nedomýšlej — vypiš to do 'gaps'. " +
      "Structured output only.",
      { label: chapter.chapter, schema: CHAPTER })
      .catch(() => ({ chapter: chapter.chapter, content: "", basedOn: [],
        gaps: ["kapitolu se nepodařilo zpracovat"] }))))

  phase("Kontrola")
  const drafted = chapters.filter(Boolean).map(c =>
    "### " + c.chapter +
    "\n    podklady: " + (c.basedOn.length > 0 ? c.basedOn.join(", ") : "žádné") +
    "\n    mezery: " + (c.gaps && c.gaps.length > 0 ? c.gaps.join("; ") : "žádné") +
    "\n    text: " + c.content).join("\n\n")

  const required = requirements.chapters
    .map(c => "- " + c.chapter + ": " + c.requires + " [" + c.source + "]").join("\n")

  const review = await agent(
    BRIEF + "\n\nZkontroluj úplnost zpracované dokumentace proti požadavkům.\n\n" +
    "Požadavky:\n" + required + "\n\nZpracované kapitoly:\n" + drafted + "\n\n" +
    "Vše, o co se smíš opřít, je zde — nedohledávej nic nového. U každé kapitoly urči, zda je úplná, " +
    "částečná, nebo chybí, a proč. Do 'gaps' shrň, co je nutné od skupiny ještě získat, seřazené " +
    "podle toho, jak moc to dokumentaci ohrožuje. Structured output only.",
    { label: "kontrola úplnosti", agentType: "codexis:synthesizer", schema: REVIEW })

  return { group: GROUP, period: YEAR, ...review,
    draftedChapters: chapters,
    stats: { required: requirements.chapters.length, drafted: chapters.length,
      documents: supplied.length } }
}

// @template legal-research
// @use-when A legal question that has to be answered from the law itself and applied to the client's
//   own documents: find the governing rules, learn what the courts actually decide on, collect the
//   papers that decide it, check nothing is repealed or overruled, then write the opinion.
// @jurisdictions CZ SK EU
//
// Six phases, in the order a lawyer actually works: frame the question, find out from case law and
// statutes WHAT DECIDES IT and which documents that requires, get those documents from the user,
// research each sub-question in source order, check the sources still hold today, and only then write.
// Every agent gets BRIEF — it sees nothing else — and every finding travels on with its citation and
// its verbatim quote. Tune the sub-question count, the verification depth and the schemas to the task.

export const meta = {
  name: "legal-research",
  description: "Právní rešerše: rámec → rekognoskace → podklady → rešerše → ověření → syntéza.",
  phases: [
    { title: "Rámec", detail: "Určit právní obor, jurisdikci, časový řez a sporné otázky" },
    { title: "Rekognoskace", detail: "Zjistit z judikatury a předpisů, podle čeho se rozhoduje a co je třeba doložit" },
    { title: "Podklady", detail: "Inventura dokumentů klienta a dožádání toho, co chybí" },
    { title: "Rešerše", detail: "Předpisy → komentáře → judikatura → literatura, po dílčích otázkách" },
    { title: "Ověření", detail: "Platnost, účinnost a překonanost každého nosného zdroje" },
    { title: "Syntéza", detail: "Aplikace ověřených závěrů na podklady klienta" }
  ]
}

const FRAME = { type: "object", required: ["area", "issues"], properties: {
  area: { type: "string" }, jurisdiction: { type: "string" }, asOf: { type: "string" },
  issues: { type: "array", items: { type: "object", required: ["label", "question"], properties: {
    label: { type: "string" }, question: { type: "string" }, why: { type: "string" } } } } } }

const FRAMEWORK = { type: "object", required: ["criteria", "provisions", "requiredMaterials"], properties: {
  criteria: { type: "array", items: { type: "object", required: ["criterion", "source"], properties: {
    criterion: { type: "string" }, source: { type: "string" }, quote: { type: "string" } } } },
  provisions: { type: "array", items: { type: "object", required: ["provision", "source"], properties: {
    provision: { type: "string" }, source: { type: "string" }, relevance: { type: "string" } } } },
  requiredMaterials: { type: "array", items: { type: "object", required: ["material", "why"], properties: {
    material: { type: "string" }, why: { type: "string" },
    importance: { enum: ["rozhodující", "podpůrný"] } } } } } }

const INVENTORY = { type: "object", required: ["present", "missing"], properties: {
  present: { type: "array", items: { type: "object", required: ["material", "path"], properties: {
    material: { type: "string" }, path: { type: "string" }, note: { type: "string" } } } },
  missing: { type: "array", items: { type: "string" } } } }

const FINDINGS = { type: "object", required: ["findings"], properties: { findings: { type: "array", items: {
  type: "object", required: ["proposition", "source", "quote", "sourceKind"], properties: {
    proposition: { type: "string" }, source: { type: "string" }, quote: { type: "string" },
    sourceKind: { enum: ["předpis", "komentář", "judikatura", "literatura"] },
    weight: { enum: ["nosný", "podpůrný"] } } } } } }

const CURRENCY = { type: "object", required: ["stillGood", "evidence"], properties: {
  stillGood: { type: "boolean" }, evidence: { type: "string" }, supersededBy: { type: "string" },
  effectiveAt: { type: "string" } } }

const OPINION = { type: "object", required: ["answer", "reasoning", "caveats"], properties: {
  answer: { type: "string" },
  reasoning: { type: "array", items: { type: "object", required: ["point", "sources"], properties: {
    point: { type: "string" }, sources: { type: "array", items: { type: "string" } },
    appliesTo: { type: "string" }, confidence: { enum: ["vysoká", "střední", "nízká"] } } } },
  caveats: { type: "string" }, openQuestions: { type: "array", items: { type: "string" } } } }

const SKILLS = ["codexis"]

export default async function (args) {
  const QUESTION = (typeof args === "object" && args && typeof args.question === "string" && args.question.trim()) || ""
  if (!QUESTION) {
    return { error: "Chybí právní otázka. Předej ji jako args: { question: '<otázka>' }." }
  }
  const JURISDICTION = (args && args.jurisdiction) || "CZ"

  phase("Rámec")
  // The framing agent is the one step that may see the conversation: what the user actually wants is
  // in the chat, not in `args`. Everything downstream reads BRIEF instead.
  const frame = await agent(
    "Jsi na začátku právní rešerše. Z otázky, z konverzace a ze souborů v pracovním adresáři urči " +
    "rámec: právní obor, rozhodnou jurisdikci, časový řez (ke kterému dni se posuzuje) a rozpad na " +
    "3-8 dílčích právních otázek, které je nutné zodpovědět, aby šlo odpovědět na tu hlavní.\n\n" +
    "Otázka: " + QUESTION + "\nJurisdikce: " + JURISDICTION + "\n\n" +
    "Každá dílčí otázka musí být samostatně zodpověditelná a nesmí se s ostatními překrývat — každá " +
    "z nich dostane vlastního rešeršistu, který neuvidí nic než její zadání. Structured output only.",
    { label: "rámec", inheritChat: true, schema: FRAME })

  const BRIEF =
    "Právní otázka: " + QUESTION +
    "\nObor: " + frame.area +
    "\nJurisdikce: " + (frame.jurisdiction || JURISDICTION) +
    (frame.asOf ? "\nPosuzuje se ke dni: " + frame.asOf : "") +
    "\n(Odpovídej česky. Každé tvrzení musí nést přesnou citaci zdroje a doslovný výňatek z něj. " +
    "Nikdy si zdroj nevymýšlej — když ho nemáš, řekni to.)"

  log("Dílčí otázky: " + frame.issues.map(i => i.label).join(", "))

  phase("Rekognoskace")
  // Before asking the user for anything, find out what actually decides cases like this — that is what
  // tells us which documents are worth asking for.
  const framework = await agent(
    BRIEF + "\n\nRychlá rekognoskace, ne plná rešerše. Zjisti, PODLE ČEHO se tato věc rozhoduje:\n" +
    "1. Která ustanovení se na ni aplikují (zdroje CR, u unijního přesahu EU).\n" +
    "2. Podle jakých kritérií soudy takové věci posuzují — projdi judikaturu (JD, u unijního práva ES) " +
    "a vytáhni test nebo znaky, které soudy fakticky používají.\n" +
    "3. Z toho odvoď, JAKÉ PODKLADY je nutné mít, aby šlo věc posoudit — konkrétní dokumenty " +
    "(smlouva, obchodní podmínky, korespondence, faktury, protokoly), ne obecné kategorie. " +
    "U každého napiš, co se z něj prokazuje, a označ, jestli je rozhodující, nebo jen podpůrný.\n\n" +
    "Tvůj výstup řídí zbytek rešerše, takže u každého kritéria uveď zdroj a doslovnou citaci. " +
    "Structured output only.",
    { label: "rozhodovací rámec", skills: SKILLS, schema: FRAMEWORK })

  log("Rozhodovací kritéria: " + framework.criteria.map(c => c.criterion).join(" | "))

  phase("Podklady")
  const materials = framework.requiredMaterials
    .map(m => "- " + m.material + " (" + (m.importance || "podpůrný") + "): " + m.why)
    .join("\n")
  const inventory = await agent(
    BRIEF + "\n\nProjdi pracovní adresář uživatele a zjisti, které z níže uvedených podkladů už tam " +
    "jsou. Pro každý nalezený uveď přesnou cestu k souboru a jednou větou, co v něm je; nečti je celé, " +
    "stačí ověřit, že jde opravdu o ten dokument. Co nenajdeš, vypiš do 'missing' přesně tak, jak je " +
    "to pojmenované níže.\n\nPotřebné podklady:\n" + materials + "\n\nStructured output only.",
    { label: "inventura podkladů", schema: INVENTORY })

  let supplied = inventory.present
  if (inventory.missing.length > 0) {
    const answer = await ask(
      "Pro posouzení potřebuji ještě tyto podklady:\n" +
      inventory.missing.map(m => "• " + m).join("\n") +
      "\n\nNahrajte je prosím, nebo napište, že je nemáte — budu pokračovat bez nich a v závěru to uvedu.",
      { files: true })
    if (answer.answered && answer.files.length > 0) {
      supplied = supplied.concat(answer.files.map(path => ({ material: "doplněno uživatelem", path })))
      log("Uživatel doplnil " + answer.files.length + " soubor(ů)")
    } else {
      log("Podklady nedoplněny — pokračuji s tím, co je k dispozici")
    }
  }

  const missingAfterAsk = inventory.missing.length > 0 && supplied.length === inventory.present.length
    ? inventory.missing
    : []

  phase("Rešerše")
  // One researcher per sub-question, each working the sources in the order that matters: the rule
  // first, then how it is read, then how it is decided, then what the literature says.
  const perIssue = await parallel(frame.issues.map(issue => () =>
    agent(
      BRIEF + "\n\nZodpověz TUTO dílčí otázku a nic jiného.\n\n" +
      "Dílčí otázka: " + issue.label + " — " + issue.question +
      (issue.why ? "\nProč je podstatná: " + issue.why : "") + "\n\n" +
      "Rozhodovací rámec zjištěný v rekognoskaci (drž se ho, ale ověř si ho vlastní rešerší):\n" +
      framework.criteria.map(c => "- " + c.criterion + " [" + c.source + "]").join("\n") + "\n\n" +
      "Postupuj v tomto pořadí zdrojů a v každém kroku vytěž, co k otázce skutečně patří:\n" +
      "1. PŘEDPISY (CR, u unijního přesahu EU) — znění rozhodných ustanovení.\n" +
      "2. KOMENTÁŘE (COMMENT) — jak se ta ustanovení vykládají.\n" +
      "3. JUDIKATURA (JD, ES) — jak se podle nich fakticky rozhoduje; přednostně vyšší soudy.\n" +
      "4. LITERATURA (LT) — jen pokud po předchozích krocích zbývá sporná otázka.\n\n" +
      "Tvá zjištění půjdou na ověření a pak do syntézy, takže u každého uveď přesnou citaci " +
      "(číslo předpisu a ustanovení, spisovou značku) a doslovný výňatek. Označ, které je nosné a " +
      "které jen podpůrné. Structured output only.",
      { label: issue.label, skills: SKILLS, schema: FINDINGS })
      .catch(() => ({ findings: [] }))))

  const allFindings = perIssue.flatMap(r => r.findings)
  const load = { nosný: 0, podpůrný: 1 }
  const loadBearing = [...allFindings]
    .sort((a, b) => (load[a.weight] ?? 1) - (load[b.weight] ?? 1))
    .slice(0, 14)
  log("Zjištění: " + allFindings.length + ", ověřuji " + loadBearing.length + " nosných")
  if (loadBearing.length === 0) {
    return { question: QUESTION, answer: "Rešerše nepřinesla použitelná zjištění.",
      caveats: "Zdroje se nepodařilo dohledat.", missingMaterials: missingAfterAsk }
  }

  phase("Ověření")
  const verified = await parallel(loadBearing.map(finding => () =>
    agent(
      BRIEF + "\n\nOvěř, jestli tohle zjištění platí i DNES. Jsi skeptik: hledej důvod, proč neplatí.\n\n" +
      "Tvrzení: " + finding.proposition +
      "\nZdroj: " + finding.source +
      "\nDruh zdroje: " + finding.sourceKind +
      "\nDoslovná citace: " + finding.quote + "\n\n" +
      "U předpisu ověř účinné znění ke dni posouzení: nebyl zrušen, novelizován ani nahrazen? " +
      "U judikatury ověř, že rozhodnutí je stále dobrou autoritou: nebylo překonáno velkým senátem, " +
      "zrušeno Ústavním soudem ani opuštěno pozdější praxí? U komentáře a literatury ověř, že vychází " +
      "z účinného znění. Otevři zdroj celý, nesuď podle úryvku. Do 'evidence' napiš rozhodující " +
      "formulaci — cituje se do závěrečného stanoviska. Structured output only.",
      { label: "ověření: " + finding.proposition.slice(0, 40), agent: "codexis:verifier", skills: SKILLS,
        schema: CURRENCY })
      .then(verdict => ({ finding, verdict }))
      .catch(() => ({ finding, verdict: { stillGood: true, evidence: "ověření se nepodařilo provést" } }))))

  const holding = verified.filter(v => v.verdict.stillGood)
  const superseded = verified.filter(v => !v.verdict.stillGood)
  log("Obstálo " + holding.length + " z " + verified.length + " zjištění")

  phase("Syntéza")
  // The synthesizer has no research tools at all, so everything it may rely on has to be here:
  // the proposition, its citation, the quote, and what the verifier found.
  const block = holding.map((v, i) =>
    "[" + i + "] " + v.finding.proposition +
    "\n    zdroj (" + v.finding.sourceKind + "): " + v.finding.source +
    "\n    citace: \"" + v.finding.quote + "\"" +
    (v.verdict.effectiveAt ? "\n    účinné znění k: " + v.verdict.effectiveAt : "") +
    (v.verdict.evidence ? "\n    ověření: " + v.verdict.evidence : "")).join("\n\n")

  const clientDocuments = supplied.length > 0
    ? supplied.map(d => "- " + d.material + ": " + d.path + (d.note ? " — " + d.note : "")).join("\n")
    : "(žádné podklady klienta nejsou k dispozici)"

  const opinion = await agent(
    BRIEF + "\n\nNapiš právní stanovisko k otázce výše.\n\n" +
    "Ověřená zjištění (každé se zdrojem, doslovnou citací a poznámkou z ověření):\n" + block + "\n\n" +
    "Podklady klienta, na které se závěry aplikují:\n" + clientDocuments + "\n\n" +
    (superseded.length > 0
      ? "Tato zjištění NEOBSTÁLA a nesmíš o ně opřít závěr: " +
        superseded.map(v => v.finding.source + " (" + v.verdict.evidence + ")").join("; ") + "\n\n"
      : "") +
    (missingAfterAsk.length > 0
      ? "Tyto podklady chybí a závěr je v jejich rozsahu předběžný: " + missingAfterAsk.join(", ") +
        " — uveď to v caveats.\n\n"
      : "") +
    "Přečti celý materiál výše, než začneš psát. Vše, o co se smíš opřít, je v něm — nedoplňuj nic " +
    "z vlastní paměti a nedohledávej nové zdroje. Nejdřív odpověz na otázku přímo, pak rozveď " +
    "argumentaci po bodech; u každého bodu uveď přesné zdroje z materiálu a napiš, na který konkrétní " +
    "podklad klienta se aplikuje. Přiřaď míru jistoty. Kde materiál otázku neuzavírá, řekni to " +
    "a dej to do caveats nebo otevřených otázek. Structured output only.",
    { label: "stanovisko", agent: "codexis:synthesizer", schema: OPINION })

  return {
    question: QUESTION,
    ...opinion,
    framework: framework.criteria,
    supersededSources: superseded.map(v => ({ source: v.finding.source, why: v.verdict.evidence })),
    missingMaterials: missingAfterAsk,
    stats: { issues: frame.issues.length, findings: allFindings.length, verified: verified.length,
      holding: holding.length }
  }
}

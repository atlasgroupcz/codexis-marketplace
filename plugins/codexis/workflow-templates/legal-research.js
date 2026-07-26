// @template legal-research
// @use-when A legal question that has to be answered from the law itself and applied to the client's
//   own documents: find the governing rules, learn what the courts actually decide on, collect the
//   papers that decide it, check nothing is repealed or overruled, then write the opinion.
// @jurisdictions CZ SK EU
// @args question (required) — the legal question, in the user's own words
// @args jurisdiction — CZ, SK or EU; defaults to CZ
//
// Seven phases, in the order a lawyer actually works: frame the question, find out from case law and
// statutes WHAT DECIDES IT and which documents that requires, get those documents from the user, pull
// the decisive passages out of them, research each sub-question in source order, check the sources
// still hold today, and only then write.
// Every agent gets BRIEF — it sees nothing else — and every finding travels on with its citation and
// its verbatim quote. Before the write-up, cdx-citation-check re-opens each cited document and drops
// any finding whose quote is not literally in it; that is the one check no model performs.
// Tune the sub-question count, the verification depth and the schemas to the task.

import { execFileSync } from "node:child_process"

export const meta = {
  name: "legal-research",
  description: "Právní rešerše: rámec → rekognoskace → podklady → extrakce → rešerše → ověření → syntéza.",
  phases: [
    { title: "Rámec", detail: "Určit právní obor, jurisdikci, časový řez a sporné otázky" },
    { title: "Rekognoskace", detail: "Zjistit z judikatury a předpisů, podle čeho se rozhoduje a co je třeba doložit" },
    { title: "Podklady", detail: "Inventura dokumentů klienta a dožádání toho, co chybí" },
    { title: "Extrakce", detail: "Vytěžení rozhodných skutečností z dokumentů klienta" },
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
  present: { type: "array", items: { type: "object", required: ["materialId", "path"], properties: {
    materialId: { type: "string" }, material: { type: "string" }, path: { type: "string" },
    note: { type: "string" } } } },
  missing: { type: "array", items: { type: "object", required: ["materialId", "material"], properties: {
    materialId: { type: "string" }, material: { type: "string" } } } } } }

const FINDINGS = { type: "object", required: ["findings"], properties: { findings: { type: "array", items: {
  type: "object", required: ["proposition", "source", "docId", "quote", "sourceKind"], properties: {
    proposition: { type: "string" }, source: { type: "string" },
    // The id the agent actually fetched, so the quote can be checked against the document itself
    // instead of against another model's recollection of it.
    docId: { type: "string" }, part: { type: "string" },
    quote: { type: "string" },
    sourceKind: { enum: ["předpis", "komentář", "judikatura", "literatura"] },
    weight: { enum: ["nosný", "podpůrný"] } } } } } }

// What the client's own documents actually say, pulled out before anyone reasons about them.
const FACTS = { type: "object", required: ["facts"], properties: {
  facts: { type: "array", items: { type: "object", required: ["fact", "path", "quote"], properties: {
    fact: { type: "string" }, path: { type: "string" }, locator: { type: "string" },
    quote: { type: "string" } } } },
  notCovered: { type: "array", items: { type: "string" } } } }

// Four states, never a boolean: "could not check" must never be able to masquerade as "checked".
const CURRENCY = { type: "object", required: ["status", "evidence"], properties: {
  status: { enum: ["supported", "refuted", "unverified", "source_unreachable"] },
  evidence: { type: "string" }, supersededBy: { type: "string" },
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
  const frame = await agent(
    "Jsi na začátku právní rešerše. Z otázky a ze souborů v pracovním adresáři urči " +
    "rámec: právní obor, rozhodnou jurisdikci, časový řez (ke kterému dni se posuzuje) a rozpad na " +
    "3-8 dílčích právních otázek, které je nutné zodpovědět, aby šlo odpovědět na tu hlavní.\n\n" +
    "Otázka: " + QUESTION + "\nJurisdikce: " + JURISDICTION + "\n\n" +
    "Každá dílčí otázka musí být samostatně zodpověditelná a nesmí se s ostatními překrývat — každá " +
    "z nich dostane vlastního rešeršistu, který neuvidí nic než její zadání. Structured output only.",
    { label: "rámec", schema: FRAME })

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
    .map((m, i) => "- [M" + i + "] " + m.material + " (" + (m.importance || "podpůrný") + "): " + m.why)
    .join("\n")
  const inventory = await agent(
    BRIEF + "\n\nProjdi pracovní adresář uživatele a zjisti, které z níže uvedených podkladů už tam " +
    "jsou. Pro každý nalezený uveď přesnou cestu k souboru a jednou větou, co v něm je; nečti je celé, " +
    "stačí ověřit, že jde opravdu o ten dokument. Co nenajdeš, vypiš do 'missing' přesně tak, jak je " +
    "to pojmenované níže; v 'materialId' vrať jeho značku [M0], [M1], … beze změny.\n\n" +
    "Potřebné podklady:\n" + materials + "\n\nStructured output only.",
    { label: "inventura podkladů", schema: INVENTORY })

  let supplied = inventory.present
  let stillMissing = inventory.missing
  if (stillMissing.length > 0) {
    const answer = await ask(
      "Pro posouzení potřebuji ještě tyto podklady:\n" +
      stillMissing.map(m => "• " + m.material).join("\n") +
      "\n\nNahrajte je prosím, nebo napište, že je nemáte — budu pokračovat bez nich a v závěru to uvedu.",
      { files: true })
    if (answer.answered && answer.files.length > 0) {
      log("Uživatel doplnil " + answer.files.length + " soubor(ů) — přiřazuji je k požadovaným podkladům")
      // Which uploads cover which requested material is a judgement call, so ask an agent — counting
      // files would silently declare everything supplied after a single upload.
      const matched = await agent(
        BRIEF + "\n\nUživatel právě nahrál tyto soubory:\n" +
        answer.files.map(p => "- " + p).join("\n") + "\n\n" +
        "Otevři každý natolik, abys poznal, co to je, a přiřaď ho k požadovanému podkladu níže. " +
        "Do 'present' dej jen ty, které jsi opravdu pokryl, s jejich materialId a cestou. " +
        "Do 'missing' vrať ty, které ani po nahrání nemáš.\n\n" +
        "Požadované podklady:\n" +
        stillMissing.map(m => "- [" + m.materialId + "] " + m.material).join("\n") + "\n\n" +
        "Structured output only.",
        { label: "přiřazení nahraných podkladů", schema: INVENTORY })
        .catch(() => null)
      if (matched) {
        supplied = supplied.concat(matched.present)
        stillMissing = matched.missing
      }
    } else {
      log("Podklady nedoplněny — pokračuji s tím, co je k dispozici")
    }
  }

  const missingAfterAsk = stillMissing.map(m => m.material)

  phase("Extrakce")
  // The synthesizer has no research tools, so a file path in its prompt is a fact it can never read.
  // Pull the decisive passages out of the client's documents here, while an agent that CAN open them
  // is still running, and carry the quotes forward instead of the paths.
  const criteriaList = framework.criteria.map(c => "- " + c.criterion).join("\n")
  const extracted = await parallel(supplied.map(doc => () =>
    agent(
      BRIEF + "\n\nOtevři dokument klienta a vytěž z něj skutečnosti rozhodné pro posouzení.\n\n" +
      "Soubor: " + doc.path + (doc.note ? "\nPoznámka z inventury: " + doc.note : "") + "\n\n" +
      "Posuzuje se podle těchto kritérií:\n" + criteriaList + "\n\n" +
      "Ke každému kritériu, které dokument zmiňuje, vrať jednu skutečnost: co z dokumentu plyne, " +
      "kde přesně to je (článek, odstavec, strana) a DOSLOVNÝ výňatek. Nevykládej právo a nedomýšlej " +
      "— jen popiš, co v dokumentu stojí. Kritéria, ke kterým dokument nic neříká, vypiš do " +
      "'notCovered'. Structured output only.",
      { label: "vytěžení: " + doc.path.split("/").pop(), schema: FACTS })
      .catch(() => null)))

  const clientFacts = extracted.filter(Boolean).flatMap(r => r.facts)
  log("Skutečnosti z podkladů klienta: " + clientFacts.length)

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
      "(číslo předpisu a ustanovení, spisovou značku) a doslovný výňatek. Do 'docId' vrať " +
      "identifikátor dokumentu, který jsi SKUTEČNĚ otevřel (to, co jsi předal do 'cdx-cli get " +
      "cdx://doc/<docId>/text'), a pokud jsi četl konkrétní část, i 'part'. Citace se strojově " +
      "porovná s textem toho dokumentu, takže musí být doslovná — přepsaná nebo zpaměti " +
      "rekonstruovaná citace zjištění vyřadí. Označ, které je nosné a které jen podpůrné. " +
      "Structured output only.",
      { label: issue.label, skills: SKILLS, schema: FINDINGS })
      .catch(() => ({ findings: [] }))))

  const allFindings = perIssue.filter(Boolean).flatMap(r => r.findings)
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
      "formulaci — cituje se do závěrečného stanoviska.\n\n" +
      "Status voli takto: 'supported' jen když jsi zdroj otevřel a tvrzení obstálo; 'refuted' když " +
      "jsi našel konkrétní důvod, proč neplatí; 'source_unreachable' když se ti zdroj nepodařilo " +
      "otevřít; 'unverified' když jsi ho otevřel, ale nedokázal jsi tvrzení potvrdit ani vyvrátit. " +
      "Nedostatek důkazu NENÍ vyvrácení a chybějící přístup NENÍ potvrzení. Structured output only.",
      { label: "ověření: " + finding.proposition.slice(0, 40), agentType: "codexis:verifier", skills: SKILLS,
        schema: CURRENCY })
      .then(verdict => ({ finding, verdict }))
      // A verifier that crashed has verified nothing. Failing open here would let a rate limit
      // promote an unchecked claim into the opinion.
      .catch(() => ({ finding, verdict: { status: "unverified",
        evidence: "ověření se nepodařilo provést (chyba běhu)" } }))))

  // parallel() keeps failed positions as null, so drop those before reading a verdict off them.
  const checked = verified.filter(Boolean)
  const holding = checked.filter(v => v.verdict.status === "supported")
  const refuted = checked.filter(v => v.verdict.status === "refuted")
  const unverified = checked.filter(v => v.verdict.status !== "supported" && v.verdict.status !== "refuted")
  log("Obstálo " + holding.length + " z " + checked.length + " zjištění, "
    + refuted.length + " vyvráceno, " + unverified.length + " neověřeno")

  // Everything above is one model checking another. This is the one step that is not: cdx-citation-check
  // re-fetches each cited document and looks for the quote in it. A quote that is not there did not
  // come from that source, whatever the verifier concluded.
  const citationVerdicts = checkCitations(holding.map((v, i) => ({
    id: String(i), docId: v.finding.docId, part: v.finding.part, quote: v.finding.quote })))
  const grounded = citationVerdicts === null
    ? holding
    : holding.filter((v, i) => citationVerdicts[i] && citationVerdicts[i].status === "verbatim")
  const fabricated = citationVerdicts === null
    ? []
    : holding.filter((v, i) => citationVerdicts[i] && citationVerdicts[i].status !== "verbatim")
  if (citationVerdicts === null) {
    log("Kontrolu citací se nepodařilo spustit — citace zůstávají neověřené strojem")
  } else {
    log("Citace ověřené v dokumentu: " + grounded.length + " z " + holding.length)
  }

  phase("Syntéza")
  // The synthesizer has no research tools at all, so everything it may rely on has to be here:
  // the proposition, its citation, the quote, and what the verifier found.
  const block = grounded.map((v, i) =>
    "[" + i + "] " + v.finding.proposition +
    "\n    zdroj (" + v.finding.sourceKind + "): " + v.finding.source +
    "\n    citace: \"" + v.finding.quote + "\"" +
    (v.verdict.effectiveAt ? "\n    účinné znění k: " + v.verdict.effectiveAt : "") +
    (v.verdict.evidence ? "\n    ověření: " + v.verdict.evidence : "")).join("\n\n")

  const clientDocuments = clientFacts.length > 0
    ? clientFacts.map(f => "- " + f.fact +
        "\n    dokument: " + f.path + (f.locator ? ", " + f.locator : "") +
        "\n    znění: \"" + f.quote + "\"").join("\n")
    : "(z podkladů klienta se nepodařilo vytěžit žádné skutečnosti)"

  const opinion = await agent(
    BRIEF + "\n\nNapiš právní stanovisko k otázce výše.\n\n" +
    "Ověřená zjištění (každé se zdrojem, doslovnou citací a poznámkou z ověření):\n" + block + "\n\n" +
    "Podklady klienta, na které se závěry aplikují:\n" + clientDocuments + "\n\n" +
    (refuted.length > 0
      ? "Tato zjištění byla VYVRÁCENA a nesmíš o ně opřít závěr: " +
        refuted.map(v => v.finding.source + " (" + v.verdict.evidence + ")").join("; ") + "\n\n"
      : "") +
    (unverified.length > 0
      ? "Tato zjištění se NEPODAŘILO OVĚŘIT. Nesmíš o ně opřít závěr ani je citovat jako platná; " +
        "pokud jsou pro otázku podstatná, uveď v caveats, že v tomto rozsahu je stanovisko " +
        "předběžné: " +
        unverified.map(v => v.finding.source + " (" + v.verdict.evidence + ")").join("; ") + "\n\n"
      : "") +
    (missingAfterAsk.length > 0
      ? "Tyto podklady chybí a závěr je v jejich rozsahu předběžný: " + missingAfterAsk.join(", ") +
        " — uveď to v caveats.\n\n"
      : "") +
    (fabricated.length > 0
      ? "Tyto citace se v uvedeném dokumentu NENAŠLY a byly proto vyřazeny; necituj je ani je " +
        "nezmiňuj jako oporu: " +
        fabricated.map(v => v.finding.source).join("; ") + "\n\n"
      : "") +
    (citationVerdicts === null
      ? "POZOR: strojovou kontrolu citací se nepodařilo provést, takže doslovnost výňatků nikdo " +
        "neověřil. Uveď to v caveats.\n\n"
      : "") +
    "Přečti celý materiál výše, než začneš psát. Vše, o co se smíš opřít, je v něm — nedoplňuj nic " +
    "z vlastní paměti a nedohledávej nové zdroje. Nejdřív odpověz na otázku přímo, pak rozveď " +
    "argumentaci po bodech; u každého bodu uveď přesné zdroje z materiálu a napiš, na který konkrétní " +
    "podklad klienta se aplikuje. Přiřaď míru jistoty. Kde materiál otázku neuzavírá, řekni to " +
    "a dej to do caveats nebo otevřených otázek. Structured output only.",
    { label: "stanovisko", agentType: "codexis:synthesizer", schema: OPINION })

  return {
    question: QUESTION,
    ...opinion,
    framework: framework.criteria,
    refutedSources: refuted.map(v => ({ source: v.finding.source, why: v.verdict.evidence })),
    fabricatedCitations: fabricated.map(v => ({ source: v.finding.source, docId: v.finding.docId })),
    citationCheck: citationVerdicts === null ? "unavailable" : "ran",
    unverifiedSources: unverified.map(v => ({ source: v.finding.source, why: v.verdict.evidence })),
    missingMaterials: missingAfterAsk,
    stats: { issues: frame.issues.length, findings: allFindings.length, checked: checked.length,
      holding: holding.length, grounded: grounded.length, refuted: refuted.length,
      unverified: unverified.length }
  }
}

// Runs the checker that ships with the codexis skill. It is installed on PATH as cdx-citation-check;
// when it is not there we return null so the caller can say the citations went unchecked, rather
// than quietly treating them as sound.
function checkCitations(citations) {
  if (citations.length === 0) {
    return []
  }
  try {
    const out = execFileSync("cdx-citation-check", [], {
      input: JSON.stringify(citations), encoding: "utf8", timeout: 300000
    })
    return JSON.parse(out)
  } catch (failure) {
    // A non-zero exit just means some citation failed — the report is still on stdout.
    if (failure && failure.stdout) {
      try {
        return JSON.parse(failure.stdout)
      } catch (unparseable) {
        return null
      }
    }
    return null
  }
}

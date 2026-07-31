// @template legal-research
// @use-when A legal question that has to be answered from the law itself and applied to the client's
//   own documents: find the governing rules, learn what the courts actually decide on, collect the
//   papers that decide it, check nothing is repealed or overruled, then write the opinion.
// @jurisdictions CZ SK EU
// @args question (required) - the legal question, in the user's own words
// @args jurisdiction - CZ, SK or EU; defaults to CZ

const SEARCH_MODEL = "GPT_5_6_LUNA"

export const meta = {
  name: "legal-research",
  description: "Právní rešerše: rámec → rekognoskace → podklady → rešerše → ověření → syntéza.",
  phases: [
    { title: "Rámec", detail: "Určit právní obor, jurisdikci, časový řez a sporné otázky" },
    { title: "Rekognoskace", detail: "Zjistit z judikatury a předpisů, podle čeho se rozhoduje a co je třeba doložit" },
    { title: "Podklady", detail: "Inventura dokumentů klienta, dožádání chybějících podkladů a vytěžení rozhodných skutečností" },
    { title: "Rešerše", detail: "Vyhledání předpisů, komentářů, judikatury a literatury po dílčích otázkách" },
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
    docId: { type: "string" }, part: { type: "string" },
    quote: { type: "string" },
    sourceKind: { enum: ["předpis", "komentář", "judikatura", "literatura"] },
    weight: { enum: ["nosný", "podpůrný"] } } } } } }

const FACTS = { type: "object", required: ["facts"], properties: {
  facts: { type: "array", items: { type: "object", required: ["fact", "path", "quote"], properties: {
    fact: { type: "string" }, path: { type: "string" }, locator: { type: "string" },
    quote: { type: "string" } } } },
  notCovered: { type: "array", items: { type: "string" } } } }

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

export default async function (args) {
  const QUESTION = (typeof args === "object" && args && typeof args.question === "string" && args.question.trim()) || ""
  if (!QUESTION) {
    return { error: "Chybí právní otázka. Předej ji jako args: { question: '<otázka>' }." }
  }
  const JURISDICTION = (args && args.jurisdiction) || "CZ"

  phase("Rámec")
  const frame = await agent(
    `Jsi na začátku právní rešerše. Z otázky a ze souborů v pracovním adresáři urči rámec: právní obor, rozhodnou jurisdikci, časový řez (ke kterému dni se posuzuje) a rozpad na 3-8 dílčích právních otázek, které je nutné zodpovědět, aby šlo odpovědět na tu hlavní.

Otázka: ${QUESTION}
Jurisdikce: ${JURISDICTION}

Každá dílčí otázka musí být samostatně zodpověditelná a nesmí se s ostatními překrývat - každá z nich dostane vlastního rešeršistu, který neuvidí nic než její zadání.`,
    { label: "rámec", schema: FRAME })

  const BRIEF = `Právní otázka: ${QUESTION}
Obor: ${frame.area}
Jurisdikce: ${frame.jurisdiction || JURISDICTION}${frame.asOf ? `\nPosuzuje se ke dni: ${frame.asOf}` : ""}
(Odpovídej česky. Každé tvrzení musí nést přesnou citaci zdroje a doslovný výňatek z něj. Nikdy si zdroj nevymýšlej - když ho nemáš, řekni to.)`

  log(`Dílčí otázky: ${frame.issues.map(i => i.label).join(", ")}`)

  phase("Rekognoskace")
  const framework = await agent(
    `${BRIEF}

Rychlá rekognoskace, ne plná rešerše. Zjisti, PODLE ČEHO se tato věc rozhoduje:
1. Která ustanovení se na ni aplikují (zdroje CR, u unijního přesahu EU).
2. Podle jakých kritérií soudy takové věci posuzují - projdi judikaturu (JD, u unijního práva ES) a vytáhni test nebo znaky, které soudy fakticky používají.
3. Z toho odvoď, JAKÉ PODKLADY je nutné mít, aby šlo věc posoudit - konkrétní dokumenty (smlouva, obchodní podmínky, korespondence, faktury, protokoly), ne obecné kategorie. U každého napiš, co se z něj prokazuje, a označ, jestli je rozhodující, nebo jen podpůrný.

Tvůj výstup řídí zbytek rešerše, takže u každého kritéria uveď zdroj a doslovnou citaci.`,
    { label: "rozhodovací rámec", schema: FRAMEWORK })

  log(`Rozhodovací kritéria: ${framework.criteria.map(c => c.criterion).join(" | ")}`)

  phase("Podklady")
  const materials = framework.requiredMaterials
    .map((m, i) => `- [M${i}] ${m.material} (${m.importance || "podpůrný"}): ${m.why}`)
    .join("\n")
  const inventory = await agent(
    `${BRIEF}

Projdi pracovní adresář uživatele a zjisti, které z níže uvedených podkladů už tam jsou. Pro každý nalezený uveď přesnou cestu k souboru a jednou větou, co v něm je; nečti je celé, stačí ověřit, že jde opravdu o ten dokument. Co nenajdeš, vypiš do 'missing' přesně tak, jak je to pojmenované níže; v 'materialId' vrať jeho značku [M0], [M1], … beze změny.

Potřebné podklady:
${materials}`,
    { label: "inventura podkladů", schema: INVENTORY })

  let supplied = inventory.present
  let stillMissing = inventory.missing

  const inventoryLine = stillMissing.length > 0
    ? `Pro posouzení mi chybí tyto podklady:\n${stillMissing.map(m => `• ${m.material}`).join("\n")}`
    : `V pracovním adresáři jsem našel všechny podklady, které jsem hledal:\n${supplied.map(d => `• ${d.material || d.path}`).join("\n")}`

  const answer = await ask(
    `${inventoryLine}

Soudy v takových věcech posuzují:
${framework.criteria.map(c => `• ${c.criterion}`).join("\n")}

Nahrajte prosím dokumenty, které k tomu máte - smlouvy, obchodní podmínky, výzvy, korespondenci, faktury, protokoly - a do textu napište skutečnosti, které z dokumentů neplynou, ale pro posouzení jsou rozhodné: co bylo ujednáno ústně, jak to fakticky probíhalo, kdy se co stalo. Pokud nic dalšího nemáte, řekněte to a já budu pokračovat s tím, co je, a v závěru to uvedu.`,
    { options: ["Nahrál jsem, co mám", "Nic dalšího nemám, pokračuj"] })

  const statedFacts = answer.answered && answer.text ? answer.text.trim() : ""
  if (!answer.answered) {
    log("Dožádání zůstalo bez odpovědi - pokračuji s tím, co je k dispozici")
  }
  if (statedFacts) {
    log("Uživatel doplnil skutečnosti, které v dokumentech nejsou")
  }
  if (answer.answered && answer.files.length > 0) {
    log(`Uživatel nahrál ${answer.files.length} soubor(ů) - přiřazuji je k požadovaným podkladům`)
    const wanted = stillMissing.length > 0
      ? stillMissing.map(m => `- [${m.materialId}] ${m.material}`)
      : framework.requiredMaterials.map((m, i) => `- [M${i}] ${m.material}`)
    const matched = await agent(
      `${BRIEF}

Uživatel právě nahrál tyto soubory:
${answer.files.map(p => `- ${p}`).join("\n")}

Otevři každý natolik, abys poznal, co to je, a přiřaď ho k požadovanému podkladu níže. Do 'present' dej jen ty, které jsi opravdu pokryl, s jejich materialId a cestou; soubor, který k žádnému z nich nepatří, ale pro věc je relevantní, dej do 'present' také a do 'materialId' mu napiš [X]. Do 'missing' vrať ty požadované podklady, které ani po nahrání nemáš.

Požadované podklady:
${wanted.join("\n")}`,
      { label: "přiřazení nahraných podkladů", schema: INVENTORY })
      .catch(() => null)
    if (matched) {
      supplied = supplied.concat(matched.present)
      stillMissing = matched.missing
    }
  }

  const missingAfterAsk = stillMissing.map(m => m.material)

  const criteriaList = framework.criteria.map(c => `- ${c.criterion}`).join("\n")
  const extracted = await parallel(supplied.map(doc => () =>
    agent(
      `${BRIEF}

Otevři dokument klienta a vytěž z něj skutečnosti rozhodné pro posouzení.

Soubor: ${doc.path}${doc.note ? `\nPoznámka z inventury: ${doc.note}` : ""}

Posuzuje se podle těchto kritérií:
${criteriaList}

Ke každému kritériu, které dokument zmiňuje, vrať jednu skutečnost: co z dokumentu plyne, kde přesně to je (článek, odstavec, strana) a DOSLOVNÝ výňatek. Nevykládej právo a nedomýšlej - jen popiš, co v dokumentu stojí. Kritéria, ke kterým dokument nic neříká, vypiš do 'notCovered'.`,
      { label: `vytěžení: ${doc.path.split("/").pop()}`, schema: FACTS, model: SEARCH_MODEL })
      .catch(() => null)))

  const clientFacts = extracted.filter(Boolean).flatMap(r => r.facts)
  log(`Skutečnosti z podkladů klienta: ${clientFacts.length}`)

  phase("Rešerše")
  const perIssue = await parallel(frame.issues.map(issue => () =>
    agent(
      `${BRIEF}

Zodpověz TUTO dílčí otázku a nic jiného.

Dílčí otázka: ${issue.label} - ${issue.question}${issue.why ? `\nProč je podstatná: ${issue.why}` : ""}

Rozhodovací rámec zjištěný v rekognoskaci (drž se ho, ale ověř si ho vlastní rešerší):
${framework.criteria.map(c => `- ${c.criterion} [${c.source}]`).join("\n")}

Postupuj v tomto pořadí zdrojů a v každém kroku vytěž, co k otázce skutečně patří:
1. PŘEDPISY (CR, u unijního přesahu EU) - znění rozhodných ustanovení.
2. KOMENTÁŘE (COMMENT) - jak se ta ustanovení vykládají.
3. JUDIKATURA (JD, ES) - jak se podle nich fakticky rozhoduje; přednostně vyšší soudy.
4. LITERATURA (LT) - jen pokud po předchozích krocích zbývá sporná otázka.

Tvá zjištění půjdou na ověření a pak do syntézy, takže u každého uveď přesnou citaci (číslo předpisu a ustanovení, spisovou značku) a doslovný výňatek. Do 'docId' vrať identifikátor dokumentu, který jsi SKUTEČNĚ otevřel (to, co jsi předal do 'cdx-cli get cdx://doc/<docId>/text'), a pokud jsi četl konkrétní část, i 'part'. Citace musí být doslovná - ověřovatel ji porovná s otevřeným zdrojem, takže přepsaná nebo zpaměti rekonstruovaná citace zjištění vyřadí. Označ, které je nosné a které jen podpůrné.`,
      { label: issue.label, schema: FINDINGS, model: SEARCH_MODEL })
      .catch(() => ({ findings: [] }))))

  const allFindings = perIssue.filter(Boolean).flatMap(r => r.findings)
  const load = { nosný: 0, podpůrný: 1 }
  const loadBearing = [...allFindings]
    .sort((a, b) => (load[a.weight] ?? 1) - (load[b.weight] ?? 1))
    .slice(0, 14)
  log(`Zjištění: ${allFindings.length}, ověřuji ${loadBearing.length} nosných`)
  if (loadBearing.length === 0) {
    return { question: QUESTION, answer: "Rešerše nepřinesla použitelná zjištění.",
      caveats: "Zdroje se nepodařilo dohledat.", missingMaterials: missingAfterAsk }
  }

  phase("Ověření")
  const verified = await parallel(loadBearing.map(finding => () =>
    agent(
      `${BRIEF}

Ověř, jestli tohle zjištění platí i DNES. Jsi skeptik: hledej důvod, proč neplatí.

Tvrzení: ${finding.proposition}
Zdroj: ${finding.source}
Druh zdroje: ${finding.sourceKind}
Doslovná citace: ${finding.quote}

U předpisu ověř účinné znění ke dni posouzení: nebyl zrušen, novelizován ani nahrazen? U judikatury ověř, že rozhodnutí je stále dobrou autoritou: nebylo překonáno velkým senátem, zrušeno Ústavním soudem ani opuštěno pozdější praxí? U komentáře a literatury ověř, že vychází z účinného znění. Otevři zdroj celý, nesuď podle úryvku. Do 'evidence' napiš rozhodující formulaci - cituje se do závěrečného stanoviska.

Status voli takto: 'supported' jen když jsi zdroj otevřel a tvrzení obstálo; 'refuted' když jsi našel konkrétní důvod, proč neplatí; 'source_unreachable' když se ti zdroj nepodařilo otevřít; 'unverified' když jsi ho otevřel, ale nedokázal jsi tvrzení potvrdit ani vyvrátit. Nedostatek důkazu NENÍ vyvrácení a chybějící přístup NENÍ potvrzení.`,
      { label: `ověření: ${finding.proposition.slice(0, 40)}`, agentType: "codexis:verifier",
        schema: CURRENCY })
      .then(verdict => ({ finding, verdict }))
      .catch(() => ({ finding, verdict: { status: "unverified",
        evidence: "ověření se nepodařilo provést (chyba běhu)" } }))))

  const checked = verified.filter(Boolean)
  const holding = checked.filter(v => v.verdict.status === "supported")
  const refuted = checked.filter(v => v.verdict.status === "refuted")
  const unverified = checked.filter(v => v.verdict.status !== "supported" && v.verdict.status !== "refuted")
  log(`Obstálo ${holding.length} z ${checked.length} zjištění, ${refuted.length} vyvráceno, ${unverified.length} neověřeno`)

  phase("Syntéza")
  const block = holding.map((v, i) =>
    `[${i}] ${v.finding.proposition}
    zdroj (${v.finding.sourceKind}): ${v.finding.source}
    citace: "${v.finding.quote}"${v.verdict.effectiveAt ? `\n    účinné znění k: ${v.verdict.effectiveAt}` : ""}${v.verdict.evidence ? `\n    ověření: ${v.verdict.evidence}` : ""}`).join("\n\n")

  const clientDocuments = clientFacts.length > 0
    ? clientFacts.map(f =>
        `- ${f.fact}
    dokument: ${f.path}${f.locator ? `, ${f.locator}` : ""}
    znění: "${f.quote}"`).join("\n")
    : "(z podkladů klienta se nepodařilo vytěžit žádné skutečnosti)"

  const opinion = await agent(
    `${BRIEF}

Napiš právní stanovisko k otázce výše.

Ověřená zjištění (každé se zdrojem, doslovnou citací a poznámkou z ověření):
${block}

Podklady klienta, na které se závěry aplikují:
${clientDocuments}

${statedFacts ? `Skutečnosti, které klient uvedl sám a NEJSOU ničím doloženy - ber je jako jeho tvrzení, u závěru, který na nich stojí, to výslovně napiš a dej to do caveats:\n${statedFacts}\n\n` : ""}${refuted.length > 0 ? `Tato zjištění byla VYVRÁCENA a nesmíš o ně opřít závěr: ${refuted.map(v => `${v.finding.source} (${v.verdict.evidence})`).join("; ")}\n\n` : ""}${unverified.length > 0 ? `Tato zjištění se NEPODAŘILO OVĚŘIT. Nesmíš o ně opřít závěr ani je citovat jako platná; pokud jsou pro otázku podstatná, uveď v caveats, že v tomto rozsahu je stanovisko předběžné: ${unverified.map(v => `${v.finding.source} (${v.verdict.evidence})`).join("; ")}\n\n` : ""}${missingAfterAsk.length > 0 ? `Tyto podklady chybí a závěr je v jejich rozsahu předběžný: ${missingAfterAsk.join(", ")} - uveď to v caveats.\n\n` : ""}Přečti celý materiál výše, než začneš psát. Vše, o co se smíš opřít, je v něm - nedoplňuj nic z vlastní paměti a nedohledávej nové zdroje. Nejdřív odpověz na otázku přímo, pak rozveď argumentaci po bodech; u každého bodu uveď přesné zdroje z materiálu a napiš, na který konkrétní podklad klienta se aplikuje. Přiřaď míru jistoty. Kde materiál otázku neuzavírá, řekni to a dej to do caveats nebo otevřených otázek.`,
    { label: "stanovisko", agentType: "codexis:synthesizer", schema: OPINION })

  return {
    question: QUESTION,
    ...opinion,
    framework: framework.criteria,
    refutedSources: refuted.map(v => ({ source: v.finding.source, why: v.verdict.evidence })),
    unverifiedSources: unverified.map(v => ({ source: v.finding.source, why: v.verdict.evidence })),
    missingMaterials: missingAfterAsk,
    clientStatedFacts: statedFacts,
    stats: { issues: frame.issues.length, findings: allFindings.length, checked: checked.length,
      holding: holding.length, refuted: refuted.length,
      unverified: unverified.length }
  }
}

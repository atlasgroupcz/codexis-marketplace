// @template legal-research
// @use-when A legal question that has to be answered from the law itself and applied to the client's
//   own documents: find the governing rules, learn what the courts actually decide on, collect the
//   papers that decide it, check nothing is repealed or overruled, then write the opinion.
// @jurisdictions CZ SK EU
// @args question (required) - the legal question, in the user's own words
// @args jurisdiction - CZ, SK or EU; defaults to CZ

const SEARCH_MODEL = "GPT_5_6_LUNA"
const SOL_MODEL = "GPT_5_6_SOL"
const TERRA_MODEL = "GPT_5_6_TERRA"
const MAX_MATERIAL_ASKS = 5

export const meta = {
  name: "legal-research",
  description: "Právní rešerše: rámec → rekognoskace → podklady → rešerše → ověření → syntéza.",
  phases: [
    { title: "Rámec", detail: "Kvalifikace, časový řez, mezinárodní přesah a procesní rámec, z nich sporné otázky" },
    { title: "Rekognoskace", detail: "Zjistit z judikatury a předpisů, podle čeho se rozhoduje a co je třeba doložit" },
    { title: "Podklady", detail: "Inventura dokumentů klienta, dožádání chybějících podkladů a vytěžení rozhodných skutečností" },
    { title: "Rešerše", detail: "Vyhledání předpisů, komentářů, judikatury a literatury po dílčích otázkách" },
    { title: "Ověření", detail: "Platnost, účinnost a překonanost každého nosného zdroje" },
    { title: "Syntéza", detail: "Aplikace ověřených závěrů na podklady klienta" }
  ]
}

const ANGLE = { type: "object", required: ["findings", "issues"], properties: {
  findings: { type: "array", items: { type: "object", required: ["point"], properties: {
    point: { type: "string" }, source: { type: "string" }, why: { type: "string" } } } },
  asOf: { type: "string" },
  issues: { type: "array", items: { type: "object", required: ["label", "question"], properties: {
    label: { type: "string" }, question: { type: "string" }, why: { type: "string" } } } } } }

const FRAME = { type: "object", required: ["area", "issues"], properties: {
  area: { type: "string" }, jurisdiction: { type: "string" }, asOf: { type: "string" },
  constraints: { type: "array", items: { type: "string" } },
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

const CONFLICTS = { type: "object", required: ["conflicts"], properties: {
  conflicts: { type: "array", items: { type: "object", required: ["issue", "sources"], properties: {
    issue: { type: "string" }, sources: { type: "array", items: { type: "string" } },
    impact: { type: "string" } } } },
  gaps: { type: "array", items: { type: "string" } } } }

const OPINION = { type: "object", required: ["answer", "reasoning", "caveats"], properties: {
  answer: { type: "string" },
  reasoning: { type: "array", items: { type: "object", required: ["point", "sources"], properties: {
    point: { type: "string" }, sources: { type: "array", items: { type: "string" } },
    appliesTo: { type: "string" }, confidence: { enum: ["vysoká", "střední", "nízká"] } } } },
  caveats: { type: "string" }, openQuestions: { type: "array", items: { type: "string" } } } }

const FRAME_ANGLES = [
  { label: "kvalifikace a povaha úpravy",
    task: `Urči, ČÍM se ta věc vůbec řídí po hmotné stránce.

1. Do jakého oboru a k jakému institutu věc patří a jak zní její právní kvalifikace - i to, jestli nepřipadá v úvahu souběh (nárok ze smlouvy vedle bezdůvodného obohacení).
2. Jestli je vztah soukromoprávní, nebo veřejnoprávní, a jestli ho omezuje veřejnoprávní regulace.
3. Jestli je úprava dispozitivní, nebo kogentní - a u kogentní, jestli připouští odchylku ve prospěch chráněné strany.
4. Jestli je některá strana zákonem chráněná (spotřebitel, zaměstnanec, nájemce bytu, slabší strana) a co z toho plyne.`
  },
  { label: "časový řez a intertemporalita",
    task: `Urči, KE KTERÉMU DNI a PODLE JAKÉHO ZNĚNÍ se věc posuzuje.

1. Kdy nastaly rozhodné skutečnosti (uzavření smlouvy, vznik škody, doručení) - vyčti data z otázky a z podkladů.
2. Které znění předpisů na ně dopadá: byla úprava novelizována nebo nahrazena a co říkají přechodná (intertemporální) ustanovení.
3. Jestli běží nebo uplynula promlčecí či prekluzivní lhůta a od jaké události se počítá.
4. Den posouzení vrať v 'asOf' jako RRRR-MM-DD; když ho z podkladů nelze určit, vrať dnešní a napiš to do zjištění.`
  },
  { label: "mezinárodní a unijní přesah",
    task: `Urči, jestli věc zůstává čistě vnitrostátní.

1. Má věc mezinárodní prvek (zahraniční strana, plnění či majetek v cizině, volba práva nebo soudu)?
2. Pokud ano: podle čeho se určuje pravomoc a rozhodné právo (Brusel I bis, Řím I, Řím II, ZMPS, mezinárodní smlouva) a k jakému výsledku to vede.
3. Dopadá na věc přímo použitelné nařízení EU, nebo transponovaná směrnice s eurokonformním výkladem?
4. Vnitrostátní povahu řekni výslovně a stručně odůvodni - je to plnohodnotný výsledek.`
  },
  { label: "procesní rámec a důkazní břemeno",
    task: `Urči, JAK se taková věc prosazuje a co je vůbec nutné prokázat.

1. V jakém řízení a před kým se rozhoduje (soud, správní orgán a poté správní soud, rozhodce) a kdo stojí na které straně.
2. Koho u každé sporné skutečnosti tíží břemeno tvrzení a důkazní břemeno a jestli se neuplatní domněnka nebo obrácené břemeno.
3. Jaké podmínky a lhůty musí být splněny dřív, než se lze nároku domoci (předžalobní výzva, vytknutí vady, odvolání, propadné lhůty).
4. Která otázka je rozhodující proto, že na ní stojí unesení břemene.`
  }
]

const EXTRACTION_ANGLES = [
  { label: "rozhodné skutečnosti",
    task: criteria => `Ke každému kritériu, které dokument zmiňuje, vrať jednu skutečnost: co z dokumentu plyne, kde přesně to je (článek, odstavec, strana) a DOSLOVNÝ výňatek. Nevykládej právo a nedomýšlej - jen popiš, co v dokumentu stojí. Kritéria, ke kterým dokument nic neříká, vypiš do 'notCovered'.

Posuzuje se podle těchto kritérií:
${criteria}`
  },
  { label: "chronologie a lhůty",
    task: () => `Vytěž z dokumentu ČASOVOU OSU a nic jiného. Ke každému datu, které v dokumentu je, vrať jednu položku: co se toho dne stalo nebo mělo stát, kde to v dokumentu je a DOSLOVNÝ výňatek.

Zajímá tě: den podpisu a účinnosti, termíny plnění a splatnosti, lhůty a způsob jejich počítání, odeslání a doručení, vytknutí vady či výzva, odstoupení nebo výpověď, doby, po které něco trvalo.

Chybějící datum, které tam mělo být (nedatováno, chybí doklad o doručení), vypiš do 'notCovered'. Lhůty sám nedopočítávej a promlčení neposuzuj.`
  },
  { label: "ujednání, forma a slabá místa",
    task: () => `Vytěž z dokumentu, CO SI STRANY UJEDNALY a jak je dokument sešitý. Ke každé položce vrať, co z dokumentu plyne, kde přesně to je a DOSLOVNÝ výňatek.

Zajímá tě: kdo jsou strany a v jaké roli, kdo a v jakém postavení podepsal (zástupce, prokurista, statutární orgán), odkaz na dokument, který se tím stává součástí (obchodní podmínky, ceník, příloha, rámcová smlouva), a ujednání měnící zákonný režim: volba práva a soudu, rozhodčí doložka, limitace náhrady škody, smluvní pokuta, výhrada vlastnictví, forma změn.

Do 'notCovered' vypiš slabá místa: chybějící podpis nebo datum, odkaz na chybějící přílohu, ručně dopsané či škrtnuté pasáže, nečitelnou nebo chybějící stranu, rozpor uvnitř dokumentu. Platnost neposuzuj - popiš, co vidíš.`
  }
]

export default async function (args) {
  const QUESTION = (typeof args === "object" && args && typeof args.question === "string" && args.question.trim()) || ""
  if (!QUESTION) {
    return { error: "Chybí právní otázka. Předej ji jako args: { question: '<otázka>' }." }
  }
  const JURISDICTION = (args && args.jurisdiction) || "CZ"

  phase("Rámec")
  const SCOPE_BRIEF = `Jsi na začátku právní rešerše a máš na starost JEDEN úhel jejího rámce. Rámec z těchto úhlů se pak skládá dohromady, takže drž se toho svého a cizí neřeš.

Otázka klienta: ${QUESTION}
Předpokládaná jurisdikce: ${JURISDICTION}

Nejde o rešerši do hloubky ani o odpověď na otázku, jen o zařazení věci. V pracovním adresáři jsou podklady; otevři je jen natolik, abys z nich vytěžil to, co potřebuje tvůj úhel. Odpovídej česky. U zjištění, které stojí na předpisu nebo judikatuře, uveď zdroj; nikdy si ho nevymýšlej - když ho nemáš, řekni to.

Z toho, co zjistíš, odvoď dílčí právní otázky, které je kvůli TVÉMU úhlu nutné zodpovědět. Každá musí být samostatně zodpověditelná a nesmí se překrývat s ostatními - dostane vlastního rešeršistu, který neuvidí nic než její zadání. Žádnou si nevymýšlej do počtu; když z tvého úhlu žádná neplyne, vrať prázdný seznam.`

  const angles = await parallel(FRAME_ANGLES.map(a => () =>
    agent(`${SCOPE_BRIEF}

Tvůj úhel: ${a.label}

${a.task}`, { label: `rámec: ${a.label}`, schema: ANGLE, model: SOL_MODEL })))

  const scoped = FRAME_ANGLES.map((a, i) => ({ angle: a, result: angles[i] })).filter(s => s.result)
  if (scoped.length === 0) {
    return { error: "Nepodařilo se určit rámec věci - žádný z úhlů se nevrátil." }
  }
  log(`Rámec z úhlů: ${scoped.map(s => s.angle.label).join(", ")}`)

  const angleReports = scoped.map(s =>
    `## ${s.angle.label}
Zjištění:
${s.result.findings.map(f => `- ${f.point}${f.source ? ` [${f.source}]` : ""}${f.why ? ` (${f.why})` : ""}`).join("\n") || "- (žádné)"}${s.result.asOf ? `\nDen posouzení podle tohoto úhlu: ${s.result.asOf}` : ""}
Navržené dílčí otázky:
${s.result.issues.map(i => `- ${i.label}: ${i.question}${i.why ? ` (proč: ${i.why})` : ""}`).join("\n") || "- (žádné)"}`).join("\n\n")

  const frame = await agent(
    `Rešeršisté nezávisle na sobě ohledali rámec jedné právní věci, každý z jiného úhlu. Slož z toho jeden rámec, ze kterého poběží zbytek rešerše.

Otázka klienta: ${QUESTION}
Předpokládaná jurisdikce: ${JURISDICTION}

${angleReports}

Vrať právní obor ('area'), rozhodnou jurisdikci ('jurisdiction'), den, ke kterému se věc posuzuje ('asOf'), a 3-8 dílčích otázek ('issues').

Otázky sluč napříč úhly: co se opakuje, spoj do jedné; co se překrývá, rozděl tak, aby si dvě nešlapaly do zelí; co je pro odpověď nepodstatné, zahoď. Rozhodující dej první. Formulaci, kterou nespojuješ, ponech tak, jak ji úhel napsal - přeformulováním se ztrácí to, co rešeršista viděl v podkladech.

Do 'constraints' vypiš jednou větou každé omezení rámce, které musí znát další rešeršisté: povahu úpravy (kogentní, dispozitivní, ochrana slabší strany), rozhodné znění a přechodná ustanovení, mezinárodní nebo unijní přesah, rozložení důkazního břemene. Nic si nedomýšlej - ber jen to, co některý z úhlů skutečně zjistil.`,
    { label: "složení rámce", agentType: "codexis:synthesizer", schema: FRAME, model: SOL_MODEL })

  const constraints = frame.constraints && frame.constraints.length > 0
    ? `\nRámcová omezení:\n${frame.constraints.map(c => `- ${c}`).join("\n")}`
    : ""

  const BRIEF = `Právní otázka: ${QUESTION}
Obor: ${frame.area}
Jurisdikce: ${frame.jurisdiction || JURISDICTION}${frame.asOf ? `\nPosuzuje se ke dni: ${frame.asOf}` : ""}${constraints}
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
    { label: "rozhodovací rámec", schema: FRAMEWORK, model: SOL_MODEL })

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
    { label: "inventura podkladů", schema: INVENTORY, model: TERRA_MODEL })

  let supplied = inventory.present
  let stillMissing = inventory.missing

  const specOf = item => {
    const idx = /(\d+)/.exec(String(item.materialId || ""))
    return (idx && framework.requiredMaterials[Number(idx[1])]) || {}
  }
  const decisive = stillMissing.filter(m => specOf(m).importance === "rozhodující")
  const asked = (decisive.length > 0 ? decisive : stillMissing).slice(0, MAX_MATERIAL_ASKS)
  const bundled = stillMissing.filter(m => !asked.includes(m))

  const covered = new Set()
  const loose = []
  const notes = []

  for (const item of asked) {
    const why = specOf(item).why
    const reply = await ask(
      `Máte k dispozici tento podklad: ${item.material}?${why ? `\n\nK čemu ho potřebuji: ${why}` : ""}`,
      { options: ["Nahrávám ho teď", "Nemám ho"] })
    if (!reply.answered) {
      log(`Bez odpovědi: ${item.material}`)
      continue
    }
    if (reply.files.length > 0) {
      supplied = supplied.concat(reply.files.map(path =>
        ({ materialId: item.materialId, material: item.material, path })))
      covered.add(item.materialId)
      log(`${item.material}: ${reply.files.length} soubor(ů)`)
    }
    if (reply.text && reply.text.trim()) {
      notes.push(`K podkladu "${item.material}": ${reply.text.trim()}`)
    }
  }

  if (bundled.length > 0) {
    const rest = await ask(
      `Máte ještě něco z podkladů, které mi zbývají?\n\n${bundled.map(m => `• ${m.material}`).join("\n")}`,
      { options: ["Nahrávám, co z toho mám", "Nic z toho nemám"] })
    if (rest.answered && rest.files.length > 0) {
      loose.push(...rest.files)
    }
    if (rest.answered && rest.text && rest.text.trim()) {
      notes.push(rest.text.trim())
    }
  }

  const answer = await ask(
    `Které skutečnosti jsou pro posouzení rozhodné, ale z dokumentů neplynou - co bylo ujednáno ústně, jak to fakticky probíhalo a kdy se co stalo?`,
    { options: ["Nic dalšího už nemám, pokračuj"] })
  if (!answer.answered) {
    log("Dožádání zůstalo bez odpovědi - pokračuji s tím, co je k dispozici")
  }
  if (answer.answered && answer.text && answer.text.trim()) {
    notes.push(answer.text.trim())
  }
  if (answer.answered && answer.files.length > 0) {
    loose.push(...answer.files)
  }

  const statedFacts = notes.join("\n")
  if (statedFacts) {
    log("Uživatel doplnil skutečnosti, které v dokumentech nejsou")
  }

  stillMissing = stillMissing.filter(m => !covered.has(m.materialId))

  if (loose.length > 0 && stillMissing.length > 0) {
    log(`Přiřazuji ${loose.length} soubor(ů) nahraných mimo konkrétní podklad`)
    const matched = await agent(
      `${BRIEF}

Uživatel právě nahrál tyto soubory:
${loose.map(p => `- ${p}`).join("\n")}

Otevři každý natolik, abys poznal, co to je, a přiřaď ho k požadovanému podkladu níže. Do 'present' dej jen ty, které jsi opravdu pokryl, s jejich materialId a cestou; soubor, který k žádnému z nich nepatří, ale pro věc je relevantní, dej do 'present' také a do 'materialId' mu napiš [X]. Do 'missing' vrať ty požadované podklady, které ani po nahrání nemáš.

Požadované podklady:
${stillMissing.map(m => `- ${m.materialId} ${m.material}`).join("\n")}`,
      { label: "přiřazení nahraných podkladů", schema: INVENTORY, model: TERRA_MODEL })
      .catch(() => null)
    if (matched) {
      supplied = supplied.concat(matched.present)
      stillMissing = matched.missing
    }
  } else if (loose.length > 0) {
    supplied = supplied.concat(loose.map(path => ({ materialId: "[X]", path })))
  }

  const missingAfterAsk = stillMissing.map(m => m.material)

  const criteriaList = framework.criteria.map(c => `- ${c.criterion}`).join("\n")
  const extractionJobs = supplied.flatMap(doc => EXTRACTION_ANGLES.map(angle => ({ doc, angle })))
  const extracted = await parallel(extractionJobs.map(({ doc, angle }) => () =>
    agent(
      `${BRIEF}

Otevři dokument klienta a vytěž z něj to, co patří do TVÉHO úhlu. Stejný dokument souběžně čtou i další rešeršisté z jiných úhlů, takže cizí úhel neřeš a nesnaž se dokument pokrýt celý.

Soubor: ${doc.path}${doc.note ? `\nPoznámka z inventury: ${doc.note}` : ""}
Tvůj úhel: ${angle.label}

${angle.task(criteriaList)}

Ke každé položce vždy uveď cestu k souboru, přesné místo v něm a doslovné znění - tvůj výstup se porovnává se zdrojem a přepsaná citace ho vyřadí.`,
      { label: `${angle.label}: ${doc.path.split("/").pop()}`, schema: FACTS, model: SEARCH_MODEL })))

  const clientFacts = extracted.filter(Boolean).flatMap(r => r.facts)
  const extractionGaps = extracted.filter(Boolean).flatMap(r => r.notCovered || [])
  log(`Skutečnosti z podkladů klienta: ${clientFacts.length} (${extracted.filter(Boolean).length} z ${extractionJobs.length} vytěžení)`)

  const crossCheck = supplied.length > 1 && clientFacts.length > 1
    ? await agent(
        `${BRIEF}

Podklady klienta byly vytěženy po dokumentech, každý zvlášť. Tvým úkolem je najít, kde si NAVZÁJEM ODPORUJÍ nebo kde mezi nimi zeje díra. Nic nedohledávej a nevykládej právo - pracuj jen s tím, co je níže.

Vytěžené skutečnosti:
${clientFacts.map(f => `- ${f.fact}\n    dokument: ${f.path}${f.locator ? `, ${f.locator}` : ""}\n    znění: "${f.quote}"`).join("\n")}

${extractionGaps.length > 0 ? `Co při vytěžení nebylo v dokumentech pokryto (nezodpovězená kritéria, chybějící data, vady dokumentů):\n${extractionGaps.map(g => `- ${g}`).join("\n")}\n\n` : ""}Do 'conflicts' dej dvojice, které spolu nesedí: různá data téže události, různá znění téhož ujednání, plnění, které podle jednoho dokumentu proběhlo a podle druhého ne, dodatek měnící to, co smlouva stanoví jinak. U každého uveď rozcházející se dokumenty a jednou větou, proč na tom záleží.

Do 'gaps' dej neuzavřené řetězce: úkon bez dokladu o doručení, odkaz na přílohu, která mezi podklady není, předpokládaná změna bez dodatku.`,
        { label: "rozpory a mezery mezi podklady", agentType: "codexis:synthesizer", schema: CONFLICTS })
        .catch(() => null)
    : null

  if (crossCheck) {
    log(`Rozpory mezi podklady: ${crossCheck.conflicts.length}, otevřené řetězce: ${(crossCheck.gaps || []).length}`)
  }

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
      { label: `ověření: ${finding.proposition}`, agentType: "codexis:verifier",
        schema: CURRENCY, model: TERRA_MODEL })
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

${crossCheck && crossCheck.conflicts.length > 0 ? `Podklady si v těchto bodech ODPORUJÍ - závěr, který na některém z nich stojí, o tom musí mluvit a patří to do caveats:\n${crossCheck.conflicts.map(c => `- ${c.issue} (${c.sources.join(" vs. ")})${c.impact ? `: ${c.impact}` : ""}`).join("\n")}\n\n` : ""}${crossCheck && crossCheck.gaps && crossCheck.gaps.length > 0 ? `Tyto řetězce nejsou v podkladech uzavřené: ${crossCheck.gaps.join("; ")}\n\n` : ""}${statedFacts ? `Skutečnosti, které klient uvedl sám a NEJSOU ničím doloženy - ber je jako jeho tvrzení, u závěru, který na nich stojí, to výslovně napiš a dej to do caveats:\n${statedFacts}\n\n` : ""}${refuted.length > 0 ? `Tato zjištění byla VYVRÁCENA a nesmíš o ně opřít závěr: ${refuted.map(v => `${v.finding.source} (${v.verdict.evidence})`).join("; ")}\n\n` : ""}${unverified.length > 0 ? `Tato zjištění se NEPODAŘILO OVĚŘIT. Nesmíš o ně opřít závěr ani je citovat jako platná; pokud jsou pro otázku podstatná, uveď v caveats, že v tomto rozsahu je stanovisko předběžné: ${unverified.map(v => `${v.finding.source} (${v.verdict.evidence})`).join("; ")}\n\n` : ""}${missingAfterAsk.length > 0 ? `Tyto podklady chybí a závěr je v jejich rozsahu předběžný: ${missingAfterAsk.join(", ")} - uveď to v caveats.\n\n` : ""}Přečti celý materiál výše, než začneš psát. Vše, o co se smíš opřít, je v něm - nedoplňuj nic z vlastní paměti a nedohledávej nové zdroje. Nejdřív odpověz na otázku přímo, pak rozveď argumentaci po bodech; u každého bodu uveď přesné zdroje z materiálu a napiš, na který konkrétní podklad klienta se aplikuje. Přiřaď míru jistoty. Kde materiál otázku neuzavírá, řekni to a dej to do caveats nebo otevřených otázek.`,
    { label: "stanovisko", agentType: "codexis:synthesizer", schema: OPINION, model: SOL_MODEL })

  return {
    question: QUESTION,
    ...opinion,
    framework: framework.criteria,
    refutedSources: refuted.map(v => ({ source: v.finding.source, why: v.verdict.evidence })),
    unverifiedSources: unverified.map(v => ({ source: v.finding.source, why: v.verdict.evidence })),
    missingMaterials: missingAfterAsk,
    clientStatedFacts: statedFacts,
    frameConstraints: frame.constraints || [],
    documentConflicts: crossCheck ? crossCheck.conflicts : [],
    documentGaps: crossCheck ? crossCheck.gaps || [] : [],
    stats: { frameAngles: scoped.length, issues: frame.issues.length,
      extractions: extracted.filter(Boolean).length, clientFacts: clientFacts.length,
      findings: allFindings.length, checked: checked.length,
      holding: holding.length, refuted: refuted.length,
      unverified: unverified.length }
  }
}

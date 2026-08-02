// @template legal-research
// @use-when A legal question that has to be answered from the law itself and applied to the client's
//   own case: read the file first, find out who the other side is and what is on record about them,
//   then find the governing rules, check nothing is repealed or overruled, and write the opinion.
// @jurisdictions CZ SK EU
// @args question (required) - the legal question, in the user's own words
// @args jurisdiction - CZ, SK or EU; defaults to CZ

const SEARCH_MODEL = "GPT_5_6_LUNA"
const SOL_MODEL = "GPT_5_6_SOL"
const TERRA_MODEL = "GPT_5_6_TERRA"

const MAX_COUNTERPARTIES = 3
const VERIFY_BUDGET = 24

const RULES = `Odpovídej česky. Každé právní tvrzení musí nést přesnou citaci zdroje a doslovný výňatek z něj. Zdroj si nikdy nevymýšlej - když ho nemáš, řekni to.`

export const meta = {
  name: "legal-research",
  description: "Právní rešerše: rekognoskace případu → plán → rešerše → ověření → stanovisko.",
  phases: [
    { title: "Rekognoskace", detail: "Cíl klienta, dokumenty, chronologie, entity a prověření protistrany" },
    { title: "Plán", detail: "Rozhodný právní režim a dílčí otázky, na kterých stojí výsledek" },
    { title: "Rešerše", detail: "Odpověď na každou dílčí otázku v podobě ověřitelných právních propozic" },
    { title: "Ověření", detail: "Sloučení propozic a kontrola, že nosné zdroje platí a sedí na věc" },
    { title: "Stanovisko", detail: "Odpověď, důkazní situace, rizika a doporučený postup" }
  ]
}

const CASE = { type: "object", required: ["goal", "documents", "timeline", "facts"], properties: {
  goal: { type: "string", description: "Čeho chce klient dosáhnout a jaké rozhodnutí na stanovisku závisí. Rozliš prevenci, vyjednávání, podání, obranu a transakci - 'má klient nárok' a 'má teď podat žalobu' jsou různá zadání." },
  documents: { type: "string", description: "Mapa dokumentů v markdownu, jeden řádek na dokument: cesta k souboru, typ, datum, strany, právní význam a vztah k ostatním. Označ, co je originál, návrh, příloha nebo pozdější změna, a co je nečitelné či neúplné." },
  timeline: { type: "string", description: "Chronologie v markdownu: datum, událost, odkud to víš, a jestli je událost prokázaná dokumentem, jen tvrzená, nebo odvozená. Lhůty sám nedopočítávej." },
  facts: { type: "string", description: "Skutkový stav v markdownu ve třech oddílech: jisté skutečnosti, sporné skutečnosti, chybějící rozhodné skutečnosti. U procesního stavu uveď, jestli už běží řízení nebo lhůta." },
  counterparties: { type: "array", description: "Subjekty, jejichž prověření může změnit právní kvalifikaci, důkazy, procesní postup nebo vymahatelnost. Klienta sem nedávej. Když ve věci žádná zjistitelná protistrana není, vrať prázdný seznam - nevymýšlej si ji do počtu.", items: { type: "object", required: ["name", "why"], properties: {
    name: { type: "string", description: "Název nebo jméno tak, jak je uvedeno v podkladech." },
    identifier: { type: "string", description: "IČO, datum narození nebo jiný jednoznačný identifikátor, pokud je v podkladech." },
    why: { type: "string", description: "Co konkrétně na tomto subjektu potřebuješ vědět a proč to může ovlivnit posouzení." } } } },
  blockingQuestions: { type: "array", description: "Jen otázky, jejichž odpověď změní směr práce: použitelný právní režim, jurisdikci, rozhodné znění, běh lhůty, existenci nároku, procesní cestu, identitu protistrany nebo výběr posuzovaného dokumentu. Na dokument, který v adresáři už je, se neptej. Když nic takového nechybí, vrať prázdný seznam.", items: { type: "object", required: ["question", "why"], properties: {
    question: { type: "string", description: "Celá otázka na klienta, jednou větou ve druhé osobě, začínající slovesem. Klient uvidí přesně to, co sem napíšeš, takže žádná uvozovací nálepka. Odkaž se na to, co v podkladech vidíš: 'Ve složce je smlouva ze 4. května 2022 a dodatek č. 2, ale dodatek č. 1, na který oba odkazují, chybí - máte ho?'" },
    why: { type: "string", description: "Proč to potřebuješ, souvislými větami, jak bys to řekl klientovi. Zobrazí se pod otázkou, takže zase bez nálepky." } } } } } }

const PARTY = { type: "object", required: ["subject", "report"], properties: {
  subject: { type: "string", description: "Jednoznačně určený subjekt: název, IČO a stav ke dni kontroly." },
  report: { type: "string", description: "Zjištění v markdownu. U každého uveď zdroj a datum kontroly, samotnou skutečnost, proč může být pro tuto věc významná, a jestli jde o ověřený údaj, indicii nebo neúplný výsledek." } } }

const PLAN = { type: "object", required: ["context", "issues"], properties: {
  context: { type: "string", description: "Hlavička, kterou dostane každý rešeršista doslova, v markdownu a maximálně na dvacet řádků. Patří do ní: otázka klienta a rozhodnutí, které na ní závisí; právní obor a rozhodná jurisdikce; den, ke kterému se věc posuzuje, a rozhodné znění předpisů; povaha úpravy (kogentní, dispozitivní, ochrana slabší strany); unijní nebo mezinárodní přesah; procesní cesta a rozložení důkazního břemene; a co podstatného plyne z prověření protistrany. Piš to jako zadání pro kolegu, ne jako výčet polí." },
  issues: { type: "array", description: "Tři až osm dílčích otázek, rozhodující první. Každá musí být samostatně zodpověditelná, nesmí se překrývat s ostatními a musí být tak konkrétní, aby rešeršista poznal, co by byla negativní odpověď. Žádnou si nevymýšlej do počtu.", items: { type: "object", required: ["label", "question", "why"], properties: {
    label: { type: "string", description: "Krátký název otázky, slouží jako popisek běhu." },
    question: { type: "string", description: "Celá otázka, včetně rozhodného data a rozhodných skutkových okolností. Ne 'odpovědnost za škodu', ale 'může objednatel podle práva účinného k 15. 3. 2023 požadovat náhradu nákladů na odstranění vady, přestože zhotoviteli neposkytl dodatečnou lhůtu'." },
    why: { type: "string", description: "Jak odpověď ovlivní výsledek: co by znamenalo ano a co ne." },
    needs: { type: "string", description: "Které skutečnosti nebo dokumenty klienta tato otázka potřebuje. Uveď cesty k souborům, pokud je má rešeršista otevřít." } } } } } }

const PROPOSITIONS = { type: "object", required: ["propositions"], properties: {
  propositions: { type: "array", description: "Omezený počet právních tvrzení, ne seznam nálezů. Pět citací k jedné právní větě je JEDNA propozice s pěti zdroji. Nepřidávej propozici, která jen opakuje už podloženou.", items: { type: "object", required: ["claim", "impact", "sources", "analysis"], properties: {
    claim: { type: "string", description: "Jedno právní tvrzení, jednou větou, formulované tak, aby šlo ověřit proti zdroji." },
    impact: { enum: ["rozhodující", "významná", "podpůrná"], description: "Rozhodující, když se bez ní změní odpověď na dílčí otázku nebo doporučení klientovi. Významná, když nese argumentaci. Podpůrná, když jen dokresluje." },
    contested: { type: "boolean", description: "True, když existuje protichůdná judikaturní linie nebo doktrinální spor, nebo když to je bod, který protistrana pravděpodobně napadne." },
    temporalRisk: { type: "boolean", description: "True, když tvrzení stojí na nedávno změněném ustanovení, na přechodných ustanoveních, na historickém znění, nebo na rozhodnutí vydaném podle staršího znění." },
    sources: { type: "array", description: "Zdroje, které tvrzení podpírají. Alespoň jeden musí mít primary: true - ten se bude znovu otevírat při ověření.", items: { type: "object", required: ["citation", "quote"], properties: {
      citation: { type: "string", description: "Přesná citace: číslo předpisu a ustanovení, nebo soud a spisová značka." },
      docId: { type: "string", description: "Identifikátor dokumentu, který jsi SKUTEČNĚ otevřel, tedy to, co jsi předal do 'cdx-cli get cdx://doc/<docId>/text'." },
      quote: { type: "string", description: "Doslovný výňatek. Ověřovatel ho porovná s otevřeným zdrojem, takže přepsaná nebo zpaměti rekonstruovaná citace propozici vyřadí." },
      primary: { type: "boolean", description: "True u zdroje, na kterém tvrzení skutečně stojí." } } } },
    analysis: { type: "string", description: "V markdownu: jak se tvrzení aplikuje na konkrétní skutečnosti klienta, v čem se jeho případ od citovaného rozhodnutí liší, jaký je nejsilnější protiargument nebo skutkový rozdíl proti tomuto závěru, a co by závěr změnilo." } } } },
  gaps: { type: "array", description: "Skutečnosti, které pro zodpovězení této otázky chybí.", items: { type: "string" } } } }

const GROUPS = { type: "object", required: ["groups"], properties: {
  groups: { type: "array", description: "Skupiny propozic, které tvrdí totéž. Propozici, která nemá dvojníka, do skupin nedávej.", items: { type: "object", required: ["members", "keep"], properties: {
    members: { type: "array", description: "Čísla propozic v této skupině.", items: { type: "integer" } },
    keep: { type: "integer", description: "Číslo té propozice, jejíž formulace je nejpřesnější. Musí být mezi members." } } } },
  conflicts: { type: "array", description: "Dvojice nebo skupiny propozic, které si navzájem odporují.", items: { type: "object", required: ["members", "note"], properties: {
    members: { type: "array", items: { type: "integer" } },
    note: { type: "string", description: "V čem se rozcházejí a proč na tom pro tuto věc záleží." } } } } } }

const VERDICT = { type: "object", required: ["status", "note"], properties: {
  status: { enum: ["supported", "refuted", "unverified", "source_unreachable"], description: "Supported jen když jsi zdroj otevřel a tvrzení obstálo. Refuted když jsi našel konkrétní důvod, proč neplatí. Source_unreachable když se ti zdroj nepodařilo otevřít. Unverified když jsi ho otevřel, ale nedokázal jsi tvrzení potvrdit ani vyvrátit. Nedostatek důkazu NENÍ vyvrácení a chybějící přístup NENÍ potvrzení." },
  note: { type: "string", description: "Rozhodující formulace ze zdroje a co z ní plyne, včetně účinného znění nebo pozdější judikatury. Cituje se do závěrečného stanoviska." } } }

const OPINION = { type: "object", required: ["answer", "opinion"], properties: {
  answer: { type: "string", description: "Přímá odpověď na otázku klienta, nejvýše několik vět." },
  opinion: { type: "string", description: "Celé stanovisko v markdownu." } } }

const bullets = items => items.map(i => `- ${i}`).join("\n")

export default async function (args) {
  const QUESTION = (typeof args === "object" && args && typeof args.question === "string" && args.question.trim()) || ""
  if (!QUESTION) {
    return { error: "Chybí právní otázka. Předej ji jako args: { question: '<otázka>' }." }
  }
  const JURISDICTION = (args && args.jurisdiction) || "CZ"

  phase("Rekognoskace")
  const dossier = await agent(
    `Přebíráš právní věc. Tvým úkolem NENÍ ji vyřešit ani hledat právo - je jím zjistit, o co jde, dřív než se rozhodne, co se bude rešeršovat.

Otázka klienta: ${QUESTION}
Předpokládaná jurisdikce: ${JURISDICTION}

Projdi pracovní adresář. Dokumenty neotevírej celé a nevytěžuj z nich všechno; otevři je jen natolik, abys u každého poznal, co to je, z kdy to je, kdo jsou strany a jestli může ovlivnit právní kvalifikaci, běh lhůt, vznik nebo zánik práva, důkazní situaci či procesní postup. Detailní rozbor udělá později ten, kdo bude dokument potřebovat pro konkrétní otázku.

Z otázky a z podkladů sestav mapu případu: čeho chce klient dosáhnout, co se stalo a kdy, co je prokázané a co jen tvrzené, kdo jsou zúčastněné osoby a v jakém postavení.

${RULES}`,
    { label: "mapa případu", schema: CASE, model: TERRA_MODEL })

  if (!dossier) {
    return { error: "Nepodařilo se sestavit mapu případu." }
  }

  const blocking = dossier.blockingQuestions || []
  const prompt = blocking.length > 0
    ? `Než pustím rešerši, potřebuju od vás doplnit tohle:\n\n${blocking.map((q, i) => `${i + 1}. ${q.question}\n${q.why}`).join("\n\n")}`
    : `Které skutečnosti jsou pro posouzení rozhodné, ale z dokumentů neplynou - co bylo ujednáno ústně, jak to fakticky probíhalo a kdy se co stalo?`

  const reply = await ask(prompt, { options: ["Nic z toho nemám, pokračuj"] })
  const stated = reply.answered && reply.text && reply.text.trim() ? reply.text.trim() : ""
  const uploaded = reply.answered ? reply.files : []
  if (!reply.answered) {
    log("Dotaz zůstal bez odpovědi - pokračuji s tím, co je k dispozici")
  }
  if (uploaded.length > 0) {
    log(`Klient nahrál ${uploaded.length} soubor(ů)`)
  }

  const targets = (dossier.counterparties || []).slice(0, MAX_COUNTERPARTIES)
  let parties = []
  if (targets.length > 0) {
    log(`Prověřuji: ${targets.map(t => t.name).join(", ")}`)
    parties = (await parallel(targets.map(t => () =>
      agent(
        `Prověř tento subjekt ve vztahu k popsané věci. Nedělej obecný profil společnosti - vrať jen to, co může ovlivnit právní kvalifikaci, důkazy, procesní postup, vymahatelnost nebo vyjednávací strategii.

Subjekt: ${t.name}${t.identifier ? `\nIdentifikátor: ${t.identifier}` : ""}
Co je na něm potřeba zjistit: ${t.why}

Věc, ve které vystupuje:
${QUESTION}

Nejdřív jednoznačně ověř identitu. Hlavním klíčem je IČO, ne název; název používej ve všech historických a pravopisných variantách. Pak projdi:
1. veřejný rejstřík - aktuální i historické údaje, změny názvu, sídla a právní formy, přeměny, likvidaci, právní nástupnictví,
2. současné a bývalé statutární orgány a prokuristy, způsob jednání a jejich angažmá v jiných společnostech,
3. sbírku listin - zejména dokumenty rozhodné pro oprávnění jednat a pro majetkovou situaci,
4. insolvenční rejstřík, včetně spřízněných dlužníků,
5. registr smluv, pokud subjekt obchoduje s veřejným sektorem,
6. judikaturu a veřejně dostupná řízení - hledej podle současného i historických názvů, podle statutárních osob, spřízněných společností a právních nástupců,
7. další registry podle předmětu věci (katastr, ochranné známky, oborové evidence).

Použij nástroje a dovednosti, které na to v prostředí máš; co není dostupné, přeskoč a napiš to.

Prázdný výsledek fulltextového hledání NENÍ důkaz, že spor, smlouva nebo skutečnost neexistuje - rozhodnutí bývají anonymizovaná nebo nezveřejněná. Takový výsledek označ jako neúplný, ne jako negativní zjištění.

${RULES}`,
        { label: `prověření: ${t.name}`, schema: PARTY, model: SEARCH_MODEL })
        .catch(() => null))))
      .filter(Boolean)
  }

  const partyBlock = parties.length > 0
    ? `\n\nPrověření protistrany:\n${parties.map(p => `### ${p.subject}\n${p.report}`).join("\n\n")}`
    : ""
  const uploadedBlock = uploaded.length > 0
    ? `\n\nSoubory, které klient právě nahrál:\n${bullets(uploaded)}`
    : ""
  const statedBlock = stated
    ? `\n\nCo klient uvedl sám a NENÍ to ničím doloženo (jeho tvrzení):\n${stated}`
    : ""

  const casePacket = `Otázka klienta: ${QUESTION}
Předpokládaná jurisdikce: ${JURISDICTION}

## Cíl klienta
${dossier.goal}

## Dokumenty
${dossier.documents}

## Chronologie
${dossier.timeline}

## Skutkový stav
${dossier.facts}${statedBlock}${uploadedBlock}${partyBlock}`

  phase("Plán")
  const plan = await agent(
    `Máš hotovou rekognoskaci případu. Převeď ji na omezený počet otázek, jejichž zodpovězení vede k praktickému závěru.

${casePacket}

Projdi věc ve čtyřech rovinách a to, co z nich plyne, promítni do hlavičky i do otázek:
1. Kvalifikace a povaha úpravy - do jakého institutu věc patří, jestli nepřipadá v úvahu souběh nároků, jestli je úprava kogentní nebo dispozitivní a jestli je některá strana zákonem chráněná.
2. Časový řez - kdy nastaly rozhodné skutečnosti, které znění předpisů na ně dopadá, co říkají přechodná ustanovení a jestli běží nebo uplynula promlčecí či prekluzivní lhůta.
3. Unijní a mezinárodní přesah - jestli má věc mezinárodní prvek, podle čeho se určuje pravomoc a rozhodné právo, jestli dopadá přímo použitelné nařízení nebo transponovaná směrnice. Čistě vnitrostátní povahu řekni výslovně.
4. Procesní rámec - v jakém řízení a před kým se rozhoduje, koho u které skutečnosti tíží důkazní břemeno, jaké podmínky a lhůty musí být splněny dřív, než se lze nároku domoci.

Nejde ještě o rešerši. Kde si nejsi jistý, která kvalifikace platí, uveď obě a udělej z rozhodnutí mezi nimi dílčí otázku.

${RULES}`,
    { label: "rozhodné otázky", agentType: "codexis:synthesizer", schema: PLAN, model: SOL_MODEL })

  if (!plan || !plan.issues || plan.issues.length === 0) {
    return { error: "Nepodařilo se sestavit rešeršní plán.", casePacket }
  }
  const context = plan.context || casePacket
  log(`Dílčí otázky: ${plan.issues.map(i => i.label).join(", ")}`)

  phase("Rešerše")
  const researched = await parallel(plan.issues.map((issue, idx) => () =>
    agent(
      `${context}

Zodpověz TUTO dílčí otázku a nic jiného. Nesnaž se pokrýt celou věc - ostatní otázky souběžně řeší jiní rešeršisté.

Tvoje otázka: ${issue.question}
Jak odpověď ovlivní výsledek: ${issue.why}${issue.needs ? `\nCo k tomu potřebuješ z podkladů klienta: ${issue.needs}` : ""}

Souběžně se řeší tyto otázky, do kterých nezasahuj:
${bullets(plan.issues.filter((_, i) => i !== idx).map(i => i.label)) || "- (žádné)"}

Postupuj takto:
1. Zorientuj se v komentáři (COMMENT) nebo přehledové literatuře - slouží ti jako mapa a jako cesta k dalším rozhodnutím, ne jako důkaz.
2. Otevři rozhodné znění předpisu (CR, u unijního přesahu EU) včetně přechodných ustanovení.
3. Projdi judikaturu (JD, ES). Hledej víc způsoby: podle ustanovení, podle institutu, podle rozhodné skutkové okolnosti, podle formulace právní věty, a podle rozhodnutí, která nalezený názor později citují nebo se od něj odchylují. Přednostně vyšší soudy.
4. Literaturu (LT) použij jen na spornou otázku, která po předchozích krocích zbývá.
5. Závěr převzatý z komentáře nebo článku ověř zpátky v textu předpisu nebo v citovaném rozhodnutí.

Když tvoje otázka potřebuje dokument klienta, otevři si ho sám a vytěž z něj jen to, co k ní patří.

Dvě věci jsou povinné. Za prvé hledej i PROTI vlastnímu předběžnému závěru: jaký je nejsilnější zdroj nebo skutkový rozdíl, který ho vyvrací. Za druhé u citovaného rozhodnutí posuď, jestli vůbec sedí na tvůj případ - jiné znění zákona, jiná smluvní konstrukce, jiná procesní situace nebo věta, která nebyla nosným důvodem rozhodnutí, ho činí nepoužitelným.

Skonči, jakmile je otázka zodpovězená a podložená. Další nález, který jen znovu podpírá už podloženou propozici, nepřidávej - přidej ho jako zdroj k té propozici.`,
      { label: issue.label, schema: PROPOSITIONS, model: SEARCH_MODEL })
      .catch(() => null)))

  const all = researched.filter(Boolean).flatMap(r => r.propositions || [])
  const gaps = researched.filter(Boolean).flatMap(r => r.gaps || [])
  log(`Propozice: ${all.length} z ${researched.filter(Boolean).length} zodpovězených otázek`)
  if (all.length === 0) {
    return { question: QUESTION, answer: "Rešerše nepřinesla použitelné právní závěry.",
      opinion: "Zdroje se nepodařilo dohledat.", casePacket }
  }

  phase("Ověření")
  const grouping = await agent(
    `Rešeršisté pracovali na dílčích otázkách nezávisle, takže některé propozice tvrdí totéž a některé si odporují. Najdi obojí.

Nic nepřeformulovávej a nic nedohledávej - vracíš POUZE čísla propozic.

${all.map((p, i) => `[${i}] (${p.impact}) ${p.claim}\n    zdroje: ${(p.sources || []).map(s => s.citation).join("; ") || "(žádné)"}`).join("\n\n")}

Do 'groups' dej skupiny propozic, které tvrdí totéž, a u každé urči, kterou formulaci nechat. Dvě propozice o stejném ustanovení, které ale tvrdí něco jiného, NEJSOU duplicita.

Do 'conflicts' dej propozice, které se navzájem vylučují: opačný závěr o téže otázce, různé rozhodné znění, neslučitelný skutkový předpoklad, konkurenční judikaturní linie.`,
    { label: "sloučení propozic", agentType: "codexis:synthesizer", schema: GROUPS, model: TERRA_MODEL })
    .catch(() => null)

  const rank = { "rozhodující": 0, "významná": 1, "podpůrná": 2 }
  const weight = p => rank[p.impact] ?? 2
  const risky = p => p.contested === true || p.temporalRisk === true
  const valid = i => Number.isInteger(i) && i >= 0 && i < all.length

  const dropped = new Set()
  for (const group of (grouping && grouping.groups) || []) {
    const members = (group.members || []).filter(valid)
    const keep = valid(group.keep) && members.includes(group.keep) ? group.keep : members[0]
    if (members.length < 2 || keep === undefined || dropped.has(keep)) {
      continue
    }
    const winner = all[keep]
    for (const member of members) {
      if (member === keep) {
        continue
      }
      const loser = all[member]
      const seen = new Set((winner.sources || []).map(s => s.citation))
      winner.sources = (winner.sources || []).concat((loser.sources || []).filter(s => !seen.has(s.citation)))
      winner.contested = winner.contested || loser.contested
      winner.temporalRisk = winner.temporalRisk || loser.temporalRisk
      if (weight(loser) < weight(winner)) {
        winner.impact = loser.impact
      }
      dropped.add(member)
    }
  }

  const conflicts = ((grouping && grouping.conflicts) || [])
    .map(c => ({ note: c.note, claims: (c.members || []).filter(valid).map(i => all[i].claim) }))
    .filter(c => c.claims.length > 1)

  const merged = all.filter((_, i) => !dropped.has(i))
  const must = merged.filter(p => weight(p) === 0 || (weight(p) === 1 && risky(p)))
  const mustSet = new Set(must)
  const optional = merged.filter(p => !mustSet.has(p))
    .sort((a, b) => weight(a) - weight(b) || (risky(b) ? 1 : 0) - (risky(a) ? 1 : 0))
  const selected = must.concat(optional.slice(0, Math.max(0, VERIFY_BUDGET - must.length)))
  const skipped = merged.filter(p => !selected.includes(p))
  log(`Slučuji ${all.length} propozic na ${merged.length}, ověřuji ${selected.length}${conflicts.length > 0 ? `, rozporů: ${conflicts.length}` : ""}`)

  const verified = await parallel(selected.map(p => () =>
    agent(
      `${context}

Ověř tuto propozici. Jsi skeptik: hledej důvod, proč neplatí nebo proč na tuhle věc nesedí.

Tvrzení: ${p.claim}

Zdroje:
${(p.sources || []).map(s => `- ${s.citation}${s.primary ? " [nosný]" : ""}${s.docId ? ` (docId ${s.docId})` : ""}\n    "${s.quote}"`).join("\n")}

Otevři nosný zdroj CELÝ, nesuď podle předaného výňatku, a zkontroluj:
1. dokument existuje a citace v něm doslova je,
2. výňatek není vytržený z kontextu a nepatří jen do rekapitulace tvrzení účastníků nebo do odlišného stanoviska,
3. rozhodnutí řešilo stejnou právní otázku a vycházelo ze stejného znění předpisu,
4. u předpisu: účinné znění ke dni posouzení, nebyl zrušen, novelizován ani nahrazen,
5. u judikatury: nebylo překonáno velkým senátem, zrušeno Ústavním soudem ani opuštěno pozdější praxí, a nejde jen o obiter dictum,
6. tvrzení není širší, než co zdroj skutečně říká,
7. neexistuje protichůdná linie.

Selže-li kterýkoli z bodů 2, 3 nebo 6, je to 'refuted' - i když dokument existuje a citace sedí.`,
      { label: p.claim, agentType: "codexis:verifier", schema: VERDICT, model: TERRA_MODEL })
      .then(verdict => ({ p, verdict }))
      .catch(() => ({ p, verdict: { status: "unverified", note: "ověření se nepodařilo provést (chyba běhu)" } }))))

  const checked = verified.filter(Boolean)
  const holding = checked.filter(v => v.verdict.status === "supported")
  const refuted = checked.filter(v => v.verdict.status === "refuted")
  const unverified = checked.filter(v => v.verdict.status !== "supported" && v.verdict.status !== "refuted")
  log(`Obstálo ${holding.length} z ${checked.length}, vyvráceno ${refuted.length}, neověřeno ${unverified.length}`)

  phase("Stanovisko")
  const held = holding.map(v =>
    `### ${v.p.claim}
Váha: ${v.p.impact}
Zdroje: ${(v.p.sources || []).map(s => `${s.citation} - "${s.quote}"`).join(" | ")}
Ověření: ${v.verdict.note}

${v.p.analysis}`).join("\n\n")

  const opinion = await agent(
    `Napiš právní stanovisko. Všechno, o co se smíš opřít, je níže - nedoplňuj nic z paměti a nedohledávej nové zdroje.

${context}

${casePacket}

## Ověřené propozice
${held || "(žádná propozice neobstála)"}
${conflicts.length > 0 ? `\n## Propozice, které si odporují\n${conflicts.map(c => `- ${c.claims.join(" VS. ")}: ${c.note}`).join("\n")}\n` : ""}${refuted.length > 0 ? `\n## VYVRÁCENO - nesmíš o to opřít závěr ani to citovat jako platné\n${refuted.map(v => `- ${v.p.claim} (${v.verdict.note})`).join("\n")}\n` : ""}${unverified.length > 0 ? `\n## NEOVĚŘENO - nesmíš o to opřít závěr; je-li to pro věc podstatné, napiš, že stanovisko je v tom rozsahu předběžné\n${unverified.map(v => `- ${v.p.claim} (${v.verdict.note})`).join("\n")}\n` : ""}${skipped.length > 0 ? `\n## Podpůrné, neprošlo ověřením - použij nanejvýš jako dokreslení a označ to\n${bullets(skipped.map(p => p.claim))}\n` : ""}${gaps.length > 0 ? `\n## Skutečnosti, které při rešerši chyběly\n${bullets(gaps)}\n` : ""}
Do 'opinion' napiš celé stanovisko v tomto pořadí: zadání a praktický cíl; krátká odpověď; rozhodné skutečnosti a předpoklady, ze kterých vycházíš; použitelné právo a rozhodné znění; rozbor po jednotlivých otázkách; argumenty ve prospěch klienta; nejsilnější argumenty protistrany; důkazní situace, tedy co je čím doloženo a co by bylo třeba prokázat; procesní, časová a vymahatelnostní rizika; varianty podle skutečností, které zůstaly neznámé; doporučený další postup; a co je třeba ještě získat nebo ověřit.

Než to odevzdáš, projdi si: Odpovídáš na praktickou otázku klienta, nebo jen na tu právní? Nezaměnil jsi právní možnost za pravděpodobný výsledek? Je každé klíčové tvrzení doložené? Vysvětlil jsi skutkové rozdíly oproti citované judikatuře? Uvedl jsi nejsilnější protiargument? Je jasné, které závěry stojí na neprokázané skutečnosti? Je doporučení uskutečnitelné vzhledem k protistraně, lhůtám a vymahatelnosti?

Kde materiál otázku neuzavírá, řekni to. Skutečnost, kterou nikdo neprokázal, se nesmí stát mlčky předpokladem.

${RULES}`,
    { label: "stanovisko", agentType: "codexis:synthesizer", schema: OPINION, model: SOL_MODEL })

  if (!opinion) {
    return { error: "Nepodařilo se sepsat stanovisko.", casePacket,
      propositions: merged.map(p => p.claim) }
  }

  return {
    question: QUESTION,
    ...opinion,
    counterparties: parties.map(p => ({ subject: p.subject, report: p.report })),
    conflicts,
    refutedPropositions: refuted.map(v => ({ claim: v.p.claim, why: v.verdict.note })),
    unverifiedPropositions: unverified.map(v => ({ claim: v.p.claim, why: v.verdict.note })),
    openFacts: gaps,
    clientStatedFacts: stated,
    stats: { issues: plan.issues.length, propositions: all.length, merged: merged.length,
      verified: checked.length, holding: holding.length, refuted: refuted.length,
      unverified: unverified.length, counterparties: parties.length }
  }
}

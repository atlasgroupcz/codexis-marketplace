// @template legal-research
// @use-when A legal question that has to be answered from the law itself and applied to the client's
//   own case: read the file first, find out who the other side is and what is on record about them,
//   then find the governing rules, check nothing is repealed or overruled, and write the opinion.
// @jurisdictions CZ SK EU
// @args question (required) - the legal question, in the user's own words
// @args jurisdiction - CZ, SK or EU; defaults to CZ

const MODEL = "GPT_5_6_LUNA"

const MAX_COUNTERPARTIES = 3
const VERIFY_BUDGET = 24

const RULES = `Odpovídej česky. Každé právní tvrzení musí nést přesnou citaci zdroje a doslovný výňatek z něj. Zdroj si nikdy nevymýšlej - když ho nemáš, řekni to.

U každého zdroje uveď jeho citaci a k tomu, odkud je - odkaz, název databáze či publikace, cestu k souboru. Bez toho si klient zdroj nedohledá.`

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
  goal: { type: "string", description: "Čeho chce klient dosáhnout a jaké rozhodnutí na stanovisku závisí - 'má klient nárok' a 'má teď podat žalobu' jsou různá zadání." },
  documents: { type: "string", description: "Mapa dokumentů v markdownu, řádek na dokument: cesta, typ, datum, strany, právní význam, vztah k ostatním. Označ originál, návrh, přílohu i pozdější změnu a co je neúplné." },
  timeline: { type: "string", description: "Chronologie v markdownu: datum, událost, odkud to víš a jestli je prokázaná dokumentem, jen tvrzená, nebo odvozená. Lhůty nedopočítávej." },
  facts: { type: "string", description: "Skutkový stav v markdownu ve třech oddílech: jisté, sporné, chybějící rozhodné skutečnosti. U procesního stavu uveď, jestli běží řízení nebo lhůta." },
  counterparties: { type: "array", description: "Subjekty, jejichž prověření může změnit právní kvalifikaci, důkazy, procesní postup nebo vymahatelnost. Klienta sem nedávej. Když ve věci žádná zjistitelná protistrana není, vrať prázdný seznam - nevymýšlej si ji do počtu.", items: { type: "object", required: ["name", "why"], properties: {
    name: { type: "string", description: "Název nebo jméno tak, jak je uvedeno v podkladech." },
    identifier: { type: "string", description: "IČO, datum narození nebo jiný jednoznačný identifikátor, pokud je v podkladech." },
    why: { type: "string", description: "Co konkrétně na tomto subjektu potřebuješ vědět a proč to může ovlivnit posouzení." } } } },
  blockingQuestions: { type: "array", description: "Jen otázky, jejichž odpověď změní směr práce: použitelný právní režim, jurisdikci, rozhodné znění, běh lhůty, existenci nároku, procesní cestu, identitu protistrany nebo výběr posuzovaného dokumentu. Na dokument, který v adresáři už je, se neptej. Když nic takového nechybí, vrať prázdný seznam.", items: { type: "object", required: ["question", "why"], properties: {
    question: { type: "string", description: "Celá otázka na klienta, jednou větou ve druhé osobě, začínající slovesem, bez uvozovací nálepky. Odkaž se na to, co v podkladech vidíš: 'Ve složce je smlouva a dodatek č. 2, ale dodatek č. 1, na který oba odkazují, chybí - máte ho?'" },
    why: { type: "string", description: "Proč to potřebuješ, souvislými větami, jak bys to řekl klientovi. Zobrazí se pod otázkou, zase bez nálepky." } } } } } }

const PARTY = { type: "object", required: ["subject", "report"], properties: {
  subject: { type: "string", description: "Jednoznačně určený subjekt: název, IČO a stav ke dni kontroly." },
  report: { type: "string", description: "Zjištění v markdownu. U každého uveď zdroj a datum kontroly, samotnou skutečnost, proč může být pro tuto věc významná, a jestli jde o ověřený údaj, indicii nebo neúplný výsledek." } } }

const PLAN = { type: "object", required: ["context", "issues"], properties: {
  context: { type: "string", description: "Hlavička, kterou dostane každý rešeršista doslova, v markdownu, do dvaceti řádků. Patří do ní: otázka klienta a rozhodnutí, které na ní závisí; obor a jurisdikce; den posouzení a rozhodné znění; povaha úpravy a ochrana slabší strany; unijní přesah; procesní cesta a důkazní břemeno; předpoklady, pod kterými se pracuje, když chybí podklady; a co podstatného plyne z prověření protistrany. Piš to jako zadání pro kolegu, ne jako výčet polí." },
  issues: { type: "array", description: "Tři až osm dílčích otázek, rozhodující první. Každá musí být samostatně zodpověditelná, nesmí se překrývat s ostatními a musí být tak konkrétní, aby rešeršista poznal, co by byla negativní odpověď. Žádnou si nevymýšlej do počtu.", items: { type: "object", required: ["label", "question", "why"], properties: {
    label: { type: "string", description: "Krátký název otázky, slouží jako popisek běhu." },
    question: { type: "string", description: "Celá otázka, včetně rozhodného data a rozhodných skutkových okolností. Ne 'odpovědnost za škodu', ale 'může objednatel podle práva účinného k 15. 3. 2023 požadovat náhradu nákladů na odstranění vady, přestože zhotoviteli neposkytl dodatečnou lhůtu'." },
    why: { type: "string", description: "Jak odpověď ovlivní výsledek: co by znamenalo ano a co ne." },
    needs: { type: "string", description: "Které skutečnosti nebo dokumenty klienta tato otázka potřebuje. Uveď cesty k souborům, pokud je má rešeršista otevřít." } } } } } }

const PROPOSITIONS = { type: "object", required: ["propositions"], properties: {
  propositions: { type: "array", description: "Omezený počet právních tvrzení, ne seznam nálezů. Pět citací k jedné právní větě je JEDNA propozice s pěti zdroji. Nepřidávej propozici, která jen opakuje už podloženou.", items: { type: "object", required: ["claim", "impact", "sources", "analysis"], properties: {
    claim: { type: "string", description: "Jedno právní tvrzení, jednou větou, formulované tak, aby šlo ověřit proti zdroji." },
    impact: { enum: ["rozhodující", "významná", "podpůrná"], description: "Polož si otázku: změnil by OPAČNÝ závěr doporučení klientovi? Ano = rozhodující. Ne, ale podstatně mění riziko nebo rozsah nároku = významná. Ne = podpůrná. Většina propozic je významná nebo podpůrná; když označíš za rozhodující víc než pár, děláš to špatně." },
    contested: { type: "boolean", description: "True jen když znáš konkrétní protichůdné rozhodnutí nebo doktrinální spor, ne když to tak jen tušíš." },
    temporalRisk: { type: "boolean", description: "True jen když umíš pojmenovat důvod: novela, přechodné ustanovení, historické znění, rozhodnutí podle staršího znění, nedávný obrat praxe. Ten důvod napiš do 'analysis'. Bez konkrétního důvodu vrať false." },
    sources: { type: "array", description: "Zdroje, které tvrzení podpírají. Alespoň jeden musí mít primary: true - ten se bude znovu otevírat při ověření.", items: { type: "object", required: ["citation", "quote"], properties: {
      citation: { type: "string", description: "Přesná citace: číslo předpisu a ustanovení, nebo soud a spisová značka." },
      source: { type: "string", description: "Odkud to je, aby si to šlo dohledat: odkaz, název databáze či publikace, cesta k souboru klienta. Opiš to přesně tak, jak to máš od nástroje, a nikdy si to nedomýšlej." },
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
  status: { enum: ["supported", "narrowed", "refuted", "unverified", "source_unreachable"], description: "Supported když jsi zdroj otevřel a tvrzení obstálo. Narrowed když jádro tvrzení ze zdroje plyne, ale formulace je širší, nepřesná nebo opomíjí výjimku - tvrzení se pak použije v opravené podobě, kterou napíšeš do 'narrowedClaim'. Refuted jen když zdroj tvrzení vyvrací nebo o něm vůbec není, takže se nedá zachránit zúžením. Source_unreachable když se ti zdroj nepodařilo otevřít. Unverified když jsi ho otevřel, ale nedokázal jsi tvrzení potvrdit ani vyvrátit. Nedostatek důkazu NENÍ vyvrácení a chybějící přístup NENÍ potvrzení." },
  narrowedClaim: { type: "string", description: "Povinné u 'narrowed': tvrzení přeformulované tak, aby přesně odpovídalo zdroji. Piš celou větu, ne jen poznámku o tom, co je špatně." },
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
    { label: "mapa případu", schema: CASE, model: MODEL })

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

U KAŽDÉ kontroly napiš, jak dopadla, i když nedopadla: "k datu X podle IČO nenalezeno nic", nebo "nezkontrolováno, zdroj nebyl dostupný", nebo "nezkontrolováno, nemám přístup". Souhrnné "nepodařilo se ověřit" je bezcenné - nepozná se z něj, jestli jde o negativní nález, nedostupný rejstřík, nebo že jsi to neudělal. U nálezu vždy uveď, čeho se týká (soud, značka, datum, účastníci) a jak jsi ověřil, že jde o tentýž subjekt.

Prázdný fulltext NENÍ důkaz, že spor neexistuje - rozhodnutí bývají anonymizovaná nebo nezveřejněná. Označ ho jako neúplný, ne jako negativní zjištění.

${RULES}`,
        { label: `prověření: ${t.name}`, schema: PARTY, model: MODEL })
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

První dílčí otázka musí vždy vymezit, CO bylo obsahem závazku a v čem konkrétně měla druhá strana pochybit - tedy podle jakého měřítka se pozná porušení. Teprve pak smějí přijít otázky o následcích (ukončení, vrácení plnění, náhrada, sankce). Ptát se na následky dřív, než je definovaná porušená povinnost, je nejčastější způsob, jak rešerše mine podstatu věci.

Nejde ještě o rešerši. Kde si nejsi jistý, která kvalifikace platí, uveď obě a udělej z rozhodnutí mezi nimi dílčí otázku.

Podklady klienta mohou chybět, a tebe to nesmí zastavit. Neodkládej otázky s tím, že se nejdřív musí něco doložit - formuluj je tak, aby na ně šlo odpovědět z práva pod výslovným předpokladem, a ten předpoklad napiš do hlavičky. Rešeršista musí vědět, co má zjistit, i když spis není k dispozici.

${RULES}`,
    { label: "rozhodné otázky", agentType: "codexis:synthesizer", schema: PLAN, model: MODEL })

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
1. Zorientuj se v komentáři nebo přehledové literatuře - slouží ti jako mapa a jako cesta k dalším rozhodnutím, ne jako důkaz.
2. Otevři rozhodné znění předpisu včetně přechodných ustanovení; u unijního přesahu i příslušný unijní předpis.
3. Projdi judikaturu, u unijního přesahu i evropskou. Hledej víc způsoby: podle ustanovení, podle institutu, podle rozhodné skutkové okolnosti, podle formulace právní věty, a podle rozhodnutí, která nalezený názor později citují nebo se od něj odchylují. Přednostně vyšší soudy.
4. Odbornou literaturu použij jen na spornou otázku, která po předchozích krocích zbývá.
5. Závěr převzatý z komentáře nebo článku ověř zpátky v textu předpisu nebo v citovaném rozhodnutí.

Když tvoje otázka potřebuje dokument klienta, otevři si ho sám a vytěž z něj jen to, co k ní patří.

Povinně hledej i PROTI vlastnímu závěru: jaký je nejsilnější zdroj nebo skutkový rozdíl, který ho vyvrací. A u každého rozhodnutí posuď, jestli vůbec sedí na tvůj případ - jiné znění zákona, jiná konstrukce, jiná procesní situace nebo věta, která nebyla nosným důvodem, ho činí nepoužitelným.

Skonči, jakmile je otázka zodpovězená a podložená. Další nález, který jen znovu podpírá už podloženou propozici, nepřidávej - přidej ho jako zdroj k té propozici.`,
      { label: issue.label, schema: PROPOSITIONS, model: MODEL })
      .catch(() => null)))

  const all = researched.flatMap((r, i) => ((r && r.propositions) || [])
    .map(p => ({ ...p, issue: plan.issues[i].label })))
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
    { label: "sloučení propozic", agentType: "codexis:synthesizer", schema: GROUPS, model: MODEL })
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
${(p.sources || []).map(s => `- ${s.citation}${s.primary ? " [nosný]" : ""}${s.source ? `\n    odkud: ${s.source}` : ""}\n    "${s.quote}"`).join("\n")}

Otevři nosný zdroj CELÝ, nesuď podle předaného výňatku, a zkontroluj:
1. dokument existuje a citace v něm doslova je,
2. výňatek není vytržený z kontextu ani z rekapitulace tvrzení účastníků či odlišného stanoviska,
3. zdroj řeší stejnou otázku a stejné znění předpisu,
4. u předpisu: účinné znění ke dni posouzení, nebyl zrušen ani nahrazen,
5. u judikatury: nebylo překonáno ani opuštěno pozdější praxí a nejde o obiter dictum,
6. tvrzení není širší, než co zdroj říká,
7. neexistuje protichůdná linie.

Zakopnutí na bodech 2, 3 nebo 6 skoro nikdy neznamená 'refuted' - tvrzení je jen širší, než co zdroj unese, a užší verze platí. Vrať 'narrowed' a napiš ji. Zahodit doložitelný závěr kvůli nepřesné formulaci je horší chyba než ho zúžit.`,
      { label: p.claim, agentType: "codexis:verifier", schema: VERDICT, model: MODEL })
      .then(verdict => ({ p, verdict }))
      .catch(() => ({ p, verdict: { status: "run_error", note: "ověření se nepodařilo provést (chyba běhu)" } }))))

  const checked = verified.filter(Boolean)
  for (const v of checked) {
    if (v.verdict.status === "narrowed" && v.verdict.narrowedClaim) {
      v.p = { ...v.p, claim: v.verdict.narrowedClaim, narrowed: true }
    }
  }
  const holding = checked.filter(v => v.verdict.status === "supported" || v.verdict.status === "narrowed")
  const refuted = checked.filter(v => v.verdict.status === "refuted")
  const brokenRun = checked.filter(v => v.verdict.status === "run_error")
  const unverified = checked.filter(v =>
    v.verdict.status === "unverified" || v.verdict.status === "source_unreachable")
  log(`Obstálo ${holding.length} z ${checked.length}, vyvráceno ${refuted.length}, neověřeno ${unverified.length}${brokenRun.length > 0 ? `, ${brokenRun.length} ověření spadlo na chybu` : ""}`)

  phase("Stanovisko")
  const held = holding.map(v =>
    `### ${v.p.claim}
Váha: ${v.p.impact}
Zdroje: ${(v.p.sources || []).map(s => `${s.citation}${s.source ? ` [${s.source}]` : ""} - "${s.quote}"`).join(" | ")}
Ověření: ${v.verdict.note}

${v.p.analysis}`).join("\n\n")

  const opinion = await agent(
    `Napiš právní stanovisko. Všechno, o co se smíš opřít, je níže - nedoplňuj nic z paměti a nedohledávej nové zdroje.

${context}

${casePacket}

## Ověřené propozice
${held || "(žádná propozice neobstála)"}
${conflicts.length > 0 ? `\n## Propozice, které si odporují\n${conflicts.map(c => `- ${c.claims.join(" VS. ")}: ${c.note}`).join("\n")}\n` : ""}${refuted.length > 0 ? `\n## VYVRÁCENO - nesmíš o to opřít závěr ani to citovat jako platné\n${refuted.map(v => `- ${v.p.claim} (${v.verdict.note})`).join("\n")}\n` : ""}${unverified.length > 0 ? `\n## NEOVĚŘENO - nesmíš o to opřít závěr; je-li to pro věc podstatné, napiš, že stanovisko je v tom rozsahu předběžné\n${unverified.map(v => `- ${v.p.claim} (${v.verdict.note})`).join("\n")}\n` : ""}${brokenRun.length > 0 ? `\n## Neověřeno kvůli chybě běhu, ne kvůli právu - neprezentuj to jako právní nejistotu; napiš, že tato část nebyla zkontrolována\n${bullets(brokenRun.map(v => v.p.claim))}\n` : ""}${skipped.length > 0 ? `\n## Podpůrné, neprošlo ověřením - použij nanejvýš jako dokreslení a označ to\n${bullets(skipped.map(p => p.claim))}\n` : ""}${gaps.length > 0 ? `\n## Skutečnosti, které při rešerši chyběly\n${bullets(gaps)}\n` : ""}
Do 'opinion' napiš celé stanovisko v tomto pořadí: zadání a praktický cíl; předpoklady, ze kterých vycházíš; krátká odpověď; použitelné právo a rozhodné znění; rozbor po jednotlivých otázkách; argumenty ve prospěch klienta; nejsilnější argumenty protistrany; důkazní situace, tedy co je čím doloženo a co by bylo třeba prokázat; procesní, časová a vymahatelnostní rizika; varianty podle skutečností, které zůstaly neznámé; doporučený další postup; a co je třeba ještě získat nebo ověřit.

Chybějící podklady nejsou důvod rozbor vynechat. "Nejdřív mi dodejte dokumenty" není stanovisko. I bez jediného dokumentu napiš z propozic výše, které právní režimy přicházejí v úvahu, jaké jsou podmínky jednotlivých nároků, které skutečnosti mezi režimy rozhodnou, koho tíží důkazní břemeno a jak dopadne každá varianta. Předpoklady vypiš na začátku a u každého uveď, co by se změnilo, kdyby neplatil. Seznam podkladů k zajištění patří na konec, ne místo rozboru. Podmíněný závěr piš podmíněně, ale piš ho.

Než to odevzdáš: odpovídáš na praktickou otázku klienta, nezaměnil jsi právní možnost za pravděpodobný výsledek, je každé klíčové tvrzení doložené, uvedl jsi nejsilnější protiargument a je jasné, které závěry stojí na neprokázané skutečnosti?

U každého zdroje, o který se opřeš, uveď i to, odkud je - je to v materiálu výše v hranatých závorkách za citací. Když je to odkaz, udělej z citace odkaz ve tvaru [citace](odkaz); jinak napiš citaci a za ni ten název nebo cestu. Nikdy si nic z toho nedomýšlej. Stanovisko, ze kterého nejde dohledat zdroje, je pro klienta neověřitelné.

Kde materiál otázku neuzavírá, řekni to. Skutečnost, kterou nikdo neprokázal, se nesmí stát mlčky předpokladem.

${RULES}`,
    { label: "stanovisko", agentType: "codexis:synthesizer", schema: OPINION, model: MODEL })

  if (!opinion) {
    return { error: "Nepodařilo se sepsat stanovisko.", casePacket,
      propositions: merged.map(p => p.claim) }
  }

  const answered = new Set(holding.map(v => v.p.issue))
  const uncovered = plan.issues.map(i => i.label).filter(l => !answered.has(l))
  const grounded = holding.filter(v => (v.p.sources || []).some(s => s.citation))
  const shortfalls = []
  if (grounded.length === 0) {
    shortfalls.push("žádná propozice neobstála s dohledatelným zdrojem")
  }
  if (uncovered.length > 0) {
    shortfalls.push(`bez ověřené odpovědi zůstaly otázky: ${uncovered.join(", ")}`)
  }
  if (brokenRun.length > 0) {
    shortfalls.push(`${brokenRun.length} ověření spadlo na chybu běhu, ta část zkontrolovaná není`)
  }
  if (shortfalls.length > 0) {
    log(`Částečně dokončeno: ${shortfalls.join("; ")}`)
  }

  return {
    question: QUESTION,
    complete: shortfalls.length === 0,
    ...(shortfalls.length > 0 ? { incomplete: shortfalls } : {}),
    uncoveredIssues: uncovered,
    ...opinion,
    counterparties: parties.map(p => ({ subject: p.subject, report: p.report })),
    propositions: holding.map(v => ({ claim: v.p.claim, impact: v.p.impact,
      narrowed: v.p.narrowed === true, analysis: v.p.analysis, checked: v.verdict.note,
      sources: (v.p.sources || []).map(s => ({ citation: s.citation, source: s.source, quote: s.quote })) })),
    conflicts,
    refutedPropositions: refuted.map(v => ({ claim: v.p.claim, why: v.verdict.note })),
    unverifiedPropositions: unverified.map(v => ({ claim: v.p.claim, why: v.verdict.note })),
    openFacts: gaps,
    clientStatedFacts: stated,
    stats: { issues: plan.issues.length, propositions: all.length, merged: merged.length,
      verified: checked.length, holding: holding.length, refuted: refuted.length,
      unverified: unverified.length, verificationErrors: brokenRun.length,
      counterparties: parties.length }
  }
}

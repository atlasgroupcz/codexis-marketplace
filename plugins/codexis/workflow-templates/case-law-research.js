// @template case-law-research
// @use-when A question that has to be answered from what the courts actually decide: what the settled
//   practice is, which decision carries it, whether it still holds under today's wording, and where
//   the courts disagree. Not for reading the client's file and not for writing the client opinion.
// @jurisdictions CZ SK EU
// @args question (required) - the legal question, in the user's own words
// @args jurisdiction - CZ, SK or EU; defaults to CZ
// @args asOf - the date the answer must be true for; defaults to today

export const meta = {
  name: "case-law-research",
  description: "Rešerše judikatury: rozbor otázky → vyhledání rozhodnutí → ověření a zpráva.",
  phases: [
    { title: "Rozbor otázky", detail: "Právní rámec, rozhodné znění a dílčí otázky" },
    { title: "Vyhledání", detail: "Rozhodnutí ke každé dílčí otázce, se skutkem a doslovnou citací" },
    { title: "Ověření a zpráva", detail: "Kontrola citovaných rozhodnutí proti odůvodnění a sepsání rešerše" }
  ]
}

const MODEL = "GPT_5_6_LUNA"


const [PHASE_SCOPE, PHASE_SEARCH, PHASE_REPORT] = meta.phases.map(p => p.title)

const LABEL_SCOPE = "rámec a dílčí otázky"
const LABEL_VERIFY = issue => `ověření: ${issue}`

const REPORT_PREFIX = "resherse-judikatura"

const RELAY = path => `---

Pokyn pro asistenta: tohle je hotová rešerše judikatury. Vypiš ji uživateli do chatu CELOU a doslova, včetně všech nadpisů, tabulek, citací rozhodnutí a doslovných výňatků.

Nešrnuj ji, nezkracuj, nevybírej z ní a nepřeformulovávej ji vlastními slovy. Nevynechávej oddíly proto, že se ti zdají dlouhé - právě ty citace a výňatky jsou to, kvůli čemu si o rešerši uživatel řekl. Nepřidávej k ní vlastní právní závěry.

Tenhle pokyn ani tuhle patičku do chatu nevypisuj. Celá rešerše je uložena i jako ${path}; odpověď tím souborem nenahrazuj.`

const RULES = `Odpovídej česky. Piš jen to, co jsi otevřel a přečetl. Zdroj si nikdy nevymýšlej - když ho nemáš, řekni to.

Právní větu ber jen jako vodítko, kde hledat, nikdy jako význam rozhodnutí. Právní věty se píšou zkratkovitě, často zobecňují víc, než soud řekl, a mlčí o skutku, na kterém závěr stojí. Každé rozhodnutí, o které se chceš opřít, proto otevři celé a přečti odůvodnění. Když jsi ho neotevřel, nesmíš se o něj opřít.

U rozhodnutí vždycky napiš, čeho se ta věc týkala. Právní věta bez skutku je past - podle skutku se pozná, jestli rozhodnutí na naši otázku vůbec sedí.

Každý dokument, o který se opřeš, označ citačním markerem podle oddílu "## Citations" ve svých instrukcích - u tvrzení, které z něj plyne, i za doslovným výňatkem. Napsat slovy, odkud to je, nestačí: marker je jediné, co se přenese dál. Co markerem neoznačíš, se k zadavateli nedostane a v rešerši ten dokument nebude.`

const DEPTH = `Projdi co nejvíce relevantních dokumentů a buď maximálně detailní. Nespokoj se s několika rozhodnutími - chci vyčerpávající přehled. Raději delší výstup než vynechané rozhodnutí.

Postupuj odshora dolů: sjednocující stanoviska a rozhodnutí velkého senátu nebo pléna dřív než jednotlivá, vyšší soudy dřív než nižší. U každého ustanovení, o které se opíráš, uveď jeho číslo a znění, které platí k rozhodnému datu.`

const OUTPUT_SHAPE = `Piš markdown. Struktura, kterou drž:

## Odpověď
Přímá odpověď na dílčí otázku, v několika větách. Když je odpověď podmíněná, napiš čím.

## Právní úprava
Ustanovení, o která se to opírá, s citací znění a s uvedením, od kdy to znění platí.

## Nosná rozhodnutí
Tabulka, řádek na rozhodnutí:

| soud | značka | datum | co z něj plyne | skutek |
| --- | --- | --- | --- | --- |
| Ústavní soud | Pl. ÚS 45/06 | 11. 6. 2007 | Obec smí vyhláškou regulovat jen místní záležitosti veřejného pořádku, ne opakovat zákonnou úpravu | Vyhláška města Jirkov o zabezpečení místních záležitostí veřejného pořádku |

## Rozbor
Ke každému nosnému rozhodnutí odstavec: co soud řešil, jak to odůvodnil a co z toho pro naši otázku plyne. Doslovný výňatek z odůvodnění dej do citace:

> Obecně závazná vyhláška nesmí upravovat věci, které jsou vyhrazeny zákonu, ani zákonnou úpravu opakovat či parafrázovat.

Za výňatkem uveď, odkud je, a připoj citační marker toho rozhodnutí.

## Kde se praxe rozchází
Rozhodnutí, která si odporují, a v čem. Když se praxe nerozchází, napiš to.

## Co zůstává nezodpovězeno
Co na tuhle otázku judikatura neřeší, nebo co se nepodařilo ověřit.`

const PROMPT_SCOPE = (question, jurisdiction, asOf) => `${RULES}

Otázka: ${question}
Jurisdikce: ${jurisdiction}
Rozhodné datum: ${asOf}

Urči právní rámec a rozděl otázku na dílčí otázky, na kterých odpověď stojí.

Otevři si ustanovení, kterých se věc týká, ať víš, o čem mluvíš, a zjisti, jaké znění platí k rozhodnému datu.

Každá dílčí otázka se musí dát zodpovědět samostatně a smí se ptát jen na jednu věc. Jedna věta, jeden otazník. Když v otázce potřebuješ "a jaké", "případně" nebo středník, slepuješ dvě otázky do jedné - udělej z nich dvě.

Kolik dílčích otázek věc potřebuje, tolik jich udělej. Žádný strop. Ale ani jednu do počtu.`

const PROMPT_SEARCH = (framing, label, question, asOf) => `${RULES}

${framing}

Dílčí otázka: ${question}

${DEPTH}

Na vyhledání kandidátů použij nástroj script: rozešli víc přeformulovaných dotazů najednou, nech je odduplikovat podle docId a vrátit jen dokumenty s doslovnými pasážemi. Celý text otevírej až u toho, co ti program vrátí - projít stovky zásahů takhle nestojí kontext.

Ověř u každého rozhodnutí, podle jakého znění předpisu soud rozhodoval a jestli to znění platí i k ${asOf}. Když najdeš pozdější rozhodnutí, které dřívější závěr mění, uveď ho a napiš, jak ho mění.

Když na tuhle otázku judikatura neodpovídá, napiš to a nevyplňuj to něčím příbuzným.

${OUTPUT_SHAPE}

Tvůj výstup se použije doslova jako oddíl "${label}" ve výsledné rešerši.`

const PROMPT_VERIFY = (framing, question, draft) => `${RULES}

${framing}

Dílčí otázka: ${question}

Níže je hotový oddíl rešerše. Otevři každé rozhodnutí, které je v něm citováno, a přečti odůvodnění. U každého zjisti trojí: sedí doslovný výňatek na to, co je uvedeno? Nese ta pasáž tvrzení, které se o ni opírá, nebo je tvrzení širší než rozhodnutí? A řešil soud tentýž problém a srovnatelný skutek? Rozhodnutí, které není citované značkou a datem, ber jako neověřitelné.

Oddíl neopisuj ani nepřepisuj - ten zůstane, jak je. Vrať jen svůj nález, v markdownu:

## Neobstálo
Řádek na každé rozhodnutí, které se nemá použít: značka, co je s ním špatně a čeho v oddílu se to týká.

## K zúžení
Řádek na každé tvrzení, které je širší než rozhodnutí: jak zní teď a jak má znít, aby odpovídalo tomu, co soud řekl.

## Obstálo
Značky rozhodnutí, která jsi otevřel a potvrdil.

## Neověřeno
Co se ověřit nepodařilo a proč.

Oddíl k ověření:

${draft}`

const REPORT_HEADER = (question, jurisdiction, asOf) => `# Rešerše judikatury

Otázka: ${question}
Jurisdikce: ${jurisdiction}
Rozhodné datum: ${asOf}
`

const SCOPE = { type: "object", required: ["framing", "issues"], properties: {
  framing: { type: "string", description: "Hlavička, kterou dostane každý vyhledávač doslova, v markdownu, do patnácti řádků: právní obor, jurisdikce, rozhodné datum a znění předpisu, ustanovení, o která se věc opírá, a co v otázce zůstává nejisté. Piš to jako zadání pro kolegu." },
  issues: { type: "array", description: "Dílčí otázky, rozhodující první.", items: { type: "object", required: ["label", "question"], properties: {
    label: { type: "string", description: "Krátký název otázky. Slouží jako popisek běhu a jako nadpis oddílu ve výsledné rešerši." },
    question: { type: "string", description: "Celá otázka, jedna věta, jeden otazník, jeden právní problém." } } } } } }

const NOT_VERIFIED = "Ověření se nepodařilo provést, tenhle oddíl prošel bez kontroly."

const section = (label, question, draft, review) => `# ${label}

Dílčí otázka: ${question}

${draft}

## Výsledek ověření

${review || NOT_VERIFIED}`

export default async function (args) {
  const question = (args && args.question) || ""
  const jurisdiction = (args && args.jurisdiction) || "CZ"
  const asOf = (args && args.asOf) || new Date().toISOString().slice(0, 10)

  if (!question.trim()) {
    throw new Error("case-law-research potřebuje otázku v args.question")
  }

  phase(PHASE_SCOPE)

  const scope = await agent(PROMPT_SCOPE(question, jurisdiction, asOf),
    { label: LABEL_SCOPE, agentType: "codexis:synthesizer", schema: SCOPE, model: MODEL })

  log(`dílčích otázek: ${scope.issues.length}, všechny souběžně`)

  phase(PHASE_SEARCH)

  const drafts = await parallel(scope.issues.map(issue => () =>
    agent(PROMPT_SEARCH(scope.framing, issue.label, issue.question, asOf),
      { label: issue.label, model: MODEL })))

  const researched = scope.issues
    .map((issue, i) => ({ issue, draft: drafts[i] }))
    .filter(x => x.draft)

  log(`zpracováno oddílů: ${researched.length} z ${scope.issues.length}`)

  phase(PHASE_REPORT)

  const reviews = await parallel(researched.map(({ issue, draft }) => () =>
    agent(PROMPT_VERIFY(scope.framing, issue.question, draft),
      { label: LABEL_VERIFY(issue.label), agentType: "codexis:verifier", model: MODEL })))

  const body = researched
    .map(({ issue, draft }, i) => section(issue.label, issue.question, draft, reviews[i]))
    .join("\n\n---\n\n")

  const unresearchedLabels = scope.issues.filter((_, i) => !drafts[i]).map(i => i.label)
  const unverifiedLabels = researched.filter((_, i) => !reviews[i]).map(x => x.issue.label)

  const gaps = [
    unresearchedLabels.length ? `Nezpracované dílčí otázky: ${unresearchedLabels.join(", ")}` : "",
    unverifiedLabels.length ? `Neověřené oddíly: ${unverifiedLabels.join(", ")}` : ""
  ].filter(Boolean)

  const missing = gaps.length
    ? "\n## Co v rešerši chybí\n\n" + gaps.join("\n") + "\n"
    : ""

  const report = `${REPORT_HEADER(question, jurisdiction, asOf)}
${body}
${missing}`

  const { writeFile } = await import("node:fs/promises")
  const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-")
  const path = `${REPORT_PREFIX}-${stamp}.md`

  await writeFile(path, report, "utf8")

  log(`zpráva zapsána: ${path}, ${report.length} znaků`)

  return `${report}

${RELAY(path)}`
}

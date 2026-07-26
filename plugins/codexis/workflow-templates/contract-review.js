// @template contract-review
// @use-when A contract the client wants reviewed: pull it apart clause by clause, judge each against
//   the law that actually governs it, and come back with the risky ones and concrete redrafts.
// @jurisdictions CZ SK EU
// @args path (required) — path to the contract file to review
// @args side — whose position it is judged from; defaults to objednatele
//
// Clause-level fan-out: one reviewer per clause, each seeing the clause text verbatim and nothing
// else, then a currency check on the provisions the risks lean on, then one drafting pass.

export const meta = {
  name: "contract-review",
  description: "Rozbor smlouvy: rozklad na ustanovení → posouzení rizik → ověření → doporučené úpravy.",
  phases: [
    { title: "Rozklad", detail: "Určit typ smlouvy, rozhodné právo a rozdělit ji na ustanovení" },
    { title: "Posouzení", detail: "Každé ustanovení proti kogentním normám a judikatuře" },
    { title: "Ověření", detail: "Platnost a účinnost norem, o které se rizika opírají" },
    { title: "Úpravy", detail: "Seřazená rizika a konkrétní návrhy přeformulování" }
  ]
}

const BREAKDOWN = { type: "object", required: ["contractType", "clauses"], properties: {
  contractType: { type: "string" }, governingLaw: { type: "string" }, party: { type: "string" },
  clauses: { type: "array", items: { type: "object", required: ["label", "text"], properties: {
    label: { type: "string" }, text: { type: "string" }, topic: { type: "string" } } } } } }

const RISK = { type: "object", required: ["risks"], properties: { risks: { type: "array", items: {
  type: "object", required: ["issue", "source", "quote", "severity"], properties: {
    issue: { type: "string" }, source: { type: "string" }, quote: { type: "string" },
    severity: { enum: ["vysoká", "střední", "nízká"] },
    consequence: { type: "string" } } } } } }

// Four states, never a boolean: "could not check" must never be able to pass for "checked".
const CURRENCY = { type: "object", required: ["status", "evidence"], properties: {
  status: { enum: ["supported", "refuted", "unverified", "source_unreachable"] },
  evidence: { type: "string" }, effectiveAt: { type: "string" } } }

const REDRAFT = { type: "object", required: ["summary", "items"], properties: {
  summary: { type: "string" },
  items: { type: "array", items: { type: "object", required: ["clause", "issue", "proposal", "sources"],
    properties: { clause: { type: "string" }, issue: { type: "string" }, proposal: { type: "string" },
      severity: { enum: ["vysoká", "střední", "nízká"] },
      sources: { type: "array", items: { type: "string" } } } } },
  caveats: { type: "string" } } }

export default async function (args) {
  const PATH = (typeof args === "object" && args && typeof args.path === "string" && args.path.trim()) || ""
  if (!PATH) {
    return { error: "Chybí cesta ke smlouvě. Předej ji jako args: { path: '<cesta k souboru>' }." }
  }
  const SIDE = (args && args.side) || "objednatele"

  phase("Rozklad")
  const breakdown = await agent(
    "Přečti smlouvu v souboru " + PATH + " a rozlož ji na jednotlivá ustanovení.\n\n" +
    "Posuzuje se z pozice " + SIDE + ".\n\n" +
    "Urči typ smlouvy a rozhodné právo. Každé ustanovení vrať s jeho označením (článek, odstavec) a " +
    "s jeho DOSLOVNÝM zněním — každé z nich dostane vlastního posuzovatele, který smlouvu neuvidí, " +
    "takže cokoli vynecháš, se ztratí. Drobná a čistě technická ustanovení slučuj. " +
    "Structured output only.",
    { label: "rozklad smlouvy", schema: BREAKDOWN })

  const BRIEF =
    "Posuzovaná smlouva: " + breakdown.contractType +
    "\nRozhodné právo: " + (breakdown.governingLaw || "české") +
    "\nPosuzuje se z pozice: " + (breakdown.party || SIDE) +
    "\n(Odpovídej česky. Každé riziko musí nést přesnou citaci ustanovení nebo rozhodnutí " +
    "a doslovný výňatek. Nikdy si zdroj nevymýšlej.)"

  log("Ustanovení k posouzení: " + breakdown.clauses.length)

  phase("Posouzení")
  const perClause = await parallel(breakdown.clauses.map(clause => () =>
    agent(
      BRIEF + "\n\nPosuď TOTO ustanovení a nic jiného.\n\n" +
      "Označení: " + clause.label + (clause.topic ? "\nTéma: " + clause.topic : "") +
      "\nZnění: \"" + clause.text + "\"\n\n" +
      "Ověř ho proti kogentním normám (CR, u unijního přesahu EU), proti výkladu v komentářích " +
      "(COMMENT) a proti tomu, jak takové doložky posuzují soudy (JD, ES). Hledej neplatnost, " +
      "nepřiměřenost, nevyváženost v neprospěch posuzované strany, rozpor se spotřebitelskou nebo " +
      "jinou ochrannou úpravou a mezery, které se v praxi vymstí. Pokud je ustanovení v pořádku, " +
      "vrať prázdný seznam rizik — nevymýšlej si problémy. U každého rizika uveď následek. " +
      "Structured output only.",
      { label: clause.label, schema: RISK })
      .catch(() => ({ risks: [] }))
      .then(result => ({ clause, risks: result.risks }))))

  const flagged = perClause.filter(Boolean).flatMap(r => r.risks.map(risk => ({ clause: r.clause, risk })))
  const bySeverity = { vysoká: 0, střední: 1, nízká: 2 }
  const ranked = [...flagged].sort((a, b) => bySeverity[a.risk.severity] - bySeverity[b.risk.severity])
  log("Nalezená rizika: " + ranked.length)
  if (ranked.length === 0) {
    return { path: PATH, contractType: breakdown.contractType,
      summary: "Posouzení neodhalilo riziková ustanovení.", items: [],
      caveats: "Posuzováno bylo " + breakdown.clauses.length + " ustanovení." }
  }

  phase("Ověření")
  const verified = await parallel(ranked.slice(0, 12).map(entry => () =>
    agent(
      BRIEF + "\n\nOvěř, jestli norma, o kterou se toto riziko opírá, platí i dnes.\n\n" +
      "Ustanovení smlouvy: " + entry.clause.label +
      "\nTvrzené riziko: " + entry.risk.issue +
      "\nZdroj: " + entry.risk.source +
      "\nCitace: " + entry.risk.quote + "\n\n" +
      "U předpisu ověř účinné znění: nebyl zrušen, novelizován ani nahrazen? U judikatury ověř, že " +
      "rozhodnutí nebylo překonáno. Otevři zdroj celý. Do 'evidence' napiš rozhodující formulaci — " +
      "půjde do návrhu úprav.\n\n" +
      "Status voli takto: 'supported' jen když jsi zdroj otevřel a norma i riziko obstály; 'refuted' " +
      "když jsi našel konkrétní důvod, proč riziko neplatí; 'source_unreachable' když se ti zdroj " +
      "nepodařilo otevřít; 'unverified' když jsi ho otevřel, ale nedokázal jsi to rozhodnout. " +
      "Nedostatek důkazu NENÍ vyvrácení a chybějící přístup NENÍ potvrzení. Structured output only.",
      { label: "ověření: " + entry.clause.label, agentType: "codexis:verifier", schema: CURRENCY })
      .then(verdict => ({ ...entry, verdict }))
      // A verifier that crashed has verified nothing. Failing open here would put an unchecked
      // risk in front of the client as if it had been confirmed.
      .catch(() => ({ ...entry, verdict: { status: "unverified",
        evidence: "ověření se nepodařilo provést (chyba běhu)" } }))))

  // parallel() keeps failed positions as null, so drop those before reading a verdict off them.
  const checked = verified.filter(Boolean)
  const holding = checked.filter(v => v.verdict.status === "supported")
  const unconfirmed = checked.filter(v => v.verdict.status !== "supported")
  log("Obstálo " + holding.length + " z " + checked.length + " rizik")

  phase("Úpravy")
  const block = holding.map((v, i) =>
    "[" + i + "] ustanovení " + v.clause.label + " — " + v.risk.issue +
    "\n    závažnost: " + v.risk.severity +
    "\n    znění ustanovení: \"" + v.clause.text + "\"" +
    "\n    zdroj: " + v.risk.source +
    "\n    citace: \"" + v.risk.quote + "\"" +
    (v.risk.consequence ? "\n    následek: " + v.risk.consequence : "") +
    (v.verdict.evidence ? "\n    ověření: " + v.verdict.evidence : "")).join("\n\n")

  const redraft = await agent(
    BRIEF + "\n\nSepiš výstup pro klienta z ověřených rizik níže.\n\n" + block + "\n\n" +
    (unconfirmed.length > 0
      ? "Tato rizika se NEPODAŘILO OVĚŘIT a nesmíš je klientovi předložit jako potvrzená; pokud jsou " +
        "podstatná, uveď je v caveats jako neověřená: " +
        unconfirmed.map(v => v.clause.label + " — " + v.risk.issue).join("; ") + "\n\n"
      : "") +
    "Vše, o co se smíš opřít, je zde — nedoplňuj nic z vlastní paměti a nedohledávej nové zdroje. " +
    "U každého rizika navrhni KONKRÉTNÍ přeformulování ustanovení, ne obecné doporučení: napiš text, " +
    "který má nahradit ten stávající. Seřaď od nejzávažnějšího. Shrnutí piš pro klienta, ne pro " +
    "právníka. Structured output only.",
    { label: "návrh úprav", agentType: "codexis:synthesizer", schema: REDRAFT })

  return { path: PATH, contractType: breakdown.contractType, ...redraft,
    stats: { clauses: breakdown.clauses.length, risks: flagged.length, checked: checked.length,
      holding: holding.length, unconfirmed: unconfirmed.length } }
}

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

const MAX_ISSUES = 8
const MAX_VERIFY = 8


const [PHASE_SCOPE, PHASE_SEARCH, PHASE_REPORT] = meta.phases.map(p => p.title)

const LABEL_SCOPE = "rámec a dílčí otázky"
const LABEL_VERIFY = issue => `ověření: ${issue}`

const REPORT_PREFIX = "resherse-judikatura"

const RELAY = (path, docx) => `---

Pokyn pro asistenta: tohle je hotová rešerše judikatury. Vypiš ji uživateli do chatu CELOU a doslova, včetně všech nadpisů, tabulek, citací rozhodnutí a doslovných výňatků.

Nešrnuj ji, nezkracuj, nevybírej z ní a nepřeformulovávej ji vlastními slovy. Nevynechávej oddíly proto, že se ti zdají dlouhé - právě ty citace a výňatky jsou to, kvůli čemu si o rešerši uživatel řekl. Nepřidávej k ní vlastní právní závěry.

Tenhle pokyn ani tuhle patičku do chatu nevypisuj. Celá rešerše je uložena i jako ${path}; odpověď tím souborem nenahrazuj.${docx ? `

Rešerše je vysázená i ve Wordu jako ${docx} - hotový dokument s titulní stranou, obsahem a hlavičkou CODEXIS. Nabídni ho odkazem na konci odpovědi a znovu ho nevyráběj.` : ""}`

const RULES = `Odpovídej česky. Piš jen to, co jsi otevřel a přečetl. Zdroj si nikdy nevymýšlej - když ho nemáš, řekni to.

Právní větu ber jen jako vodítko, kde hledat, nikdy jako význam rozhodnutí. Právní věty se píšou zkratkovitě, často zobecňují víc, než soud řekl, a mlčí o skutku, na kterém závěr stojí. Každé rozhodnutí, o které se chceš opřít, proto otevři celé a přečti odůvodnění. Když jsi ho neotevřel, nesmíš se o něj opřít.

U rozhodnutí vždycky napiš, čeho se ta věc týkala. Právní věta bez skutku je past - podle skutku se pozná, jestli rozhodnutí na naši otázku vůbec sedí.

Každý dokument, o který se opřeš, označ citačním markerem podle oddílu "## Citations" ve svých instrukcích - u tvrzení, které z něj plyne, i za doslovným výňatkem. Napsat slovy, odkud to je, nestačí: marker je jediné, co se přenese dál. Co markerem neoznačíš, se k zadavateli nedostane a v rešerši ten dokument nebude.`

const STANDARD = path => path
  ? `

Podobu, kterou má rešerše mít, popisuje ${path}. Přečti si ji nástrojem readFile dřív, než začneš psát, a drž se jí.`
  : ""

const DEPTH = `Hledej vyčerpávajícně, piš úsporně. To jsou dvě různé věci a nesmíš je zaměnit.

Projdi co nejvíc relevantních dokumentů a nespokoj se s několika rozhodnutími - vynechané rozhodnutí je chyba. Ale do výstupu dej jen to, co nese závěr: nosná rozhodnutí, nosné věty, skutek. Co jsi přečetl a neopíráš se o to, do rešerše nepatří. Delší text není lepší rešerše; čtenář je právník, který má málo času, a rozvláčnost čte jako nedodělanou práci.

Postupuj odshora dolů: sjednocující stanoviska a rozhodnutí velkého senátu nebo pléna dřív než jednotlivá, vyšší soudy dřív než nižší. U každého ustanovení, o které se opíráš, uveď jeho číslo a znění, které platí k rozhodnému datu.`

const OUTPUT_SHAPE = `Piš markdown. Struktura, kterou drž:

## Odpověď
Dvě až pět vět, nic víc. Začni slovem Ano, Ne, Spíše ano, Spíše ne nebo Nelze určit, pak pravidlo a pak skutečnost, na které to stojí. Když je odpověď podmíněná, napiš čím. Neopakuj otázku, nezačínej "Otázka se ptá, zda" a nehedguj víc, než kolik je skutečné právní nejistoty. Tenhle oddíl čte i ten, kdo dál nedočte.

## Právní úprava
Jen ta věta ustanovení, o kterou skutečně jde, ne celý paragraf. K ní číslo ustanovení a znění platné k rozhodnému datu. Celá znění nikam neopisuj - kdo je potřebuje, otevře si předpis.

## Nosná rozhodnutí
Tabulka, řádek na rozhodnutí. Poslední sloupec je jednoslovný: přiléhá, odlišné nebo proti.

| soud | značka | datum | co z něj plyne | skutek | vztah |
| --- | --- | --- | --- | --- | --- |
| Ústavní soud | Pl. ÚS 45/06 | 11. 6. 2007 | Obec smí vyhláškou regulovat jen místní záležitosti veřejného pořádku, ne opakovat zákonnou úpravu | Vyhláška města Jirkov o zabezpečení místních záležitostí veřejného pořádku | přiléhá |

## Rozbor
Ke každému nosnému rozhodnutí odstavec o třech až šesti větách: co soud řešil, jak to odůvodnil a co z toho pro naši otázku plyne.

Doslovně cituj jen nosnou větu - tu, na které závěr stojí. Nikdy ne celý odstavec odůvodnění; zbytek převyprávěj vlastními slovy. Před každým výňatkem musí být věta, která říká, proč ho čtenář má číst. Výňatek delší než dvě věty zkrať:

> Obecně závazná vyhláška nesmí upravovat věci, které jsou vyhrazeny zákonu, ani zákonnou úpravu opakovat či parafrázovat.

Za výňatkem uveď, odkud je, a připoj citační marker toho rozhodnutí.

## Kde se praxe rozchází
Rozhodnutí, která si odporují, a v čem. Nezamlčuj rozhodnutí, které jde proti závěru - kdo se o rešerši opře a narazí na ně až u soudu, je na tom hůř než ten, komu jsi rovnou napsal, že praxe není jednotná. Když se praxe nerozchází, napiš to jednou větou.

## Co zůstává nezodpovězeno
Co na tuhle otázku judikatura neřeší, nebo co se nepodařilo ověřit. Piš to natvrdo: "K X nebylo nalezeno žádné rozhodnutí." Nevyplňuj mezeru něčím příbuzným.`

const PROMPT_SCOPE = (question, jurisdiction, asOf) => `${RULES}

Otázka: ${question}
Jurisdikce: ${jurisdiction}
Rozhodné datum: ${asOf}

Urči právní rámec a rozděl otázku na dílčí otázky, na kterých odpověď stojí.

Otevři si ustanovení, kterých se věc týká, ať víš, o čem mluvíš, a zjisti, jaké znění platí k rozhodnému datu.

Každá dílčí otázka se musí dát zodpovědět samostatně a smí se ptát jen na jednu věc. Jedna věta, jeden otazník. Když v otázce potřebuješ "a jaké", "případně" nebo středník, slepuješ dvě otázky do jedné - udělej z nich dvě.

Dílčích otázek udělej nejvýše ${MAX_ISSUES}, seřazených od nejdůležitější. Vyber ty, na kterých odpověď doopravdy stojí - raději ${MAX_ISSUES} pořádně zodpovězených než dvakrát tolik odbytých. Když jich věc snese míň, udělej míň; ani jednu do počtu.`

const PROMPT_SEARCH = (framing, label, question, asOf, standard) => `${RULES}${STANDARD(standard)}

${framing}

Dílčí otázka: ${question}

${DEPTH}

Na vyhledání kandidátů použij nástroj script: rozešli víc přeformulovaných dotazů najednou, nech je odduplikovat podle docId a vrátit jen dokumenty s doslovnými pasážemi. Celý text otevírej až u toho, co ti program vrátí - projít stovky zásahů takhle nestojí kontext.

Ověř u každého rozhodnutí, podle jakého znění předpisu soud rozhodoval a jestli to znění platí i k ${asOf}. Když najdeš pozdější rozhodnutí, které dřívější závěr mění, uveď ho a napiš, jak ho mění.

Když na tuhle otázku judikatura neodpovídá, napiš to a nevyplňuj to něčím příbuzným.

${OUTPUT_SHAPE}

Tvůj výstup se použije doslova jako oddíl "${label}" ve výsledné rešerši.`

const PROMPT_VERIFY = (framing, question, draft, standard) => `${RULES}${STANDARD(standard)}

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
  issues: { type: "array", maxItems: MAX_ISSUES, description: `Dílčí otázky, rozhodující první, nejvýše ${MAX_ISSUES}.`, items: { type: "object", required: ["label", "question"], properties: {
    label: { type: "string", description: "Krátký název otázky. Slouží jako popisek běhu a jako nadpis oddílu ve výsledné rešerši." },
    question: { type: "string", description: "Celá otázka, jedna věta, jeden otazník, jeden právní problém." } } } } } }

const NOT_VERIFIED = "Ověření se nepodařilo provést, tenhle oddíl prošel bez kontroly."

const DOCX_PREFIX = "reserse-judikatury"
const BRAND = "CODEXIS"
const BRAND_GREEN = "7CB302"
const BRAND_DARK = "3F5A0B"
const RULE_GREY = "C8CEC9"
const INK = "222222"
const FONT = "Times New Roman"

const plain = text => String(text == null ? "" : text)
  .replace(/[\s\S]*?/g, "")
  .replace(/【src_[0-9a-f]+】/g, "")

const LINK = /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)|(https?:\/\/[^\s)]+)/g

const emphasised = (d, text, options) => text.split("**").map((piece, i) =>
  new d.TextRun({ font: FONT, size: 24, color: INK, ...(options || {}), text: piece, bold: (options && options.bold) || i % 2 === 1 }))

const runsOf = (d, text, options) => {
  const source = plain(text)
  const out = []
  let last = 0
  for (const match of source.matchAll(LINK)) {
    if (match.index > last) { out.push(...emphasised(d, source.slice(last, match.index), options)) }
    out.push(new d.ExternalHyperlink({
      link: match[2] || match[3],
      children: [new d.TextRun({ font: FONT,
        ...(options || {}),
        text: match[1] || match[3],
        color: BRAND_GREEN,
        underline: {}
      })]
    }))
    last = match.index + match[0].length
  }
  if (last < source.length) { out.push(...emphasised(d, source.slice(last), options)) }
  return out.length ? out : emphasised(d, "", options)
}

const TEXT_WIDTH = 9638

const tableOf = (d, rows) => {
  const cells = rows.map(row => row.replace(/^\s*\|/, "").replace(/\|\s*$/, "").split("|").map(c => c.trim()))
  const width = Math.max(...cells.map(r => r.length))
  const weights = Array.from({ length: width }, (_, i) =>
    Math.min(40, Math.max(7, ...cells.map(row => (row[i] || "").length))))
  const total = weights.reduce((sum, weight) => sum + weight, 0)
  const columnWidths = weights.map(weight => Math.round(TEXT_WIDTH * weight / total))
  const hairline = { style: d.BorderStyle.SINGLE, size: 3, color: RULE_GREY }
  return new d.Table({
    width: { size: TEXT_WIDTH, type: d.WidthType.DXA },
    columnWidths,
    borders: { top: hairline, bottom: hairline, left: hairline, right: hairline,
      insideHorizontal: hairline, insideVertical: hairline },
    rows: cells.map((row, index) => new d.TableRow({
      tableHeader: index === 0,
      children: Array.from({ length: width }, (_, i) => new d.TableCell({
        shading: index === 0 ? { fill: BRAND_DARK } : undefined,
        margins: { top: 60, bottom: 60, left: 100, right: 100 },
        children: [new d.Paragraph({
          spacing: { after: 0 },
          children: runsOf(d, row[i] || "", index === 0
            ? { bold: true, color: "FFFFFF" }
            : { size: 22 })
        })]
      }))
    }))
  })
}

const bodyOf = (d, markdown) => {
  const lines = plain(markdown).split("\n")
  const out = []
  let i = 0
  while (i < lines.length) {
    const line = lines[i]
    if (!line.trim() || /^\s*---+\s*$/.test(line)) { i++; continue }
    const heading = /^(#{1,6})\s+(.*)$/.exec(line)
    if (heading) {
      out.push(new d.Paragraph({
        heading: heading[1].length <= 2 ? d.HeadingLevel.HEADING_2 : d.HeadingLevel.HEADING_3,
        children: runsOf(d, heading[2])
      }))
      i++
      continue
    }
    if (/^\s*\|/.test(line)) {
      const rows = []
      while (i < lines.length && /^\s*\|/.test(lines[i])) {
        if (!/^\s*\|[\s|:-]*\|?\s*$/.test(lines[i])) { rows.push(lines[i]) }
        i++
      }
      if (rows.length) {
        out.push(tableOf(d, rows))
        out.push(new d.Paragraph({
          spacing: { before: 60, after: 200 },
          children: [new d.TextRun({ font: FONT,
            text: "Zdroj: CODEXIS, rozhodnutí uvedená v tabulce.", italics: true, size: 16, color: "6B6B6B"
          })]
        }))
      }
      continue
    }
    if (/^\s*>/.test(line)) {
      const quote = []
      while (i < lines.length && /^\s*>/.test(lines[i])) {
        quote.push(lines[i].replace(/^\s*>\s?/, ""))
        i++
      }
      const text = quote.join(" ").trim()
      out.push(text.split(/\s+/).length < 50
        ? new d.Paragraph({ children: runsOf(d, `„${text}“`, { italics: true }) })
        : new d.Paragraph({
          indent: { left: 567, right: 340 },
          spacing: { before: 120, after: 120 },
          border: { left: { style: d.BorderStyle.SINGLE, size: 12, space: 12, color: BRAND_GREEN } },
          children: runsOf(d, text, { italics: true })
        }))
      continue
    }
    const paragraph = []
    while (i < lines.length && lines[i].trim() && !/^\s*[|>#]/.test(lines[i])) {
      paragraph.push(lines[i].replace(/^\s*[-*]\s+/, "• "))
      i++
    }
    out.push(new d.Paragraph({ alignment: d.AlignmentType.JUSTIFIED, children: runsOf(d, paragraph.join(" ")) }))
  }
  return out
}

const SECTION_ANSWER = "Odpověď"
const SECTION_LAW = "Právní úprava"
const SECTION_CONFLICT = "Kde se praxe rozchází"
const SECTION_OPEN = "Co zůstává nezodpovězeno"

const splitSections = markdown => {
  const found = {}
  let current = null
  for (const line of plain(markdown).split("\n")) {
    const heading = /^#{1,6}\s+(.*)$/.exec(line)
    if (heading) {
      current = heading[1].trim()
      found[current] = found[current] || []
      continue
    }
    if (current) { found[current].push(line) }
  }
  const trimmed = {}
  for (const [key, lines] of Object.entries(found)) {
    trimmed[key] = lines.join("\n").trim()
  }
  return trimmed
}

const PROMPT_DOCX_QA = pages => `Prohlédni si stránky hotového dokumentu. Předpokládej, že chyby tam jsou, a najdi je.

Otevři si každou stránku nástrojem readFile, jednu po druhé:
${pages.map(p => `- ${p}`).join("\n")}

U každé hledej: text uříznutý na okraji nebo přetékající ze stránky, překrývající se prvky, rozbitou nebo přes okraj přetékající tabulku, chybějící či zmrzačenou diakritiku (ě š č ř ž ů), nejednotné velikosti nadpisů, nadpis osamocený na konci stránky, zbylý zástupný text, prázdnou stránku, logo v nesmyslné velikosti.

Vrať krátký seznam po stránkách: číslo stránky a co je na ní špatně. Stránku, která je v pořádku, odbyj jednou větou. Nic nepřikrášluj - tenhle dokument jde klientovi.`

const renderPages = async docx => {
  const { execFile } = await import("node:child_process")
  const { promisify } = await import("node:util")
  const { stdout } = await promisify(execFile)("cdx-render", ["-i", docx], { timeout: 300000 })
  return (stdout.match(/\S+\.png/g) || [])
}

const pluginDirectory = async () => {
  const { readdir, access } = await import("node:fs/promises")
  const { join } = await import("node:path")
  const root = join(process.env.HOME || "/home/codexis", ".cdx", "marketplaces")
  for (const entry of await readdir(root).catch(() => [])) {
    const candidate = join(root, entry, "plugins", "codexis")
    if (await access(join(candidate, "icon.svg")).then(() => true, () => false)) { return candidate }
  }
  return null
}

const standardPath = async () => {
  const { access } = await import("node:fs/promises")
  const { dirname, join } = await import("node:path")
  const { fileURLToPath } = await import("node:url")
  const here = dirname(fileURLToPath(import.meta.url))
  const candidate = join(here, "references", "reserse-judikatury-standard.md")
  return access(candidate).then(() => candidate, () => null)
}

const logoPng = async () => {
  const { mkdir, readFile } = await import("node:fs/promises")
  const { join } = await import("node:path")
  const { execFile } = await import("node:child_process")
  const { promisify } = await import("node:util")
  const plugin = await pluginDirectory()
  const svg = plugin ? join(plugin, "icon.svg") : null
  if (!svg) { return null }
  await mkdir(".tmp", { recursive: true })
  await promisify(execFile)("soffice",
    ["--headless", "--convert-to", "png", "--outdir", ".tmp", svg], { timeout: 120000 })
  return readFile(join(".tmp", "icon.png"))
}

const titlePage = (d, logo, question, jurisdiction, asOf) => {
  const line = (label, value) => new d.Paragraph({
    spacing: { before: 60 },
    alignment: d.AlignmentType.LEFT,
    children: [new d.TextRun({ font: FONT, text: `${label}\t`, bold: true }), new d.TextRun({ font: FONT, text: plain(value) })]
  })
  return [
    ...(logo ? [new d.Paragraph({
      alignment: d.AlignmentType.CENTER,
      spacing: { before: 1200, after: 240 },
      children: [new d.ImageRun({ type: "png", data: logo, transformation: { width: 96, height: 96 } })]
    })] : []),
    new d.Paragraph({
      alignment: d.AlignmentType.CENTER,
      spacing: { after: 480 },
      children: [new d.TextRun({ font: FONT, text: BRAND, bold: true, size: 40, color: BRAND_DARK, characterSpacing: 40 })]
    }),
    new d.Paragraph({
      alignment: d.AlignmentType.CENTER,
      spacing: { after: 120 },
      border: { bottom: { style: d.BorderStyle.SINGLE, size: 12, space: 10, color: BRAND_GREEN } },
      children: [new d.TextRun({ font: FONT, text: "Rešerše judikatury", bold: true, size: 40, color: BRAND_DARK })]
    }),
    new d.Paragraph({
      alignment: d.AlignmentType.CENTER,
      spacing: { before: 240, after: 960 },
      children: [new d.TextRun({ font: FONT, text: plain(question), size: 24, italics: true, color: "555555" })]
    }),
    line("Jurisdikce:", jurisdiction),
    line("Rozhodné datum:", asOf),
    line("Zpracováno:", new Date().toLocaleDateString("cs-CZ"))
  ]
}

const buildDocx = async (target, question, jurisdiction, asOf, sections, gaps) => {
  const module = await import("docx")
  const d = module.Document ? module : module.default
  const logo = await logoPng().catch(() => null)
  const parsed = sections.map(entry => ({ ...entry, parts: splitSections(entry.draft) }))
  const heading1 = (text, newPage) => new d.Paragraph({
    heading: d.HeadingLevel.HEADING_1,
    pageBreakBefore: newPage === true,
    children: [new d.TextRun({ font: FONT, text })]
  })
  const children = [...titlePage(d, logo, question, jurisdiction, asOf)]

  children.push(heading1("Stručná odpověď", true))
  parsed.forEach(({ issue, parts }, index) => {
    children.push(new d.Paragraph({
      spacing: { before: 200, after: 60 },
      children: [new d.TextRun({ font: FONT, text: `${index + 1}. ${plain(issue.question)}`, bold: true })]
    }))
    children.push(...bodyOf(d, parts[SECTION_ANSWER] || "Na tuhle dílčí otázku se odpověď nepodařilo sestavit."))
  })
  parsed.forEach(({ issue, parts }, index) => {
    children.push(heading1(`${index + 1}. ${plain(issue.label)}`, index === 0))
    children.push(new d.Paragraph({
      spacing: { after: 160 },
      children: [new d.TextRun({ font: FONT, text: plain(issue.question), italics: true, color: "555555" })]
    }))
    for (const [title, text] of Object.entries(parts)) {
      if (title === SECTION_ANSWER || title === SECTION_LAW || !text) { continue }
      children.push(new d.Paragraph({
        heading: d.HeadingLevel.HEADING_2, children: [new d.TextRun({ font: FONT, text: title })]
      }))
      children.push(...bodyOf(d, text))
    }
  })

  if (gaps.length) {
    children.push(heading1("Co v rešerši chybí"))
    children.push(...bodyOf(d, gaps.join(" ")))
  }

  const statutes = parsed.filter(({ parts }) => parts[SECTION_LAW])
  if (statutes.length) {
    children.push(heading1("Příloha: dotčená ustanovení", true))
    statutes.forEach(({ issue, parts }) => {
      children.push(new d.Paragraph({
        spacing: { before: 200, after: 60 },
        children: [new d.TextRun({ font: FONT, text: plain(issue.label), bold: true })]
      }))
      children.push(...bodyOf(d, parts[SECTION_LAW]))
    })
  }
  const styles = level => ({
    run: { font: FONT, size: level === 1 ? 32 : 26, bold: true, color: BRAND_DARK },
    paragraph: {
      spacing: { before: level === 1 ? 360 : 240, after: 120, line: 276, lineRule: d.LineRuleType.AUTO },
      outlineLevel: level,
      border: level === 1
        ? { bottom: { style: d.BorderStyle.SINGLE, size: 8, space: 6, color: BRAND_GREEN } }
        : undefined
    }
  })
  const document = new d.Document({
    styles: {
      default: {
        document: {
          run: { font: FONT, size: 24, color: INK },
          paragraph: {
            alignment: d.AlignmentType.JUSTIFIED,
            spacing: { after: 140, line: 264, lineRule: d.LineRuleType.AUTO }
          }
        },
        heading1: styles(1),
        heading2: styles(2),
        heading3: { ...styles(3), run: { font: FONT, size: 24, bold: true, color: BRAND_DARK } }
      }
    },
    sections: [{
      properties: {
        page: {
          size: { width: 11906, height: 16838 },
          margin: { top: 1440, right: 1134, bottom: 1440, left: 1134 }
        }
      },
      headers: {
        default: new d.Header({
          children: [new d.Paragraph({
            alignment: d.AlignmentType.RIGHT,
            border: { bottom: { style: d.BorderStyle.SINGLE, size: 6, space: 4, color: BRAND_GREEN } },
            children: [new d.TextRun({ font: FONT, text: `${BRAND} · Rešerše judikatury`, size: 18, color: "666666" })]
          })]
        })
      },
      footers: {
        default: new d.Footer({
          children: [new d.Paragraph({
            alignment: d.AlignmentType.CENTER,
            children: [new d.TextRun({ font: FONT, children: ["str. ", d.PageNumber.CURRENT], size: 18, color: "666666" })]
          })]
        })
      },
      children
    }]
  })
  const { writeFile } = await import("node:fs/promises")
  await writeFile(target, await d.Packer.toBuffer(document))
  return target
}

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

  const standard = await standardPath().catch(() => null)
  if (standard) { log(`standard rešerše: ${standard}`) }

  phase(PHASE_SCOPE)

  const scope = await agent(PROMPT_SCOPE(question, jurisdiction, asOf),
    { label: LABEL_SCOPE, agentType: "codexis:synthesizer", schema: SCOPE, model: MODEL })

  const issues = scope.issues.slice(0, MAX_ISSUES)

  log(`dílčích otázek: ${issues.length}, všechny souběžně`)

  phase(PHASE_SEARCH)

  const drafts = await parallel(issues.map(issue => () =>
    agent(PROMPT_SEARCH(scope.framing, issue.label, issue.question, asOf, standard),
      { label: issue.label, model: MODEL })))

  const researched = issues
    .map((issue, i) => ({ issue, draft: drafts[i] }))
    .filter(x => x.draft)

  log(`zpracováno oddílů: ${researched.length} z ${issues.length}`)

  phase(PHASE_REPORT)

  const verified = researched.slice(0, MAX_VERIFY)

  log(`ověřuje se oddílů: ${verified.length}`)

  const reviews = await parallel(verified.map(({ issue, draft }) => () =>
    agent(PROMPT_VERIFY(scope.framing, issue.question, draft, standard),
      { label: LABEL_VERIFY(issue.label), agentType: "codexis:verifier", model: MODEL })))

  const body = researched
    .map(({ issue, draft }, i) => section(issue.label, issue.question, draft, reviews[i]))
    .join("\n\n---\n\n")

  const unresearchedLabels = issues.filter((_, i) => !drafts[i]).map(i => i.label)
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

  const sections = researched.map(({ issue, draft }, i) => ({ issue, draft, review: reviews[i] }))
  let docx = null
  try {
    docx = await buildDocx(`${DOCX_PREFIX}-${stamp}.docx`, question, jurisdiction, asOf, sections, gaps)
    log(`dokument zapsán: ${docx}`)
  } catch (failure) {
    log(`dokument se nepodařilo vysázet: ${failure.message}`)
  }

  return `${report}

${RELAY(path, docx)}`
}

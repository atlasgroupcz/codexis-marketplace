---
uuid: 30a77323-ca87-415f-b4de-d8c5f166a11d
name: docx
icon: icon.webp
description: "Word dokumenty (.docx) - smlouva, podání, memo, dopis, report, šablona. Vytvoř, uprav či přeformátuj Word a zkontroluj, jak vypadá. Ne pro Excel, PDF ani prezentace."
version: 1.0.0
allowed-tools: shell
i18n:
  cs:
    displayName: "Word dokumenty"
    summary: "Tvorba, úprava a vizuální kontrola .docx - smlouvy, podání, memoranda, dopisy."
    examplePrompts:
      - "Připrav ve Wordu vzor nájemní smlouvy k bytu."
      - "Přečti přiloženou smlouvu ve Wordu a shrň mi, co je v ní sjednáno."
      - "Sjednoť v tomto dokumentu písmo na Times New Roman 12 s řádkováním 1,5 a ukaž náhled stránek."
  en:
    displayName: "Word Documents"
    summary: "Create, edit and visually verify .docx files - contracts, filings, memos, letters."
    examplePrompts:
      - "Draft a residential lease agreement template in Word."
      - "Read the attached Word contract and summarize what it agrees to."
      - "Reformat this document to Times New Roman 12 with 1.5 line spacing and show me a preview of the pages."
  sk:
    displayName: "Word dokumenty"
    summary: "Tvorba, úprava a vizuálna kontrola .docx - zmluvy, podania, memorandá, listy."
    examplePrompts:
      - "Priprav vo Worde vzor nájomnej zmluvy k bytu."
      - "Prečítaj priloženú zmluvu vo Worde a zhrň mi, čo je v nej dohodnuté."
      - "Zjednoť v tomto dokumente písmo na Times New Roman 12 s riadkovaním 1,5 a ukáž náhľad strán."
---

# DOCX - creation, editing, visual verification

A `.docx` is a ZIP of XML files. You never have to read that XML by hand: generate with
`docx-js`, then **look at the rendered result** before you hand it over.

**IMPORTANT:** Stop on a genuine `Error:` line from `cdx-render` - report it and don't blindly
retry. LibreOffice and poppler are part of the guest image, so a missing `soffice` or `pdftoppm`
means something is broken with the image, not with your command. Report it; never work around it
with a hand-rolled conversion that skips the visual check.

## Reading a document - always do both

Text and images answer different questions. Text gives you exact wording; the render gives
you layout, stamps, signatures and tables that text extraction flattens or loses.

Reading the file gives you the wording as markdown. It does not give you the pages, so
render those as well:

```bash
cdx-render -i /home/codexis/smlouva.docx
```

Then `Read` the PNGs. Do not skip this for documents where the layout carries meaning -
a signature block, a stamp, a two-column annex or a table with merged cells is invisible
in the markdown but obvious in the image.

Tracked changes are flattened in the markdown. When the user asks about revisions, read the
raw markup instead:

```bash
pandoc --track-changes=all smlouva.docx -o /tmp/smlouva-revisions.md
```

## Creating a document

Use `docx-js` (installed in the guest image; `require('docx')` resolves with no setup).

```javascript
const fs = require('fs');
const { Document, Packer, Paragraph, TextRun, Table, TableRow, TableCell,
        Header, Footer, AlignmentType, HeadingLevel, TableOfContents,
        LineRuleType, PageNumber, WidthType, BorderStyle } = require('docx');

const FONT = 'Times New Roman';

const doc = new Document({
  styles: {
    default: {
      document: {
        run: { font: FONT, size: 24 },                                     // 24 half-points = 12pt
        paragraph: { spacing: { line: 360, lineRule: LineRuleType.AUTO } }, // 360 = 1.5 lines
      },
      heading1: {
        run: { font: FONT, size: 32, bold: true, color: '000000' },
        paragraph: { spacing: { before: 240, after: 240, line: 360, lineRule: LineRuleType.AUTO } },
      },
      heading2: {
        run: { font: FONT, size: 28, bold: true, color: '000000' },
        paragraph: { spacing: { before: 240, after: 120, line: 360, lineRule: LineRuleType.AUTO } },
      },
    },
  },
  sections: [{
    properties: {
      page: {
        size: { width: 11906, height: 16838 },          // A4 in DXA (1440 DXA = 1 inch)
        margin: { top: 1440, right: 1440, bottom: 1440, left: 1440 },
      },
    },
    children: [
      new Paragraph({ heading: HeadingLevel.HEADING_1, children: [new TextRun('Kupní smlouva')] }),
      new Paragraph({ children: [new TextRun('uzavřená podle § 2079 a násl. občanského zákoníku')] }),
    ],
  }],
});

Packer.toBuffer(doc).then((buf) => fs.writeFileSync('/home/codexis/smlouva.docx', buf));
```

**Only the `.docx` is the deliverable - write it under `./`.** Everything you produce merely to
build or check it is throwaway scratch that belongs under `./.tmp`, never next to the document:
your generator script (`./.tmp/create-smlouva.js`), the LibreOffice intermediate `.pdf`, and the
page PNGs. `cdx-render` already writes its output under `./.tmp` by default, so just `Read` the
paths it prints - do not pass `-o` pointing back into the working directory.

**Page size:** `docx-js` defaults to A4 (11906 × 16838 DXA), which is what Czech and Slovak
documents want - but set it explicitly anyway. US Letter is 12240 × 15840.

**Landscape:** pass the *portrait* dimensions plus `orientation: PageOrientation.LANDSCAPE`;
`docx-js` swaps them internally. Passing pre-swapped values produces a broken page.

**Table of contents:** headings must use `HeadingLevel`, and any custom heading style must
carry `outlineLevel`. A TOC over paragraphs that merely *look* like headings comes out empty.

```javascript
new TableOfContents('Obsah', { hyperlink: true, headingStyleRange: '1-3' })
```

Word only populates a TOC on open ("update fields"), so it will look empty in the render.
That is expected - do not try to "fix" it.

## Verification - required, in this order

Your first render is almost never right. Treat QA as a bug hunt, not a confirmation step.

**1. Render and look.**

```bash
cdx-render -i /home/codexis/smlouva.docx
```

`Read` every page image. You are looking for: text clipped at the margin, overlapping
elements, a table spilling off the page, a heading orphaned at the bottom, broken diacritics
(`ě š č ř ž`), empty pages, default-template styling.

**2. Use a subagent for anything longer than two pages.** You wrote the code, so you will see
what you intended, not what is there. Dispatch a subagent with the page images and this prompt:

```
Visually inspect these document pages. Assume there are problems - find them.
Look for: text clipped or running off the page; overlapping elements; broken tables;
missing or mangled Czech diacritics; inconsistent heading sizes; widows and orphans;
placeholder text left behind; signature blocks split across a page break.
For each page, list every issue, including minor ones.
```

**3. Grep for leftovers.**

```bash
pandoc /home/codexis/smlouva.docx -t plain |
  grep -iE 'lorem|ipsum|xxxx|TODO|\[doplnit\]|\{\{'
```

**4. Fix, then re-render the affected pages** (`cdx-render -i file.docx -f 3 -l 3`). One fix
routinely creates the next problem. Do not declare success until a full pass finds nothing new.

## Word compatibility - the render is LibreOffice, not Word

`cdx-render` draws the document with LibreOffice. The user opens it in Word. A file can look
perfect in the render and wrong on their machine, and the render will never tell you. Close the
gap in the generator instead:

**Always pass `lineRule` with `line`.** `{ line: 360 }` alone leaves the rule unset, and Word and
LibreOffice do not have to agree on the default.

```javascript
paragraph: { spacing: { line: 360, lineRule: LineRuleType.AUTO } }   // 360 = 1.5 lines
```

**Override headings through `styles.default.heading1`, never through `paragraphStyles`.** A
`paragraphStyles` entry with `id: 'Heading1'` is silently half-applied: `docx-js` keeps your
`size` but drops `font` and `bold`, and leaves Word's built-in heading blue (`2E74B5`) in place.
The heading then comes out **blue and in the wrong font in Word** - while LibreOffice re-styles it
and the render looks fine, so the QA loop never catches it.

```javascript
// WRONG - font and bold are dropped, the heading stays blue
paragraphStyles: [{ id: 'Heading1', run: { font: FONT, size: 32, bold: true } }]

// RIGHT
styles: {
  default: {
    heading1: { run: { font: FONT, size: 32, bold: true, color: '000000' } },
  },
}
```

Set `color` explicitly. Without it the blue survives.

Verify it landed - the style must carry `rFonts`, `<w:b/>` and your colour:

```bash
unzip -p smlouva.docx word/styles.xml | grep -o '<w:style [^>]*Heading1.*\?</w:style>'
```

**Use fonts the user actually has.** Times New Roman, Arial and Calibri are safe. LibreOffice
substitutes a metric-compatible face (Liberation Serif for Times New Roman) and the render looks
right even when the real font is missing - which is exactly how a font problem hides from you.

## Quality bar

- Never ship an AI-citation token (`【…†L1-L2】`) into the document body. Convert every source
  into a normal human-readable citation.
- Use ASCII hyphens. The non-breaking hyphen (U+2011) and exotic dashes do not render in some
  Word installs and come out as boxes.
- Consistent typography: one body font, one heading scale, one spacing rhythm. For legal
  documents Times New Roman 12pt with 1.5 line spacing is the safe house style.
- A document with the right words and a broken layout is a failed deliverable. The render is
  the deliverable, not the code that produced it.

## Converting to PDF

```bash
soffice -env:UserInstallation="file:///tmp/lo_profile_$$" --headless \
  --convert-to pdf --outdir /home/codexis /home/codexis/smlouva.docx
```

## Dependencies

- `docx-js` (Node, preinstalled) - creating documents
- `cdx-render` - document → page PNGs (LibreOffice + poppler)
- `pandoc` - tracked changes

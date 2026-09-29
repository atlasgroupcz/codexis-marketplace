---
uuid: 2aeefd95-cd11-431e-9b6c-b7dd54f3bbae
name: pdf
icon: icon.svg
description: "PDF - vygeneruj PDF, vyplň formulář, slouč/rozděl/orazítkuj, přečti naskenovanou smlouvu, rozsudek, fakturu (kde záleží na vzhledu). Ne pro čistý text - ten přečteš rovnou ze souboru."
version: 1.0.0
allowed-tools: shell
i18n:
  cs:
    displayName: "PDF dokumenty"
    summary: "Čtení PDF včetně vizuální kontroly, tvorba PDF, vyplňování formulářů, slučování a dělení."
    examplePrompts:
      - "Přečti tenhle naskenovaný rozsudek a zkontroluj i razítka a podpisy na stránkách."
      - "Vyplň v přiloženém formuláři jméno a datum a výsledek ulož jako PDF."
      - "Slouč přiložená PDF do jednoho dokumentu ve správném pořadí."
  en:
    displayName: "PDF Documents"
    summary: "Read PDFs visually, produce PDF deliverables, fill forms, merge, split and watermark."
    examplePrompts:
      - "Read this scanned judgment and check the stamps and signatures on the pages too."
      - "Fill in the name and date in the attached form and save the result as a PDF."
      - "Merge the attached PDFs into a single document in the right order."
  sk:
    displayName: "PDF dokumenty"
    summary: "Čítanie PDF vrátane vizuálnej kontroly, tvorba PDF, vypĺňanie formulárov, zlučovanie."
    examplePrompts:
      - "Prečítaj tento naskenovaný rozsudok a skontroluj aj pečiatky a podpisy na stranách."
      - "Vyplň v priloženom formulári meno a dátum a výsledok ulož ako PDF."
      - "Zlúč priložené PDF do jedného dokumentu v správnom poradí."
---

# PDF - reading, creation, forms

**IMPORTANT:** Stop on a genuine `Error:` line from `cdx-render` - report it and don't blindly
retry. LibreOffice and poppler are part of the guest image, so a missing `soffice` or `pdftoppm`
means something is broken with the image, not with your command. Report it; never work around it
with a hand-rolled conversion that skips the visual check.

## Reading a PDF - text and images, not one or the other

Read every PDF with `readFile`. It returns per-page Markdown next to each page image, so one
read gives you the exact strings and the visual layout together. Scanned pages go through the
platform OCR, the best OCR available to you, so a PDF with no text layer reads too.

**Never install or run your own OCR.** No `tesseract`, `ocrmypdf`, `pytesseract`, `easyocr`,
`paddleocr` or any other OCR engine, and no `cdxctl package install` to get one. It cannot beat
the Markdown `readFile` already gave you, and it costs the user time for a worse result.

One read covers at most 20 pages and carries the page images only when the window is 10 pages
or fewer - pass a page range (`1-5`, `3`) for anything longer, so you get the images.

The image shows what text cannot: a struck-through clause, a stamp over the signature, merged
table cells. **The image wins for layout, the text wins for exact strings.** When the Markdown
of a page is garbled or missing a value, read the value off that page's image. When the inline
image is too small to read, render the page at a higher resolution and read the PNG:

```bash
cdx-render -i rozsudek.pdf -f 3 -l 3 -r 300   # page 3 at 300 dpi
```

`cdx-render` runs at 150 dpi by default against the inline 100, and also covers a wider window
than one read carries images for.

## Creating a PDF

**Author it as a `.docx` and convert.** That gives you real typography, headers, footers,
page numbers and a table of contents for free, and it reuses the `docx` skill.

```bash
# after generating smlouva.docx with docx-js (see the docx skill)
soffice -env:UserInstallation="file:///tmp/lo_profile_$$" --headless \
  --convert-to pdf --outdir /home/codexis /home/codexis/smlouva.docx
```

Only reach for `pdf-lib` (Node, preinstalled) when you need to manipulate an *existing* PDF -
stamping, merging, filling fields - not to lay out a document from scratch.

## Page operations

```javascript
const fs = require('fs');
const { PDFDocument, degrees } = require('pdf-lib');

// Merge
const out = await PDFDocument.create();
for (const file of ['a.pdf', 'b.pdf']) {
  const src = await PDFDocument.load(fs.readFileSync(file));
  const pages = await out.copyPages(src, src.getPageIndices());
  pages.forEach((p) => out.addPage(p));
}
fs.writeFileSync('merged.pdf', await out.save());

// Split - keep pages 1-5
const src = await PDFDocument.load(fs.readFileSync('input.pdf'));
const part = await PDFDocument.create();
const pages = await part.copyPages(src, [0, 1, 2, 3, 4]);
pages.forEach((p) => part.addPage(p));
fs.writeFileSync('pages-1-5.pdf', await part.save());

// Rotate
const doc = await PDFDocument.load(fs.readFileSync('input.pdf'));
doc.getPage(0).setRotation(degrees(90));
fs.writeFileSync('rotated.pdf', await doc.save());
```

`qpdf` covers the same ground from the shell and is faster for big files:

```bash
qpdf --empty --pages a.pdf b.pdf -- merged.pdf
qpdf input.pdf --pages . 1-5 -- pages-1-5.pdf
qpdf --password=heslo --decrypt encrypted.pdf decrypted.pdf
```

## Filling forms

```javascript
const { PDFDocument } = require('pdf-lib');
const doc = await PDFDocument.load(fs.readFileSync('formular.pdf'));
const form = doc.getForm();

// List the fields first - never guess their names
form.getFields().forEach((f) => console.log(f.getName(), f.constructor.name));

form.getTextField('jmeno').setText('Jan Novák');
form.getCheckBox('souhlas').check();

form.flatten();   // bake the values in so they survive every viewer
fs.writeFileSync('formular-vyplneny.pdf', await doc.save());
```

**Always render the filled form and look at it.** Field names lie, text overflows its box,
and a value written into the wrong field looks perfectly fine in the code.

```bash
cdx-render -i formular-vyplneny.pdf
```

## Verification - required

1. `cdx-render -i output.pdf` and `Read` every page.
2. Assume there are problems. Look for: clipped or overflowing text, overlapping elements,
   broken tables, missing diacritics, blank pages, wrong page order after a merge, values in
   the wrong form field.
3. For anything over two pages, hand the images to a subagent with fresh eyes - you will
   otherwise see what you intended rather than what is on the page.
4. Fix and re-render before delivering.

Never ship internal citation tokens (`【…†L1-L2】`) into the PDF body, and use ASCII hyphens -
U+2011 renders as a box in several PDF viewers.

## Dependencies

- `cdx-render` - PDF → page PNGs (poppler)
- `pdf-lib` (Node, preinstalled) - page ops, forms, stamping
- `qpdf`, `pdftoppm` - shell-side page operations
- LibreOffice - docx → pdf

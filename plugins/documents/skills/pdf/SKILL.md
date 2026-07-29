---
uuid: 2aeefd95-cd11-431e-9b6c-b7dd54f3bbae
name: pdf
icon: icon.svg
description: "PDF - vygeneruj PDF, vyplň formulář, slouč/rozděl/orazítkuj, přečti naskenovanou smlouvu, rozsudek, fakturu (kde záleží na vzhledu). Ne pro čistý text - na ten stačí ocr."
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

Text extraction is exact but blind: it cannot tell you that a clause is struck through, that
a stamp covers the signature, or that a table has merged cells. The page image shows all of
that but is unreliable for long numbers, spisové značky and URLs. **Use both, page by page.**

```bash
# 1. Text
ocr /home/codexis/rozsudek.pdf
unzip -o /home/codexis/rozsudek.pdf-ocr.zip -d /tmp/rozsudek-text

# 2. Pages as images
cdx-render -i /home/codexis/rozsudek.pdf
```

Then `Read` the page PNGs alongside the extracted markdown and cross-check them. When they
disagree about a number or a name, **the image wins for layout, the text wins for exact
strings** - go back to the image and read the value off it.

A scanned PDF has no text layer at all; `ocr` handles the OCR, but always look at the pages
too - OCR silently mangles diacritics and handwritten annotations.

Long documents: render selectively rather than burning tokens on 200 pages.

```bash
cdx-render -i rozsudek.pdf -f 1 -l 3      # first three pages
```

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
- `ocr` - text extraction, including scanned pages
- `pdf-lib` (Node, preinstalled) - page ops, forms, stamping
- `qpdf`, `pdftoppm` - shell-side page operations
- LibreOffice - docx → pdf

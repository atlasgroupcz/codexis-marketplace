---
uuid: e99d4d14-570c-433b-870d-4a146fb02019
name: xlsx
icon: icon.svg
description: >-
  Use whenever a spreadsheet is the primary input or output — creating an .xlsx from scratch or
  from other data, editing an existing workbook without breaking its formulas, building financial
  models, cleaning messy tabular data, or converting between .xlsx/.csv/.tsv. Trigger even on a
  casual file reference ("ten excel ve stažených"). Triggers on "Excel", "xlsx", "tabulka",
  "sešit", "vzorec", "výpočet", "rozpočet", "model", "cash flow", "přehled nákladů", "spreadsheet".
  Do NOT trigger when the deliverable is a Word document, a chart image, an HTML report or a
  standalone script, even if tabular data is involved.
version: 1.0.0
allowed-tools: shell
i18n:
  cs:
    displayName: "Excel tabulky"
    summary: "Tvorba a úprava .xlsx — vzorce, formátování, finanční modely, kontrola chyb a vizuální náhled."
  en:
    displayName: "Excel Spreadsheets"
    summary: "Create and edit .xlsx — formulas, formatting, financial models, error checking, visual review."
  sk:
    displayName: "Excel tabuľky"
    summary: "Tvorba a úprava .xlsx — vzorce, formátovanie, finančné modely, kontrola chýb a náhľad."
---

# XLSX — creation, editing, verification

**IMPORTANT:** Stop on a genuine `Error:` line from `cdx-render` or `cdx-recalc` — report it and
don't blindly retry. A missing `soffice` means the guest image has no renderer; that is a setup
problem to report, not something to work around by computing the values yourself.

`cdx-recalc` exiting with code **2 is not a tool failure** — it is the finding you asked for:
the workbook has error cells. Read the reported types, fix the formulas, run it again. That is
the loop working, not a reason to give up or to hardcode the values.

## The rule that matters most: derived values are formulas, never constants

If you compute a total in JavaScript and write the number into the cell, you have delivered a
screenshot, not a spreadsheet. The user changes one input and everything silently stays wrong.

```javascript
// WRONG — the total is frozen
sheet.getCell('B10').value = data.reduce((a, b) => a + b, 0);

// RIGHT — the spreadsheet recalculates
sheet.getCell('B10').value = { formula: 'SUM(B2:B9)' };
```

The same goes for growth rates, margins, ratios and differences. And never bake a magic number
into a formula — put it in an input cell and reference it: `=H6*(1+$B$3)`, not `=H6*1.04`.

## Creating a workbook

Use `ExcelJS` (preinstalled; `NODE_PATH` is set, so `require('exceljs')` just works).

```javascript
const ExcelJS = require('exceljs');

const wb = new ExcelJS.Workbook();
const sheet = wb.addWorksheet('Model');

sheet.getCell('A1').value = 'Tržby (tis. Kč)';
sheet.getCell('A1').font = { name: 'Arial', bold: true };

sheet.getCell('B3').value = 0.04;                            // input
sheet.getCell('B3').font = { color: { argb: 'FF0000FF' } };  // blue = input
sheet.getCell('B3').numFmt = '0.0%';

sheet.getCell('C6').value = { formula: 'B6*(1+$B$3)' };      // black = formula
sheet.getCell('C6').numFmt = '#,##0;(#,##0);-';

sheet.getColumn('A').width = 32;

await wb.xlsx.writeFile('/home/codexis/model.xlsx');
```

Editing an existing workbook: `await wb.xlsx.readFile(path)`, change the cells, write it back.
ExcelJS preserves formulas and formatting it understands — but **match the existing style
exactly**. A workbook that already has a house format always overrides the conventions below;
never impose your formatting on someone else's template.

## Verification — two gates, cheap one first

**Gate 1 — recalculate. This is not optional.** ExcelJS writes formulas as strings with no
cached result. Until LibreOffice has recalculated the file, you have no idea whether it is full
of `#REF!`.

```bash
cdx-recalc -i /home/codexis/model.xlsx --in-place
# {"status":"ok","errors":0}
# {"status":"errors_found","errors":3,"types":["#REF!","#DIV/0!"]}
```

Exit code 2 means error cells. Fix them and run it again. **Ship only on `"errors":0`.**
Common causes: `#REF!` a range that points off the sheet, `#DIV/0!` a divisor cell that is
still empty, `#NAME?` a function LibreOffice does not know, or text starting with `=` (prefix
it with an apostrophe: `'=vysoká-nízká`).

**Gate 2 — render and look.**

```bash
cdx-render -i /home/codexis/model.xlsx
```

`Read` the images. Numbers formatted as raw floats, `###` columns too narrow to show their
value, headers indistinguishable from data, text spilling into the neighbouring cell — none of
that shows up in the XML, all of it is obvious in the picture. Recalculate *before* rendering,
otherwise the cells render empty.

## Formula constraints

- **Do not use dynamic array functions** — `FILTER`, `XLOOKUP`, `SORT`, `SEQUENCE`. ExcelJS and
  LibreOffice do not round-trip them in a way Excel accepts, and the user ends up with a broken
  workbook. Use `INDEX`/`MATCH` and `VLOOKUP` instead.
- Avoid volatile functions (`INDIRECT`, `OFFSET`) unless there is no alternative.
- Keep formulas legible. Intermediate results belong in helper cells, not nested five deep.
- Use `$B$4` / `B4` deliberately so that copying a formula sideways still does the right thing.

## Formatting

- Dates and years are not numbers: `2028`, never `2 028`.
- Currency always carries its unit in the header — `Tržby (tis. Kč)` — and uses `#,##0`.
- Percentages default to one decimal (`0.0%`), multiples to `0.0x`.
- Headers look different from data. Use fills, borders and merged cells sparingly; do not box
  every single cell.
- Column widths fit the content — no `###`, no text bleeding into the next column.

### Financial models

The industry colour convention, which Czech and Slovak finance teams read fluently:

| Colour | Meaning |
|---|---|
| **Blue text** (RGB 0,0,255) | Hardcoded inputs — the numbers the user will change |
| **Black text** (RGB 0,0,0) | All formulas and calculations |
| **Green text** (RGB 0,128,0) | Links from another sheet in the same workbook |
| **Red text** (RGB 255,0,0) | Links to another file |
| **Yellow fill** (RGB 255,255,0) | Key assumptions that need attention |

Plus: zeros format as `-`; negative numbers go in parentheses and red
(`#,##0;[Red](#,##0);-`); totals sum a contiguous range directly above them, with a horizontal
border above the total row; every hardcoded input cites its source in a cell comment
(`Zdroj: výroční zpráva 2024, s. 45`).

## Dependencies

- `ExcelJS` (Node, preinstalled) — creating and editing workbooks
- `cdx-recalc` — formula recalculation and error report (LibreOffice)
- `cdx-render` — workbook → page PNGs
- `ocr` — reading an existing workbook as markdown

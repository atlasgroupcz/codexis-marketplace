# documents

Create, edit and **visually verify** Word, PDF, Excel and PowerPoint documents.

The premise: an AI that writes a document and never looks at it ships broken tables, clipped
text and empty formula cells. Every skill here ends in the same loop — **generate → render to
page images → look → fix → re-render**. The render is the deliverable, not the code that
produced it.

## Skills

| Skill | Reads | Writes |
|---|---|---|
| `docx` | `ocr` (text) + `cdx-render` (pages), `pandoc` for tracked changes | `docx-js` |
| `pdf` | `ocr` + `cdx-render` | `.docx` → LibreOffice; `pdf-lib` for page ops and forms |
| `xlsx` | `ocr` + `cdx-render` | `ExcelJS`, gated by `cdx-recalc` |
| `pptx` | `ocr` + `cdx-render` | `pptxgenjs` |

Reading is always **text and images together**. Text is exact but blind to layout; the render
shows stamps, signatures and merged cells that extraction flattens away. Neither alone is enough.

## Commands installed on PATH

`postInstall` copies two scripts into `${HOME}/.local/bin`:

- **`cdx-render -i <file> [-o <dir>] [-r <dpi>] [-f <n>] [-l <n>]`** — any document to one PNG
  per page. Everything routes through LibreOffice → PDF → poppler. 150 DPI by default: readable,
  and cheap enough on vision tokens to render a whole document.
- **`cdx-recalc -i <file.xlsx> [--in-place]`** — recalculates formulas and reports error cells as
  JSON. Exit code 2 means the workbook has `#REF!` / `#DIV/0!` / `#NAME?` cells.

`cdx-recalc` exists because spreadsheet libraries write formulas as strings with **no cached
result**. A freshly written `.xlsx` literally contains `<c r="C6"><f>B6*(1+$B$3)</f></c>` — a
formula and no value. LibreOffice recalculates on load and caches the values back into the file.
Without this step you cannot know whether the model shipped a workbook full of errors.

## Guest image requirements

The plugin ships no binaries; it depends on the guest rootfs
(`vm-os-assets-plane`, `rootfs/Ubuntu 26.04/Dockerfile`) providing:

| Requirement | Used for | Status |
|---|---|---|
| `libreoffice` (headless) | docx/xlsx/pptx → PDF, formula recalculation | already in the rootfs |
| `poppler-utils` (`pdftoppm`, `pdftotext`) | PDF → page PNGs | already in the rootfs |
| `qpdf` | shell-side merge, split, rotate, decrypt | already in the rootfs |
| `pandoc` | tracked changes in `.docx` | already in the rootfs |
| `unzip`, `nodejs`, `npm`, DejaVu/Liberation fonts | scripts, generation, glyph coverage | already in the rootfs |
| npm globals: `docx`, `exceljs`, `pptxgenjs`, `pdf-lib` | document generation | **added by vm-os-assets-plane!5** |

Only the npm libraries were ever missing — the rootfs has carried the conversion and rendering
tooling since it was created. `NODE_PATH=/usr/local/lib/node_modules` is declared in
`plugin.json` (and exported from `/etc/profile.d` by the rootfs) because npm's global root is
**not** one of node's default resolution paths on Debian/Ubuntu: without it `require('docx')`
throws `MODULE_NOT_FOUND`.

These all live in the base rootfs, not in the per-user overlay. The rootfs is immutable,
hash-addressed and mounted read-only as `/dev/vda`, shared by every microVM on the node — so
LibreOffice costs disk once per node, not once per user. The per-VM writable overlay
(`writableRoot/<vmUuid>/<rootfsHash>/overlayfs.ext4`) and the per-uid home sit on top of it, and
the home survives a rootfs bump.

`hooks/install-scripts.sh` warns at install time if `soffice` or `pdftoppm` is missing rather
than letting the model discover it halfway through a task.

## Relationship to the `ocr` plugin

`ocr` is the text half and this plugin depends on it — it converts any document to Markdown via
a server-side endpoint. `documents` adds the two things `ocr` cannot do: **see** the page, and
**author** the file. Install both.
